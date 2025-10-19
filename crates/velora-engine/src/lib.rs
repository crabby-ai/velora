//! # velora-engine
//!
//! Live trading engine for the Velora HFT platform.
//!
//! ## Features
//!
//! - Real-time strategy execution
//! - Order and position management
//! - Dry-run (paper trading) mode
//! - Event-driven architecture
//! - Comprehensive monitoring and logging
//!
//! ## Example
//!
//! ```no_run
//! use velora_engine::{TradingEngine, EngineConfig, ExecutionMode};
//! use velora_strategy::SmaCrossoverStrategy;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create strategy
//!     let strategy = SmaCrossoverStrategy::new("BTC-USD-PERP", 10, 50)?;
//!
//!     // Configure engine
//!     let config = EngineConfig::builder()
//!         .mode(ExecutionMode::DryRun)
//!         .add_symbol("BTC-USD-PERP".to_string())
//!         .initial_capital(10_000.0)
//!         .build();
//!
//!     // Create and start engine
//!     let mut engine = TradingEngine::new(config)
//!         .with_strategy(Box::new(strategy));
//!
//!     // Start trading
//!     engine.start().await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

mod config;
mod engine;
mod errors;
mod events;
mod execution;
mod order_manager;
mod position_tracker;

pub use config::{EngineConfig, ExecutionMode, MetricsConfig};
pub use engine::{EngineState, EngineStatus, TradingEngine};
pub use errors::{EngineError, EngineResult};
pub use events::{Fill, MarketEvent, OrderStatus, OrderUpdate};
pub use execution::ExecutionHandler;
pub use order_manager::{Order, OrderEvent, OrderEventType, OrderManager};
pub use position_tracker::{EquitySnapshot, Position, PositionTracker};
