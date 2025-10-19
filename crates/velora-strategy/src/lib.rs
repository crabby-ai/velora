//! # velora-strategy
//!
//! Strategy framework for the Velora HFT platform.
//!
//! This crate provides the core strategy building framework, allowing users to:
//! - Implement custom trading strategies using the `Strategy` trait
//! - Access market data and manage positions via `StrategyContext`
//! - Generate trading signals (`Signal`)
//! - Track positions and P&L
//!
//! Technical indicators are provided by the standalone `velora-ta` crate.
//!
//! ## Quick Start
//!
//! ### Using Technical Indicators
//!
//! ```ignore
//! use velora_strategy::indicators::{SMA, EMA, RSI, SingleIndicator};
//!
//! // Create indicators
//! let mut sma = SMA::new(20)?;
//! let mut rsi = RSI::new(14)?;
//!
//! // Stream mode: process prices one at a time
//! for price in prices {
//!     if let Some(sma_val) = sma.update(price, timestamp)? {
//!         println!("SMA(20): {:.2}", sma_val);
//!     }
//!
//!     if let Some(rsi_val) = rsi.update(price, timestamp)? {
//!         if rsi_val > 70.0 {
//!             println!("Overbought!");
//!         }
//!     }
//! }
//! ```
//!
//! ### Building a Strategy
//!
//! ```ignore
//! use velora_strategy::{Strategy, StrategyContext, Signal};
//! use async_trait::async_trait;
//!
//! struct MyStrategy {
//!     // strategy fields
//! }
//!
//! #[async_trait]
//! impl Strategy for MyStrategy {
//!     fn name(&self) -> &str { "My Strategy" }
//!
//!     async fn on_candle(&mut self, candle: &Candle, ctx: &StrategyContext) -> StrategyResult<Signal> {
//!         // Your strategy logic here
//!         Ok(Signal::Hold)
//!     }
//!
//!     // Implement other required methods...
//! }
//! ```
//!
//! ## Features
//!
//! - **Strategy Trait**: Base trait for all trading strategies
//! - **Signal Types**: Buy, Sell, Close, Modify, Hold
//! - **Position Management**: Track positions and calculate P&L
//! - **Strategy Context**: Access market data, positions, and capital
//! - **Event-Driven**: React to candles, trades, ticks, fills, and timers
//! - **Async Support**: Full async/await support for non-blocking operations

#![warn(missing_docs)]

pub mod context;
pub mod errors;
pub mod strategy;
pub mod types;

// Re-export all technical indicators from velora-ta
/// Technical indicators module (re-exported from velora-ta)
pub mod indicators {
    pub use velora_ta::*;
}

// Re-export core types
pub use context::{MarketSnapshot, StrategyContext};
pub use errors::{StrategyError, StrategyResult};
pub use strategy::{ParameterInfo, Strategy, StrategyMetadata};
pub use types::{Position, PositionSide, Signal, StrategyConfig, StrategyState};

// Re-export commonly used indicator types for convenience
pub use velora_ta::{
    CircularBuffer, Indicator, IndicatorError, IndicatorResult, MultiIndicator, SingleIndicator,
    EMA, RSI, SMA,
};
