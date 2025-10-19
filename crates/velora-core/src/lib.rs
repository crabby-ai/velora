//! # velora-core
//!
//! Core types, errors, and utilities for the Velora HFT platform.
//!
//! This crate provides the foundational types used throughout the Velora ecosystem,
//! including trading types (orders, trades, positions), market data structures
//! (ticks, candles, order books), error handling, and configuration management.
//!
//! ## Features
//!
//! - **Type-safe numeric types**: Using `OrderedFloat` for prices and volumes
//! - **Comprehensive trading types**: Orders, trades, positions, balances
//! - **Market data structures**: Ticks, candles, order books
//! - **Error handling**: Unified error type with conversions
//! - **Configuration management**: TOML-based configuration
//!
//! ## Example
//!
//! ```
//! use velora_core::*;
//!
//! // Create a new limit order
//! let order = Order::new_limit(
//!     Symbol::new("BTC/USD"),
//!     Side::Buy,
//!     50000.0.into(),
//!     0.1.into(),
//! );
//!
//! assert_eq!(order.order_type, OrderType::Limit);
//! assert!(order.is_active());
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

pub mod config;
pub mod errors;
pub mod types;

// Re-export commonly used types for convenience
pub use config::*;
pub use errors::{Result, VeloraError};
pub use types::*;
