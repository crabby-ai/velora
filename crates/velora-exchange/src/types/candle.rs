//! Candle/Kline types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export Interval from velora-core
pub use velora_core::Interval;

/// Candlestick/Kline data
///
/// IMPORTANT: This is instrument-agnostic!
/// Same structure for Spot, Perpetuals, Futures, Options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Symbol
    pub symbol: Symbol,

    /// Candle interval
    pub interval: Interval,

    /// Open price
    pub open: Price,

    /// High price
    pub high: Price,

    /// Low price
    pub low: Price,

    /// Close price
    pub close: Price,

    /// Volume
    pub volume: Decimal,

    /// Candle open time
    pub open_time: DateTime<Utc>,

    /// Candle close time
    pub close_time: DateTime<Utc>,

    /// Number of trades in this candle
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trade_count: Option<u64>,

    /// Quote asset volume (volume in quote currency)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quote_volume: Option<Decimal>,
}

impl Candle {
    /// Check if candle is complete (closed)
    pub fn is_closed(&self) -> bool {
        Utc::now() >= self.close_time
    }

    /// Get the price range (high - low)
    pub fn range(&self) -> Decimal {
        Decimal::try_from(self.high.0 - self.low.0).unwrap_or(Decimal::ZERO)
    }

    /// Get the body size (close - open)
    pub fn body(&self) -> Decimal {
        Decimal::try_from((self.close.0 - self.open.0).abs()).unwrap_or(Decimal::ZERO)
    }

    /// Check if candle is bullish (close > open)
    pub fn is_bullish(&self) -> bool {
        self.close.0 > self.open.0
    }

    /// Check if candle is bearish (close < open)
    pub fn is_bearish(&self) -> bool {
        self.close.0 < self.open.0
    }
}
