//! HTTP REST client wrapper for exchange API requests

use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    Client, Method, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;
use tracing::{debug, error};

use crate::types::{ExchangeError, Result};

/// REST client for making HTTP requests to exchange APIs
pub struct RestClient {
    /// Underlying HTTP client
    client: Client,

    /// Base URL for the API
    base_url: String,

    /// Default headers to include in all requests
    default_headers: HeaderMap,
}

impl RestClient {
    /// Create a new REST client
    ///
    /// # Arguments
    /// * `base_url` - Base URL for the exchange API (e.g., "https://api.binance.com")
    /// * `timeout` - Request timeout duration
    pub fn new(base_url: impl Into<String>, timeout: Duration) -> Result<Self> {
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .map_err(ExchangeError::Network)?;

        Ok(Self {
            client,
            base_url: base_url.into(),
            default_headers: HeaderMap::new(),
        })
    }

    /// Set a default header
    pub fn set_header(&mut self, key: impl AsRef<str>, value: impl AsRef<str>) -> Result<()> {
        let header_name = HeaderName::from_bytes(key.as_ref().as_bytes())
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid header name: {e}")))?;
        let header_value = HeaderValue::from_str(value.as_ref())
            .map_err(|e| ExchangeError::InvalidRequest(format!("Invalid header value: {e}")))?;

        self.default_headers.insert(header_name, header_value);
        Ok(())
    }

    /// Perform a GET request
    pub async fn get<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let empty: Option<&()> = None;
        self.request(Method::GET, endpoint, empty, HeaderMap::new())
            .await
    }

    /// Perform a GET request with query parameters
    pub async fn get_with_params<T: DeserializeOwned, P: Serialize>(
        &self,
        endpoint: &str,
        params: &P,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        debug!("GET request to: {}", url);

        let request = self
            .client
            .get(&url)
            .query(params)
            .headers(self.default_headers.clone());

        let response = request.send().await.map_err(ExchangeError::Network)?;

        self.handle_response(response).await
    }

    /// Perform a POST request
    pub async fn post<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.request(Method::POST, endpoint, Some(body), HeaderMap::new())
            .await
    }

    /// Perform a PUT request
    pub async fn put<T: DeserializeOwned, B: Serialize>(
        &self,
        endpoint: &str,
        body: &B,
    ) -> Result<T> {
        self.request(Method::PUT, endpoint, Some(body), HeaderMap::new())
            .await
    }

    /// Perform a DELETE request
    pub async fn delete<T: DeserializeOwned>(&self, endpoint: &str) -> Result<T> {
        let empty: Option<&()> = None;
        self.request(Method::DELETE, endpoint, empty, HeaderMap::new())
            .await
    }

    /// Perform a generic HTTP request
    pub async fn request<T: DeserializeOwned, B: Serialize>(
        &self,
        method: Method,
        endpoint: &str,
        body: Option<&B>,
        extra_headers: HeaderMap,
    ) -> Result<T> {
        let url = format!("{}{}", self.base_url, endpoint);
        debug!("{} request to: {}", method, url);

        let mut request = self
            .client
            .request(method.clone(), &url)
            .headers(self.default_headers.clone())
            .headers(extra_headers);

        if let Some(body) = body {
            request = request.json(body);
        }

        let response = request.send().await.map_err(ExchangeError::Network)?;

        self.handle_response(response).await
    }

    /// Handle HTTP response and parse JSON
    async fn handle_response<T: DeserializeOwned>(&self, response: Response) -> Result<T> {
        let status = response.status();

        if status.is_success() {
            response
                .json::<T>()
                .await
                .map_err(|e| ExchangeError::ParseError(format!("Failed to parse response: {e}")))
        } else {
            let error_text = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());

            error!("HTTP error {}: {}", status, error_text);

            match status {
                StatusCode::TOO_MANY_REQUESTS => {
                    Err(ExchangeError::RateLimit("Rate limit exceeded".to_string()))
                }
                StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(
                    ExchangeError::Authentication(format!("Auth failed: {error_text}")),
                ),
                StatusCode::BAD_REQUEST => Err(ExchangeError::InvalidRequest(error_text)),
                _ => Err(ExchangeError::ApiError {
                    code: status.as_u16() as i32,
                    message: error_text,
                }),
            }
        }
    }

    /// Get the base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }
}

impl std::fmt::Debug for RestClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RestClient")
            .field("base_url", &self.base_url)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let client = RestClient::new("https://api.example.com", Duration::from_secs(30));
        assert!(client.is_ok());

        let client = client.unwrap();
        assert_eq!(client.base_url(), "https://api.example.com");
    }

    #[test]
    fn test_set_header() {
        let mut client =
            RestClient::new("https://api.example.com", Duration::from_secs(30)).unwrap();
        assert!(client.set_header("X-API-KEY", "test_key").is_ok());
    }
}
