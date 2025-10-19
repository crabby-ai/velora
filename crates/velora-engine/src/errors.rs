//! Error types for the live trading engine

use thiserror::Error;

/// Result type for engine operations
pub type EngineResult<T> = Result<T, EngineError>;

/// Errors that can occur during live trading engine operations
#[derive(Debug, Error)]
pub enum EngineError {
    /// Strategy error
    #[error("Strategy error: {0}")]
    Strategy(#[from] velora_strategy::StrategyError),

    /// Exchange API error
    #[error("Exchange error: {0}")]
    Exchange(String),

    /// Risk management violation
    #[error("Risk check failed: {0}")]
    RiskViolation(String),

    /// Order management error
    #[error("Order error: {0}")]
    OrderError(String),

    /// Position tracking error
    #[error("Position error: {0}")]
    PositionError(String),

    /// Rate limit exceeded
    #[error("Rate limit exceeded: max {max} orders per second")]
    RateLimitExceeded { max: u32 },

    /// Configuration error
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Engine not running
    #[error("Engine is not running")]
    NotRunning,

    /// Engine already running
    #[error("Engine is already running")]
    AlreadyRunning,

    /// Network/connection error
    #[error("Connection error: {0}")]
    ConnectionError(String),

    /// Market data error
    #[error("Market data error: {0}")]
    MarketDataError(String),

    /// Insufficient capital
    #[error("Insufficient capital: available={available}, required={required}")]
    InsufficientCapital { available: f64, required: f64 },

    /// Order not found
    #[error("Order not found: {0}")]
    OrderNotFound(String),

    /// Position not found
    #[error("Position not found: {0}")]
    PositionNotFound(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Generic error
    #[error("{0}")]
    Other(String),
}
