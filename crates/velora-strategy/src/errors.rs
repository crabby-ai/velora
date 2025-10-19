//! Error types for the strategy framework.

use thiserror::Error;

/// Result type for strategy operations
pub type StrategyResult<T> = Result<T, StrategyError>;

/// Errors that can occur during strategy execution
#[derive(Debug, Error)]
pub enum StrategyError {
    /// Strategy initialization failed
    #[error("Strategy initialization failed: {0}")]
    InitializationFailed(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// Invalid signal
    #[error("Invalid signal: {0}")]
    InvalidSignal(String),

    /// Position management error
    #[error("Position error: {0}")]
    PositionError(String),

    /// Market data not available
    #[error("Market data not available: {0}")]
    DataNotAvailable(String),

    /// Indicator error
    #[error("Indicator error: {0}")]
    IndicatorError(#[from] velora_ta::IndicatorError),

    /// Exchange error
    #[error("Exchange error: {0}")]
    ExchangeError(String),

    /// Insufficient capital
    #[error("Insufficient capital: available={available}, required={required}")]
    InsufficientCapital { available: f64, required: f64 },

    /// Risk limit exceeded
    #[error("Risk limit exceeded: {0}")]
    RiskLimitExceeded(String),

    /// Strategy already running
    #[error("Strategy is already running")]
    AlreadyRunning,

    /// Strategy not running
    #[error("Strategy is not running")]
    NotRunning,

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}
