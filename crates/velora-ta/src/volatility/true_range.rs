//! True Range
//!
//! True Range is a measure of volatility that accounts for gaps in price.
//! It's the greatest of:
//! - Current High - Current Low
//! - abs(Current High - Previous Close)
//! - abs(Current Low - Previous Close)
//!
//! This is the foundation for ATR (Average True Range).

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    types::OhlcBar,
    IndicatorError, IndicatorResult,
};

/// True Range indicator.
///
/// Measures volatility accounting for price gaps.
/// Always returns a positive value.
#[derive(Debug, Clone)]
pub struct TrueRange {
    previous_close: Option<f64>,
    name: String,
}

impl TrueRange {
    /// Creates a new True Range indicator.
    pub fn new() -> Self {
        TrueRange {
            previous_close: None,
            name: "TrueRange".to_string(),
        }
    }

    /// Update the indicator with OHLC data.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if !bar.high.is_finite() || !bar.low.is_finite() || !bar.close.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "OHLC values must be finite numbers".to_string(),
            ));
        }

        if bar.high < bar.low {
            return Err(IndicatorError::InvalidInput(
                "High must be >= Low".to_string(),
            ));
        }

        let tr = if let Some(prev_close) = self.previous_close {
            // True Range = max of:
            // 1. High - Low
            // 2. abs(High - Previous Close)
            // 3. abs(Low - Previous Close)
            let high_low = bar.high - bar.low;
            let high_prev_close = (bar.high - prev_close).abs();
            let low_prev_close = (bar.low - prev_close).abs();

            high_low.max(high_prev_close).max(low_prev_close)
        } else {
            // First bar: just use high - low
            bar.high - bar.low
        };

        self.previous_close = Some(bar.close);
        Ok(Some(tr))
    }
}

impl Default for TrueRange {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for TrueRange {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        1 // Needs at least 1 bar
    }

    fn is_ready(&self) -> bool {
        self.previous_close.is_some()
    }

    fn reset(&mut self) {
        self.previous_close = None;
    }
}

impl SingleIndicator for TrueRange {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        Err(IndicatorError::NotInitialized(
            "True Range requires OHLC data. Use update_ohlc() instead.".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        // True Range doesn't store current value, only previous close
        None
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Err(IndicatorError::NotInitialized(
            "True Range requires OHLC data. Use calculate_ohlc() instead.".to_string(),
        ))
    }
}

impl TrueRange {
    /// Calculate True Range values for historical OHLC data (batch mode).
    pub fn calculate_ohlc(&self, bars: &[OhlcBar]) -> IndicatorResult<Vec<Option<f64>>> {
        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut tr = Self::new();
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for bar in bars {
            result.push(tr.update_ohlc(bar, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_range_creation() {
        let tr = TrueRange::new();
        assert_eq!(tr.name(), "TrueRange");
        assert!(!tr.is_ready());
    }

    #[test]
    fn test_true_range_first_bar() {
        let mut tr = TrueRange::new();
        let timestamp = Utc::now();

        let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
        let value = tr.update_ohlc(&bar, timestamp).unwrap().unwrap();

        // First bar: TR = High - Low = 105 - 95 = 10
        assert_eq!(value, 10.0);
    }

    #[test]
    fn test_true_range_with_gap_up() {
        let mut tr = TrueRange::new();
        let timestamp = Utc::now();

        // First bar
        let bar1 = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
        tr.update_ohlc(&bar1, timestamp).unwrap();

        // Second bar gaps up
        let bar2 = OhlcBar::new(110.0, 115.0, 108.0, 112.0);
        let value = tr.update_ohlc(&bar2, timestamp).unwrap().unwrap();

        // TR should be max of:
        // - High - Low = 115 - 108 = 7
        // - abs(High - Prev Close) = abs(115 - 102) = 13  <- Largest
        // - abs(Low - Prev Close) = abs(108 - 102) = 6
        assert_eq!(value, 13.0);
    }

    #[test]
    fn test_true_range_with_gap_down() {
        let mut tr = TrueRange::new();
        let timestamp = Utc::now();

        // First bar
        let bar1 = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
        tr.update_ohlc(&bar1, timestamp).unwrap();

        // Second bar gaps down
        let bar2 = OhlcBar::new(88.0, 92.0, 85.0, 90.0);
        let value = tr.update_ohlc(&bar2, timestamp).unwrap().unwrap();

        // TR should be max of:
        // - High - Low = 92 - 85 = 7
        // - abs(High - Prev Close) = abs(92 - 102) = 10
        // - abs(Low - Prev Close) = abs(85 - 102) = 17  <- Largest
        assert_eq!(value, 17.0);
    }

    #[test]
    fn test_true_range_batch() {
        let tr = TrueRange::new();

        let bars = vec![
            OhlcBar::new(100.0, 105.0, 95.0, 102.0),
            OhlcBar::new(102.0, 107.0, 100.0, 105.0),
            OhlcBar::new(105.0, 110.0, 103.0, 108.0),
        ];

        let values = tr.calculate_ohlc(&bars).unwrap();

        assert_eq!(values.len(), 3);
        assert_eq!(values[0].unwrap(), 10.0); // First bar: 105 - 95
                                              // All should have values
        assert!(values[1].is_some());
        assert!(values[2].is_some());
    }

    #[test]
    fn test_true_range_reset() {
        let mut tr = TrueRange::new();
        let timestamp = Utc::now();

        let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
        tr.update_ohlc(&bar, timestamp).unwrap();

        assert!(tr.is_ready());

        tr.reset();
        assert!(!tr.is_ready());
    }
}
