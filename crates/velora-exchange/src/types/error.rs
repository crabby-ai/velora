//! Exchange error types.

use thiserror::Error;

/// Exchange operation result type
pub type Result<T> = std::result::Result<T, ExchangeError>;

/// Exchange error types
#[derive(Error, Debug)]
pub enum ExchangeError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Authentication failed
    #[error("Authentication failed: {0}")]
    Authentication(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Invalid order parameters
    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    /// Insufficient balance
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    /// Order not found
    #[error("Order not found: {0}")]
    OrderNotFound(String),

    /// Market not found
    #[error("Market not found: {0}")]
    MarketNotFound(String),

    /// API error from exchange
    #[error("API error {code}: {message}")]
    ApiError {
        /// Error code
        code: i32,
        /// Error message
        message: String,
    },

    /// Network error
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// WebSocket error
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    /// Invalid request
    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    /// Timeout error
    #[error("Operation timed out")]
    Timeout,

    /// Unsupported feature
    #[error("Unsupported: {0}")]
    Unsupported(String),

    /// Unsupported exchange
    #[error("Unsupported exchange: {0}")]
    UnsupportedExchange(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),

    /// Not connected to exchange
    #[error("Not connected to exchange")]
    NotConnected,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}

// Implement conversions for common error types
impl From<serde_json::Error> for ExchangeError {
    fn from(err: serde_json::Error) -> Self {
        ExchangeError::Serialization(err.to_string())
    }
}

impl From<std::io::Error> for ExchangeError {
    fn from(err: std::io::Error) -> Self {
        ExchangeError::Connection(err.to_string())
    }
}
