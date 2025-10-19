//! Common types used by technical indicators.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Value returned by a single-value indicator at a specific point in time.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct IndicatorValue {
    /// Timestamp of this indicator value
    pub timestamp: DateTime<Utc>,
    /// The indicator value
    pub value: f64,
}

impl IndicatorValue {
    /// Create a new indicator value.
    pub fn new(timestamp: DateTime<Utc>, value: f64) -> Self {
        Self { timestamp, value }
    }
}

/// Multi-value indicator output (e.g., Bollinger Bands returns upper/middle/lower).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MultiIndicatorValue {
    /// Timestamp of this indicator value
    pub timestamp: DateTime<Utc>,
    /// The indicator values (order depends on specific indicator)
    pub values: Vec<f64>,
}

impl MultiIndicatorValue {
    /// Create a new multi-indicator value.
    pub fn new(timestamp: DateTime<Utc>, values: Vec<f64>) -> Self {
        Self { timestamp, values }
    }

    /// Get a specific value by index.
    pub fn get(&self, index: usize) -> Option<f64> {
        self.values.get(index).copied()
    }

    /// Get the number of values.
    pub fn len(&self) -> usize {
        self.values.len()
    }

    /// Check if there are no values.
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

impl From<Vec<f64>> for MultiIndicatorValue {
    fn from(values: Vec<f64>) -> Self {
        Self {
            timestamp: Utc::now(),
            values,
        }
    }
}

/// Price type to extract from OHLCV data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum PriceType {
    /// Open price
    Open,
    /// High price
    High,
    /// Low price
    Low,
    /// Close price
    #[default]
    Close,
    /// Typical price: (High + Low + Close) / 3
    Typical,
    /// Weighted close: (High + Low + Close + Close) / 4
    Weighted,
    /// Average price: (High + Low) / 2
    Average,
    /// Median price: (High + Low) / 2 (same as Average)
    Median,
}

impl PriceType {
    /// Extract price from OHLC values.
    ///
    /// # Arguments
    ///
    /// * `open` - Open price
    /// * `high` - High price
    /// * `low` - Low price
    /// * `close` - Close price
    pub fn extract(&self, open: f64, high: f64, low: f64, close: f64) -> f64 {
        match self {
            PriceType::Open => open,
            PriceType::High => high,
            PriceType::Low => low,
            PriceType::Close => close,
            PriceType::Typical => (high + low + close) / 3.0,
            PriceType::Weighted => (high + low + close + close) / 4.0,
            PriceType::Average | PriceType::Median => (high + low) / 2.0,
        }
    }
}

/// Simple OHLC bar for indicators that need high/low data.
///
/// This is a lightweight structure used internally by indicators like
/// Stochastic, Williams %R, and ATR. It's separate from velora-core's
/// Candle type to keep this library standalone.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct OhlcBar {
    /// Open price
    pub open: f64,
    /// High price
    pub high: f64,
    /// Low price
    pub low: f64,
    /// Close price
    pub close: f64,
}

impl OhlcBar {
    /// Create a new OHLC bar.
    pub fn new(open: f64, high: f64, low: f64, close: f64) -> Self {
        Self {
            open,
            high,
            low,
            close,
        }
    }

    /// Extract a specific price type from this bar.
    pub fn price(&self, price_type: PriceType) -> f64 {
        price_type.extract(self.open, self.high, self.low, self.close)
    }

    /// Get typical price: (High + Low + Close) / 3
    pub fn typical_price(&self) -> f64 {
        (self.high + self.low + self.close) / 3.0
    }

    /// Get range: High - Low
    pub fn range(&self) -> f64 {
        self.high - self.low
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_price_type_extraction() {
        let open = 100.0;
        let high = 105.0;
        let low = 95.0;
        let close = 102.0;

        assert_eq!(PriceType::Open.extract(open, high, low, close), 100.0);
        assert_eq!(PriceType::High.extract(open, high, low, close), 105.0);
        assert_eq!(PriceType::Low.extract(open, high, low, close), 95.0);
        assert_eq!(PriceType::Close.extract(open, high, low, close), 102.0);

        // Typical: (105 + 95 + 102) / 3 = 100.666...
        assert!((PriceType::Typical.extract(open, high, low, close) - 100.666666).abs() < 0.001);

        // Weighted: (105 + 95 + 102 + 102) / 4 = 101.0
        assert_eq!(PriceType::Weighted.extract(open, high, low, close), 101.0);

        // Average/Median: (105 + 95) / 2 = 100.0
        assert_eq!(PriceType::Average.extract(open, high, low, close), 100.0);
        assert_eq!(PriceType::Median.extract(open, high, low, close), 100.0);
    }

    #[test]
    fn test_indicator_value() {
        let timestamp = Utc::now();
        let value = IndicatorValue::new(timestamp, 42.0);

        assert_eq!(value.timestamp, timestamp);
        assert_eq!(value.value, 42.0);
    }

    #[test]
    fn test_multi_indicator_value() {
        let timestamp = Utc::now();
        let values = vec![1.0, 2.0, 3.0];
        let multi = MultiIndicatorValue::new(timestamp, values.clone());

        assert_eq!(multi.timestamp, timestamp);
        assert_eq!(multi.len(), 3);
        assert!(!multi.is_empty());
        assert_eq!(multi.get(0), Some(1.0));
        assert_eq!(multi.get(1), Some(2.0));
        assert_eq!(multi.get(2), Some(3.0));
        assert_eq!(multi.get(3), None);
    }
}
