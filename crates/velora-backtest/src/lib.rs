//! # velora-backtest
//!
//! Backtesting engine for Velora trading strategies.
//!
//! This crate provides a realistic backtesting environment for validating
//! trading strategies with historical data before deploying them to live trading.
//!
//! ## Features
//!
//! - **Realistic Simulation**: Models commission fees, slippage, and market impact
//! - **Event-Driven**: Same execution model as live trading
//! - **Comprehensive Analytics**: Detailed performance metrics (Sharpe, drawdown, win rate, etc.)
//! - **Multiple Fill Models**: Market, realistic, and pessimistic execution
//! - **Fast Execution**: Process years of data in seconds
//!
//! ## Quick Start
//!
//! ```ignore
//! use velora_backtest::{Backtester, BacktestConfig, ExecutionConfig};
//! use velora_strategy::Strategy;
//!
//! // Create your strategy
//! let strategy = MyStrategy::new()?;
//!
//! // Configure backtest
//! let config = BacktestConfig::new()
//!     .with_capital(10_000.0)
//!     .with_symbols(vec!["BTC-USD-PERP".to_string()])
//!     .with_execution(ExecutionConfig::realistic());
//!
//! // Run backtest
//! let report = Backtester::new(config)
//!     .with_strategy(Box::new(strategy))
//!     .run()
//!     .await?;
//!
//! // Analyze results
//! report.print_summary();
//! ```

#![warn(missing_docs)]

pub mod backtester;
pub mod config;
pub mod errors;
pub mod execution;
pub mod performance;
pub mod portfolio;

// Re-exports
pub use backtester::{BacktestReport, Backtester};
pub use config::{BacktestConfig, ExecutionConfig, FillModel};
pub use errors::{BacktestError, BacktestResult};
pub use execution::{ExecutionSimulator, Fill, Order, OrderId, OrderStatus};
pub use performance::PerformanceMetrics;
pub use portfolio::{CompletedTrade, EquityPoint, Portfolio};
