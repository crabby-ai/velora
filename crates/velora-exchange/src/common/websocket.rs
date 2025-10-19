//! WebSocket client for real-time exchange data streams

use futures::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use serde::{Deserialize, Serialize};
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, tungstenite::Message, MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info, warn};

use crate::types::{ExchangeError, Result};

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    /// Raw message data
    pub data: String,
}

/// WebSocket client for exchange streaming connections
pub struct WebSocketClient {
    /// WebSocket URL
    url: String,

    /// Write half of the WebSocket stream
    write: Option<SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>>,

    /// Read half of the WebSocket stream
    read: Option<SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>>,

    /// Connection status
    connected: bool,
}

impl WebSocketClient {
    /// Create a new WebSocket client
    ///
    /// # Arguments
    /// * `url` - WebSocket URL (e.g., "wss://stream.binance.com:9443/ws")
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            write: None,
            read: None,
            connected: false,
        }
    }

    /// Connect to the WebSocket server
    pub async fn connect(&mut self) -> Result<()> {
        info!("Connecting to WebSocket: {}", self.url);

        let (ws_stream, response) = connect_async(&self.url)
            .await
            .map_err(|e| ExchangeError::WebSocket(format!("Connection failed: {e}")))?;

        debug!("WebSocket connected with response: {:?}", response);

        let (write, read) = ws_stream.split();
        self.write = Some(write);
        self.read = Some(read);
        self.connected = true;

        Ok(())
    }

    /// Send a message to the WebSocket
    pub async fn send(&mut self, message: &str) -> Result<()> {
        if !self.connected {
            return Err(ExchangeError::WebSocket("Not connected".to_string()));
        }

        let write = self
            .write
            .as_mut()
            .ok_or_else(|| ExchangeError::WebSocket("Write stream not available".to_string()))?;

        write
            .send(Message::Text(message.to_string()))
            .await
            .map_err(|e| ExchangeError::WebSocket(format!("Send failed: {e}")))?;

        Ok(())
    }

    /// Send a JSON-serializable message
    pub async fn send_json<T: Serialize>(&mut self, message: &T) -> Result<()> {
        let json = serde_json::to_string(message)
            .map_err(|e| ExchangeError::ParseError(format!("JSON serialization failed: {e}")))?;
        self.send(&json).await
    }

    /// Subscribe to a stream by sending a subscription message
    pub async fn subscribe(&mut self, subscribe_message: &str) -> Result<()> {
        debug!("Subscribing with message: {}", subscribe_message);
        self.send(subscribe_message).await
    }

    /// Receive the next message from the WebSocket
    pub async fn recv(&mut self) -> Result<Option<String>> {
        loop {
            if !self.connected {
                return Err(ExchangeError::WebSocket("Not connected".to_string()));
            }

            let read = self
                .read
                .as_mut()
                .ok_or_else(|| ExchangeError::WebSocket("Read stream not available".to_string()))?;

            match read.next().await {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => return Ok(Some(text)),
                    Message::Binary(data) => {
                        let text = String::from_utf8(data).map_err(|e| {
                            ExchangeError::ParseError(format!("UTF-8 decode error: {e}"))
                        })?;
                        return Ok(Some(text));
                    }
                    Message::Ping(data) => {
                        debug!("Received ping, sending pong");
                        if let Some(write) = self.write.as_mut() {
                            write.send(Message::Pong(data)).await.ok();
                        }
                        // Continue loop to get next message
                        continue;
                    }
                    Message::Pong(_) => {
                        debug!("Received pong");
                        // Continue loop to get next message
                        continue;
                    }
                    Message::Close(frame) => {
                        warn!("WebSocket closed: {:?}", frame);
                        self.connected = false;
                        return Ok(None);
                    }
                    _ => {
                        warn!("Received unexpected message type");
                        // Continue loop to get next message
                        continue;
                    }
                },
                Some(Err(e)) => {
                    error!("WebSocket error: {}", e);
                    self.connected = false;
                    return Err(ExchangeError::WebSocket(format!("Receive error: {e}")));
                }
                None => {
                    info!("WebSocket stream ended");
                    self.connected = false;
                    return Ok(None);
                }
            }
        }
    }

    /// Receive and parse a JSON message
    pub async fn recv_json<T: for<'de> Deserialize<'de>>(&mut self) -> Result<Option<T>> {
        match self.recv().await? {
            Some(text) => {
                let parsed = serde_json::from_str(&text)
                    .map_err(|e| ExchangeError::ParseError(format!("JSON parse error: {e}")))?;
                Ok(Some(parsed))
            }
            None => Ok(None),
        }
    }

    /// Send a ping message
    pub async fn ping(&mut self) -> Result<()> {
        if !self.connected {
            return Err(ExchangeError::WebSocket("Not connected".to_string()));
        }

        let write = self
            .write
            .as_mut()
            .ok_or_else(|| ExchangeError::WebSocket("Write stream not available".to_string()))?;

        write
            .send(Message::Ping(vec![]))
            .await
            .map_err(|e| ExchangeError::WebSocket(format!("Ping failed: {e}")))?;

        Ok(())
    }

    /// Close the WebSocket connection
    pub async fn close(&mut self) -> Result<()> {
        if !self.connected {
            return Ok(());
        }

        info!("Closing WebSocket connection");

        if let Some(mut write) = self.write.take() {
            write
                .close()
                .await
                .map_err(|e| ExchangeError::WebSocket(format!("Close failed: {e}")))?;
        }

        self.connected = false;
        Ok(())
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.connected
    }

    /// Get the WebSocket URL
    pub fn url(&self) -> &str {
        &self.url
    }
}

impl Drop for WebSocketClient {
    fn drop(&mut self) {
        if self.connected {
            warn!("WebSocketClient dropped while still connected");
        }
    }
}

impl std::fmt::Debug for WebSocketClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSocketClient")
            .field("url", &self.url)
            .field("connected", &self.connected)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = WebSocketClient::new("wss://example.com/ws");
        assert_eq!(client.url(), "wss://example.com/ws");
        assert!(!client.is_connected());
    }
}
