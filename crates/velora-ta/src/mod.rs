//! Technical indicators for trading strategies.
//!
//! This module provides a comprehensive library of technical indicators
//! organized by category:
//!
//! - **Trend**: SMA, EMA, WMA, DEMA, TEMA, HMA
//! - **Momentum**: RSI, MACD, Stochastic, ROC, CCI
//! - **Volatility**: ATR, Bollinger Bands, Keltner Channels, StdDev
//! - **Volume**: OBV, VWAP, ADL, MFI, CMF
//! - **Composite**: Indicator chains, crossovers, threshold detection
//!
//! All indicators support both streaming (real-time) and batch (historical) modes.

pub mod errors;
pub mod traits;
pub mod types;
pub mod utils;

// Indicator implementations
pub mod trend;

// Re-export core types and traits
pub use errors::{IndicatorError, IndicatorResult};
pub use traits::{Indicator, MultiIndicator, OhlcvIndicator, SingleIndicator, VolumeIndicator};
pub use types::{IndicatorValue, MultiIndicatorValue, PriceType};
pub use utils::CircularBuffer;

// Re-export trend indicators for convenience
pub use trend::{EMA, SMA};
