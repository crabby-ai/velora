//! Paradex streaming implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use futures::stream::Stream;
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use velora_core::Price;

use crate::{
    auth::StarknetWalletAuth,
    common::WebSocketClient,
    traits::Streaming,
    types::{
        BalanceUpdate, Candle, ExchangeError, Interval, OrderBookUpdate, OrderUpdate,
        PositionUpdate, PriceLevel, Result, Side, StreamTrade, Symbol, Ticker, UserDataEvent,
    },
};

use super::types::*;

/// Paradex streaming component
pub struct ParadexStreaming {
    ws_client: Arc<RwLock<WebSocketClient>>,
    auth: Option<StarknetWalletAuth>,
}

impl ParadexStreaming {
    pub fn new(ws_client: Arc<RwLock<WebSocketClient>>, auth: Option<StarknetWalletAuth>) -> Self {
        Self { ws_client, auth }
    }

    /// Helper function to parse price from string
    fn parse_price(s: &str) -> Result<Price> {
        let decimal = Decimal::from_str(s)
            .map_err(|e| ExchangeError::ParseError(format!("Failed to parse price '{s}': {e}")))?;
        let f64_val = decimal.to_string().parse::<f64>().unwrap_or(0.0);
        Ok(Price::from(f64_val))
    }

    /// Helper function to parse decimal from string
    fn parse_decimal(s: &str) -> Result<Decimal> {
        Decimal::from_str(s)
            .map_err(|e| ExchangeError::ParseError(format!("Failed to parse decimal '{s}': {e}")))
    }

    /// Convert Paradex trade to common StreamTrade type
    fn convert_trade(trade_info: &ParadexTrade, symbol: &Symbol) -> Result<StreamTrade> {
        let price = Self::parse_price(&trade_info.price)?;
        let quantity = Self::parse_decimal(&trade_info.size)?;

        let side = match trade_info.side.as_str() {
            "BUY" => Side::Buy,
            "SELL" => Side::Sell,
            _ => Side::Buy,
        };

        let timestamp =
            DateTime::from_timestamp_millis(trade_info.created_at).unwrap_or_else(Utc::now);

        Ok(StreamTrade {
            symbol: symbol.clone(),
            trade_id: trade_info.id.clone(),
            price,
            quantity,
            side,
            timestamp,
            buyer_maker: side == Side::Sell, // In Paradex, SELL trades have buyer as maker
        })
    }

    /// Convert Paradex orderbook to common OrderBookUpdate type
    fn convert_orderbook(
        ob_data: &ParadexOrderBookData,
        symbol: &Symbol,
        max_depth: Option<usize>,
    ) -> Result<OrderBookUpdate> {
        let mut bids = Vec::new();
        for level in &ob_data.bids {
            if let (Some(price_str), Some(size_str)) = (level.first(), level.get(1)) {
                let price = Self::parse_price(price_str)?;
                let quantity = Self::parse_decimal(size_str)?;
                bids.push(PriceLevel { price, quantity });

                if let Some(depth) = max_depth {
                    if bids.len() >= depth {
                        break;
                    }
                }
            }
        }

        let mut asks = Vec::new();
        for level in &ob_data.asks {
            if let (Some(price_str), Some(size_str)) = (level.first(), level.get(1)) {
                let price = Self::parse_price(price_str)?;
                let quantity = Self::parse_decimal(size_str)?;
                asks.push(PriceLevel { price, quantity });

                if let Some(depth) = max_depth {
                    if asks.len() >= depth {
                        break;
                    }
                }
            }
        }

        let timestamp =
            DateTime::from_timestamp_millis(ob_data.last_updated_at).unwrap_or_else(Utc::now);

        Ok(OrderBookUpdate {
            symbol: symbol.clone(),
            bids,
            asks,
            final_update_id: Some(ob_data.seq_no),
            first_update_id: None,
            timestamp,
        })
    }
}

