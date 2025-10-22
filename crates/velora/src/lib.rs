//! # Velora
//!
//! A modular, high-performance algorithmic trading platform for cryptocurrency markets.
//!
//! ## Overview
//!
//! Velora is a comprehensive trading platform built in Rust that provides everything you need
//! to develop, test, and deploy trading strategies:
//!
//! - **Market Data** - Ingest and manage data from CSV, Parquet, PostgreSQL
//! - **Technical Analysis** - 40+ indicators and candlestick pattern recognition
//! - **Strategy Framework** - Async trait-based strategy development
//! - **Backtesting** - Realistic simulation with comprehensive metrics
//! - **Live Trading** - Event-driven engine with dry-run support
//! - **Exchange Integration** - Connect to DEXs (Lighter, Paradex) and CEXs
//! - **Risk Management** - Position sizing and limit enforcement
//!
//! ## Architecture
//!
//! Velora is organized into modular crates, each with a specific responsibility:
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                        velora                           │  ← You are here
//! │                   (Umbrella Crate)                      │
//! └─────────────────────────────────────────────────────────┘
//!           │
//!           ├── velora-core      (types, config, errors)
//!           ├── velora-data      (data ingestion & storage)
//!           ├── velora-ta        (technical analysis)
//!           ├── velora-strategy  (strategy framework)
//!           ├── velora-backtest  (backtesting engine)
//!           ├── velora-engine    (live trading engine)
//!           ├── velora-exchange  (exchange connectivity)
//!           └── velora-risk      (risk management)
//! ```
//!
//! ## Quick Start
//!
//! ### Backtesting a Strategy
//!
//! ```rust,ignore
//! use velora::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Load historical data
//!     let data_source = CsvDataSource::new("data/BTCUSDT.csv")?;
//!     let candles = data_source.load_candles(
//!         &Symbol::new("BTCUSDT"),
//!         Timeframe::H1
//!     ).await?;
//!
//!     // 2. Create your strategy
//!     let strategy = MyStrategy::new(/* config */);
//!
//!     // 3. Run backtest
//!     let mut engine = BacktestEngine::builder()
//!         .initial_capital(10000.0)
//!         .commission_rate(0.001)
//!         .build();
//!
//!     let results = engine.run(strategy, candles).await?;
//!
//!     // 4. Analyze results
//!     println!("Total Return: {:.2}%", results.total_return() * 100.0);
//!     println!("Sharpe Ratio: {:.2}", results.sharpe_ratio());
//!     println!("Max Drawdown: {:.2}%", results.max_drawdown() * 100.0);
//!
//!     Ok(())
//! }
//! ```
//!
//! ### Live Trading (Dry Run)
//!
//! ```rust,ignore
//! use velora::prelude::*;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // 1. Configure exchange connection
//!     let exchange = LighterExchange::new(
//!         ExchangeConfig::from_env()?
//!     ).await?;
//!
//!     // 2. Create your strategy
//!     let strategy = MyStrategy::new(/* config */);
//!
//!     // 3. Configure risk management
//!     let risk_manager = RiskManager::builder()
//!         .max_position_size(1000.0)
//!         .max_drawdown(0.15)
//!         .build();
//!
//!     // 4. Start trading engine in dry-run mode
//!     let engine = TradingEngine::builder()
//!         .mode(ExecutionMode::DryRun)
//!         .exchange(exchange)
//!         .strategy(strategy)
//!         .risk_manager(risk_manager)
//!         .build();
//!
//!     engine.run().await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! The crate uses feature flags to allow selective compilation:
//!
//! - `full` (default) - All features enabled
//! - `data` - Data ingestion and management
//! - `ta` - Technical analysis indicators
//! - `strategy` - Strategy development framework
//! - `backtest` - Backtesting engine
//! - `engine` - Live trading engine
//! - `exchange` - Exchange integrations
//! - `risk` - Risk management
//! - `utils` - Integration utilities
//!
//! To use only specific features:
//!
//! ```toml
//! [dependencies]
//! velora = { version = "0.0.1", default-features = false, features = ["backtest", "ta"] }
//! ```
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//!
//! - `simple_backtest.rs` - Basic backtesting workflow
//! - `live_trading.rs` - Live trading with dry-run mode
//! - `strategy_development.rs` - End-to-end strategy creation
//!
//! ## Safety and Performance
//!
//! Velora leverages Rust's type system to prevent common trading bugs:
//!
//! - **Type-safe orders** - Compile-time guarantees on order validity
//! - **Zero-cost abstractions** - No runtime overhead for safety
//! - **Async-first** - Efficient concurrent operations
//! - **Memory safe** - No data races or memory leaks
//!
//! ## Documentation
//!
//! For detailed documentation, see:
//!
//! - [Architecture Guide](../ARCHITECTURE.md)
//! - [Development Roadmap](../ROADMAP.md)
//! - [Contributing Guidelines](../CONTRIBUTING.md)

