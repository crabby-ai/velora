//! # Velora
//!
//! High-Frequency Trading platform with data ingestion, exchange integration, and strategy execution.
//!
//! ## Overview
//!
//! Velora is a modular HFT platform built in Rust that provides:
//! - Market data ingestion and storage
//! - Exchange connectivity
//! - Strategy framework
//! - Backtesting engine
//! - Live trading engine
//! - Risk management
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use velora::prelude::*;
//!
//! // Example code will be added during implementation
//! ```

#![warn(missing_docs)]

// Re-export core types (always available)
pub use velora_core::*;

// Re-export optional modules based on features
#[cfg(feature = "data")]
pub use velora_data as data;

#[cfg(feature = "exchange")]
pub use velora_exchange as exchange;

#[cfg(feature = "strategy")]
pub use velora_strategy as strategy;

#[cfg(feature = "backtest")]
pub use velora_backtest as backtest;

#[cfg(feature = "engine")]
pub use velora_engine as engine;

#[cfg(feature = "risk")]
pub use velora_risk as risk;

/// Prelude module for convenient imports
pub mod prelude {
    pub use velora_core::*;

    #[cfg(feature = "data")]
    #[allow(unused_imports)]
    pub use velora_data::*;

    #[cfg(feature = "exchange")]
    #[allow(unused_imports)]
    pub use velora_exchange::*;

    #[cfg(feature = "strategy")]
    #[allow(unused_imports)]
    pub use velora_strategy::*;

    #[cfg(feature = "backtest")]
    #[allow(unused_imports)]
    pub use velora_backtest::*;

    #[cfg(feature = "engine")]
    #[allow(unused_imports)]
    pub use velora_engine::*;

    #[cfg(feature = "risk")]
    #[allow(unused_imports)]
    pub use velora_risk::*;
}