#[async_trait]
impl Streaming for ParadexStreaming {
    async fn subscribe_trades(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<StreamTrade>> + Send + Unpin>> {
        info!("Subscribing to Paradex trades for {}", symbol);

        // Clone the WebSocket client for the stream
        let ws_client = Arc::clone(&self.ws_client);
        let symbol_clone = symbol.clone();

        // Connect and subscribe
        {
            let mut ws = ws_client.write().await;
            if !ws.is_connected() {
                debug!("WebSocket not connected, connecting...");
                ws.connect().await?;
            }

            // Send subscription message (JSON-RPC 2.0 format)
            let subscription = ParadexWsSubscribe {
                jsonrpc: "2.0".to_string(),
                method: "subscribe".to_string(),
                params: ParadexWsSubscribeParams {
                    channel: format!("trades.{}", symbol.as_str()),
                },
                id: 1,
            };

            debug!("Sending subscription: {:?}", subscription);
            ws.send_json(&subscription).await?;
        }

        // Create the stream using unfold
        let stream = futures::stream::unfold(ws_client, move |client| {
            let symbol = symbol_clone.clone();
            async move {
                loop {
                    let mut ws = client.write().await;

                    match ws.recv().await {
                        Ok(Some(text)) => {
                            drop(ws); // Release lock before processing

                            debug!("Received WebSocket message: {}", text);

                            // Try to parse as trades message
                            match serde_json::from_str::<ParadexWsTrade>(&text) {
                                Ok(trade_msg) => {
                                    // Convert the trade from the message
                                    match Self::convert_trade(&trade_msg.params.data, &symbol) {
                                        Ok(trade) => return Some((Ok(trade), client)),
                                        Err(e) => {
                                            error!("Failed to convert trade: {}", e);
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Not a trade message (or parse error): {}", e);
                                    // Could be subscription confirmation or other message
                                    continue;
                                }
                            }
                        }
                        Ok(None) => {
                            drop(ws);
                            error!("WebSocket stream ended");
                            return None;
                        }
                        Err(e) => {
                            drop(ws);
                            error!("WebSocket receive error: {}", e);
                            return Some((Err(e), client));
                        }
                    }
                }
            }
        });

        Ok(Box::new(Box::pin(stream)))
    }

    async fn subscribe_orderbook(
        &self,
        symbol: &Symbol,
        depth: Option<usize>,
    ) -> Result<Box<dyn Stream<Item = Result<OrderBookUpdate>> + Send + Unpin>> {
        info!(
            "Subscribing to Paradex orderbook for {} (depth: {:?})",
            symbol, depth
        );

        let ws_client = Arc::clone(&self.ws_client);
        let symbol_clone = symbol.clone();

        // Connect and subscribe
        {
            let mut ws = ws_client.write().await;
            if !ws.is_connected() {
                debug!("WebSocket not connected, connecting...");
                ws.connect().await?;
            }

            // Send subscription message
            let subscription = ParadexWsSubscribe {
                jsonrpc: "2.0".to_string(),
                method: "subscribe".to_string(),
                params: ParadexWsSubscribeParams {
                    channel: format!("order_book.{}", symbol.as_str()),
                },
                id: 2,
            };

            debug!("Sending orderbook subscription: {:?}", subscription);
            ws.send_json(&subscription).await?;
        }

        // Create the stream
        let stream = futures::stream::unfold(ws_client, move |client| {
            let symbol = symbol_clone.clone();
            let max_depth = depth;

            async move {
                loop {
                    let mut ws = client.write().await;

                    match ws.recv().await {
                        Ok(Some(text)) => {
                            drop(ws);

                            debug!("Received WebSocket message: {}", text);

                            // Try to parse as orderbook message
                            match serde_json::from_str::<ParadexWsOrderBook>(&text) {
                                Ok(ob_msg) => {
                                    match Self::convert_orderbook(
                                        &ob_msg.params.data,
                                        &symbol,
                                        max_depth,
                                    ) {
                                        Ok(update) => return Some((Ok(update), client)),
                                        Err(e) => {
                                            error!("Failed to convert orderbook: {}", e);
                                            continue;
                                        }
                                    }
                                }
                                Err(e) => {
                                    debug!("Not an orderbook message: {}", e);
                                    continue;
                                }
                            }
                        }
                        Ok(None) => {
                            drop(ws);
                            error!("WebSocket stream ended");
                            return None;
                        }
                        Err(e) => {
                            drop(ws);
                            error!("WebSocket receive error: {}", e);
                            return Some((Err(e), client));
                        }
                    }
                }
            }
        });

        Ok(Box::new(Box::pin(stream)))
    }

    async fn subscribe_ticker(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<Ticker>> + Send + Unpin>> {
        Err(ExchangeError::Unsupported(format!(
            "Ticker WebSocket streaming not yet implemented for Paradex. Use REST API get_ticker() for symbol: {symbol}"
        )))
    }

    async fn subscribe_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
    ) -> Result<Box<dyn Stream<Item = Result<Candle>> + Send + Unpin>> {
        Err(ExchangeError::Unsupported(format!(
            "Candles WebSocket streaming not yet implemented for Paradex. Use REST API get_candles() for symbol: {symbol} interval: {interval:?}"
        )))
    }

    async fn subscribe_orders(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<OrderUpdate>> + Send + Unpin>> {
        Err(ExchangeError::Authentication(
            "Order updates WebSocket requires Starknet wallet authentication. Not yet implemented."
                .to_string(),
        ))
    }

    async fn subscribe_positions(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<PositionUpdate>> + Send + Unpin>> {
        Err(ExchangeError::Authentication(
            "Position updates WebSocket requires Starknet wallet authentication. Not yet implemented.".to_string()
        ))
    }

    async fn subscribe_balances(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<BalanceUpdate>> + Send + Unpin>> {
        Err(ExchangeError::Authentication(
            "Balance updates WebSocket requires Starknet wallet authentication. Not yet implemented.".to_string()
        ))
    }

    async fn subscribe_user_data(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<UserDataEvent>> + Send + Unpin>> {
        Err(ExchangeError::Authentication(
            "User data WebSocket requires Starknet wallet authentication. Not yet implemented."
                .to_string(),
        ))
    }
}