#![warn(missing_docs)]
#![deny(unsafe_code)]

// Re-export core types (always available)
pub use velora_core::*;

// Re-export optional modules based on features
#[cfg(feature = "data")]
pub use velora_data as data;

#[cfg(feature = "exchange")]
pub use velora_exchange as exchange;

#[cfg(feature = "ta")]
pub use velora_ta as ta;

#[cfg(feature = "strategy")]
pub use velora_strategy as strategy;

#[cfg(feature = "backtest")]
pub use velora_backtest as backtest;

#[cfg(feature = "engine")]
pub use velora_engine as engine;

#[cfg(feature = "risk")]
pub use velora_risk as risk;

/// Prelude module for convenient imports
///
/// This module re-exports the most commonly used types and traits from all modules,
/// allowing you to get started quickly with a single import:
///
/// ```rust,ignore
/// use velora::prelude::*;
/// ```
pub mod prelude {
    //! Convenient re-exports of commonly used types and traits.

    // Core types (always available)
    pub use velora_core::*;

    // Data module
    #[cfg(feature = "data")]
    pub use velora_data::*;

    // Technical analysis
    #[cfg(feature = "ta")]
    pub use velora_ta::*;

    // Exchange integration
    #[cfg(feature = "exchange")]
    pub use velora_exchange::prelude::*;

    // Strategy framework
    #[cfg(feature = "strategy")]
    pub use velora_strategy::*;

    // Backtesting
    #[cfg(feature = "backtest")]
    pub use velora_backtest::*;

    // Live trading engine
    #[cfg(feature = "engine")]
    pub use velora_engine::*;

    // Risk management
    #[cfg(feature = "risk")]
    pub use velora_risk::*;
}

/// Integration utilities for common workflows
///
/// This module provides helper functions and builders that combine multiple
/// Velora components to simplify common use cases.
#[cfg(feature = "utils")]
pub mod utils {
    //! Utility functions for common integration patterns.

    use super::*;

    /// Result type for utility functions
    pub type Result<T> = std::result::Result<T, anyhow::Error>;

    /// Helper for setting up logging and tracing
    #[cfg(feature = "utils")]
    pub fn init_tracing(level: &str) -> Result<()> {
        use tracing_subscriber::prelude::*;

        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(level));

        tracing_subscriber::registry()
            .with(filter)
            .with(tracing_subscriber::fmt::layer())
            .init();

        Ok(())
    }

    /// Common configuration for backtesting
    #[cfg(all(feature = "backtest", feature = "utils"))]
    pub struct BacktestConfig {
        /// Initial capital in quote currency
        pub initial_capital: f64,
        /// Commission rate (e.g., 0.001 for 0.1%)
        pub commission_rate: f64,
        /// Slippage in basis points
        pub slippage_bps: f64,
    }

    #[cfg(all(feature = "backtest", feature = "utils"))]
    impl Default for BacktestConfig {
        fn default() -> Self {
            Self {
                initial_capital: 10000.0,
                commission_rate: 0.001,
                slippage_bps: 5.0,
            }
        }
    }

    /// Common configuration for live trading
    #[cfg(all(feature = "engine", feature = "utils"))]
    pub struct LiveTradingConfig {
        /// Execution mode (Live or DryRun)
        pub mode: crate::engine::ExecutionMode,
        /// Whether to enable health checks
        pub health_checks: bool,
        /// Heartbeat interval in seconds
        pub heartbeat_interval: u64,
    }

    #[cfg(all(feature = "engine", feature = "utils"))]
    impl Default for LiveTradingConfig {
        fn default() -> Self {
            Self {
                mode: crate::engine::ExecutionMode::DryRun,
                health_checks: true,
                heartbeat_interval: 30,
            }
        }
    }
}

/// Version information
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Platform name
pub const PLATFORM_NAME: &str = "Velora";

/// Get the full version string
pub fn version_string() -> String {
    format!("{} v{}", PLATFORM_NAME, VERSION)
}
