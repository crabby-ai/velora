//! Error types for the Velora trading platform.

use thiserror::Error;

/// Main error type for the Velora platform.
#[derive(Error, Debug)]
pub enum VeloraError {
    // Data errors
    /// General data error
    #[error("Data error: {0}")]
    DataError(String),

    /// Invalid market data received
    #[error("Invalid market data: {0}")]
    InvalidMarketData(String),

    /// Requested data not found
    #[error("Data not found: {0}")]
    DataNotFound(String),

    // Exchange errors
    /// General exchange error
    #[error("Exchange error: {0}")]
    ExchangeError(String),

    /// Connection error to exchange
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Authentication error with exchange
    #[error("Authentication error: {0}")]
    AuthenticationError(String),

    /// Exchange rate limit exceeded
    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    /// Invalid response from exchange API
    #[error("Invalid API response: {0}")]
    InvalidResponse(String),

    // Order errors
    /// General order error
    #[error("Order error: {0}")]
    OrderError(String),

    /// Insufficient balance for order
    #[error("Insufficient balance: {0}")]
    InsufficientBalance(String),

    /// Invalid order parameters
    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    /// Order not found
    #[error("Order not found: {0}")]
    OrderNotFound(String),

    // Strategy errors
    /// General strategy error
    #[error("Strategy error: {0}")]
    StrategyError(String),

    /// Invalid strategy configuration
    #[error("Invalid strategy configuration: {0}")]
    InvalidStrategyConfig(String),

    // Risk management errors
    /// Risk limit exceeded
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    /// Position limit exceeded
    #[error("Position limit exceeded: {0}")]
    PositionLimitExceeded(String),

    // Configuration errors
    /// General configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Missing required configuration
    #[error("Missing configuration: {0}")]
    MissingConfig(String),

    // IO and serialization errors
    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Parse error
    #[error("Parse error: {0}")]
    ParseError(String),

    // General errors
    /// Internal error (should not happen in normal operation)
    #[error("Internal error: {0}")]
    InternalError(String),

    /// Not implemented feature
    #[error("Not implemented: {0}")]
    NotImplemented(String),

    /// Invalid parameter
    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),
}

// Implement conversions from common error types
impl From<serde_json::Error> for VeloraError {
    fn from(err: serde_json::Error) -> Self {
        VeloraError::SerializationError(err.to_string())
    }
}

impl From<config::ConfigError> for VeloraError {
    fn from(err: config::ConfigError) -> Self {
        VeloraError::ConfigError(err.to_string())
    }
}

impl From<gonfig::Error> for VeloraError {
    fn from(err: gonfig::Error) -> Self {
        VeloraError::ConfigError(err.to_string())
    }
}

/// Result type alias for Velora operations.
pub type Result<T> = std::result::Result<T, VeloraError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let error = VeloraError::InvalidOrder("Price must be positive".to_string());
        assert_eq!(error.to_string(), "Invalid order: Price must be positive");
    }

    #[test]
    fn test_error_from_io() {
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let velora_error: VeloraError = io_error.into();
        assert!(matches!(velora_error, VeloraError::IoError(_)));
    }

    #[test]
    fn test_result_type() {
        fn returns_result() -> Result<i32> {
            Ok(42)
        }

        assert_eq!(returns_result().unwrap(), 42);
    }
}
