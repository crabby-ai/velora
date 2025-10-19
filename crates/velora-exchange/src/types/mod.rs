//! Type definitions for exchange integrations.
//!
//! This module contains all the data types used across different exchanges,
//! designed to be instrument-agnostic where possible.

pub mod account;
pub mod candle;
pub mod common;
pub mod error;
pub mod instrument;
pub mod market;
pub mod order;
pub mod orderbook;
pub mod stream;
pub mod trade;

// Re-export commonly used types
pub use account::*;
pub use candle::*;
pub use common::*;
pub use error::*;
pub use instrument::*;
pub use market::*;
pub use order::*;
pub use orderbook::*;
pub use stream::*;
pub use trade::*;
