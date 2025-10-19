//! Error types for the backtesting engine.

use thiserror::Error;

/// Result type for backtest operations
pub type BacktestResult<T> = Result<T, BacktestError>;

/// Errors that can occur during backtesting
#[derive(Debug, Error)]
pub enum BacktestError {
    /// Strategy error during backtest
    #[error("Strategy error: {0}")]
    Strategy(#[from] velora_strategy::StrategyError),

    /// Data loading error
    #[error("Data error: {0}")]
    DataError(String),

    /// Invalid configuration
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    /// No data available for the specified period
    #[error("No data available for symbol {symbol} between {start} and {end}")]
    NoData {
        symbol: String,
        start: String,
        end: String,
    },

    /// Insufficient capital
    #[error("Insufficient capital: available={available}, required={required}")]
    InsufficientCapital { available: f64, required: f64 },

    /// Invalid order
    #[error("Invalid order: {0}")]
    InvalidOrder(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON serialization error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// CSV error
    #[error("CSV error: {0}")]
    Csv(String),

    /// Internal error
    #[error("Internal error: {0}")]
    Internal(String),
}
