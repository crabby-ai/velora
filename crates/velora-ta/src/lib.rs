//! # velora-ta
//!
//! Technical Analysis indicators library for algorithmic trading.
//!
//! This is a standalone technical analysis library that can be used independently
//! or as part of the Velora HFT platform. It provides a comprehensive collection
//! of technical indicators organized by category.
//!
//! ## Features
//!
//! - **Trend Indicators**: SMA, EMA, WMA, DEMA, TEMA, HMA
//! - **Momentum Indicators**: RSI, MACD, Stochastic, ROC, CCI
//! - **Volatility Indicators**: ATR, Bollinger Bands, Keltner Channels, StdDev
//! - **Volume Indicators**: OBV, VWAP, ADL, MFI, CMF
//! - **Composite Tools**: Indicator chains, crossovers, threshold detection
//!
//! ## Design Philosophy
//!
//! - **Streaming-first**: Process data one point at a time for real-time trading
//! - **Batch support**: Calculate historical indicator values efficiently
//! - **Zero-copy**: Efficient with slices and references where possible
//! - **Type-safe**: Compile-time guarantees via Rust's type system
//! - **Well-tested**: Comprehensive test coverage with edge cases
//!
//! ## Quick Start
//!
//! ```ignore
//! use velora_ta::{SMA, EMA, RSI, SingleIndicator};
//! use chrono::Utc;
//!
//! // Create indicators
//! let mut sma = SMA::new(20)?;
//! let mut ema = EMA::new(10)?;
//! let mut rsi = RSI::new(14)?;
//!
//! // Stream mode: process prices one at a time
//! for price in prices {
//!     let timestamp = Utc::now();
//!
//!     if let Some(sma_val) = sma.update(price, timestamp)? {
//!         println!("SMA(20): {:.2}", sma_val);
//!     }
//!
//!     if let Some(rsi_val) = rsi.update(price, timestamp)? {
//!         if rsi_val > 70.0 {
//!             println!("Overbought!");
//!         } else if rsi_val < 30.0 {
//!             println!("Oversold!");
//!         }
//!     }
//! }
//!
//! // Or use batch mode for historical data
//! let sma = SMA::new(20)?;
//! let values = sma.calculate(&historical_prices)?;
//! ```
//!
//! ## Indicator Categories
//!
//! ### Trend Indicators
//!
//! Track the direction and strength of price trends:
//! - **SMA**: Simple Moving Average - equal-weighted average
//! - **EMA**: Exponential Moving Average - more weight on recent prices
//! - **WMA**: Weighted Moving Average - linear weights (planned)
//! - **HMA**: Hull Moving Average - very responsive (planned)
//!
//! ### Momentum Indicators
//!
//! Measure the speed and magnitude of price changes:
//! - **RSI**: Relative Strength Index - overbought/oversold detector
//! - **MACD**: Moving Average Convergence Divergence - trend + momentum (in progress)
//! - **Stochastic**: Oscillator comparing close to high-low range (planned)
//!
//! ### Volatility Indicators (Planned)
//!
//! Measure price volatility and potential ranges:
//! - **ATR**: Average True Range - for stop-loss placement
//! - **Bollinger Bands**: Volatility bands around price
//! - **Keltner Channels**: ATR-based channels
//!
//! ### Volume Indicators (Planned)
//!
//! Analyze volume patterns:
//! - **OBV**: On-Balance Volume - cumulative volume
//! - **VWAP**: Volume-Weighted Average Price - intraday benchmark
//! - **MFI**: Money Flow Index - RSI with volume
//!
//! ## Usage Patterns
//!
//! ### Streaming Mode (Real-time)
//!
//! ```ignore
//! let mut indicator = SMA::new(20)?;
//!
//! // Process live data
//! loop {
//!     let price = get_latest_price();
//!     let timestamp = Utc::now();
//!
//!     if let Some(value) = indicator.update(price, timestamp)? {
//!         make_trading_decision(value);
//!     }
//! }
//! ```
//!
//! ### Batch Mode (Historical)
//!
//! ```ignore
//! let indicator = SMA::new(20)?;
//! let prices = load_historical_data();
//! let values = indicator.calculate(&prices)?;
//!
//! for (i, value) in values.iter().enumerate() {
//!     if let Some(v) = value {
//!         println!("Price: {}, SMA: {}", prices[i], v);
//!     }
//! }
//! ```
//!
//! ## Standalone Usage
//!
//! This library can be used completely independently of the Velora platform:
//!
//! ```toml
//! [dependencies]
//! velora-ta = "0.1"
//! ```

#![warn(missing_docs)]
#![warn(missing_debug_implementations)]
#![warn(rust_2018_idioms)]

pub mod errors;
pub mod traits;
pub mod types;
pub mod utils;

// Indicator implementations
pub mod momentum;
pub mod trend;
pub mod volatility;
pub mod volume;

// Advanced modules
pub mod patterns;
pub mod statistical;
pub mod williams;

// Re-export core types and traits
pub use errors::{IndicatorError, IndicatorResult};
pub use traits::{Indicator, MultiIndicator, SingleIndicator, VolumeIndicator};
pub use types::{IndicatorValue, MultiIndicatorValue, PriceType};
pub use utils::CircularBuffer;

// Re-export trend indicators for convenience
pub use trend::{
    Aroon, ParabolicSAR, SuperTrend, Vortex, ADX, DEMA, EMA, HMA, KAMA, SMA, SMMA, TEMA, VWMA, WMA,
};

// Re-export momentum indicators for convenience
pub use momentum::{Momentum, Stochastic, WilliamsR, CCI, MACD, ROC, RSI, TSI};

// Re-export volatility indicators for convenience
pub use volatility::{BollingerBands, DonchianChannels, KeltnerChannels, StdDev, TrueRange, ATR};

// Re-export volume indicators for convenience
pub use volume::{ForceIndex, AD, CMF, EMV, MFI, OBV, VWAP};

// Re-export Bill Williams indicators
pub use williams::{Alligator, AwesomeOscillator, Fractals};

// Re-export statistical indicators
pub use statistical::{Correlation, LinearRegression, ZScore};

// Re-export pattern detectors
pub use patterns::{
    BearishEngulfing, BullishEngulfing, Doji, Hammer, PatternDetector, PatternSignal, ShootingStar,
    ThreeBlackCrows, ThreeWhiteSoldiers,
};

// Re-export types
pub use types::OhlcBar;
