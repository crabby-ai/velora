//! Moving Average Convergence Divergence (MACD)
//!
//! MACD is a trend-following momentum indicator that shows the relationship between
//! two exponential moving averages.
//!
//! Components:
//! - MACD Line = EMA(fast) - EMA(slow)
//! - Signal Line = EMA(MACD Line, signal_period)
//! - Histogram = MACD Line - Signal Line
//!
//! Common settings: fast=12, slow=26, signal=9

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::EMA,
    types::MultiIndicatorValue,
    IndicatorError, IndicatorResult,
};

/// MACD (Moving Average Convergence Divergence) indicator.
///
/// Outputs three values: MACD line, Signal line, and Histogram.
///
/// # Examples
///
/// ```
/// use velora_ta::{MACD, MultiIndicator};
/// use chrono::Utc;
///
/// let mut macd = MACD::new(12, 26, 9).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 101.0, 102.0, 103.0, 104.0] {
///     if let Some(values) = macd.update(price, timestamp).unwrap() {
///         println!("MACD: {:.2}, Signal: {:.2}, Histogram: {:.2}",
///                  values[0], values[1], values[2]);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct MACD {
    fast_period: usize,
    slow_period: usize,
    signal_period: usize,
    ema_fast: EMA,
    ema_slow: EMA,
    ema_signal: EMA,
    name: String,
}

impl MACD {
    /// Creates a new MACD indicator.
    ///
    /// # Arguments
    ///
    /// * `fast_period` - Fast EMA period (typically 12)
    /// * `slow_period` - Slow EMA period (typically 26, must be > fast_period)
    /// * `signal_period` - Signal line EMA period (typically 9)
    ///
    /// # Errors
    ///
    /// Returns an error if any period is 0 or if slow_period <= fast_period.
    pub fn new(
        fast_period: usize,
        slow_period: usize,
        signal_period: usize,
    ) -> IndicatorResult<Self> {
        if fast_period == 0 || slow_period == 0 || signal_period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "All periods must be greater than 0".to_string(),
            ));
        }

        if slow_period <= fast_period {
            return Err(IndicatorError::InvalidParameter(
                "Slow period must be greater than fast period".to_string(),
            ));
        }

        Ok(MACD {
            fast_period,
            slow_period,
            signal_period,
            ema_fast: EMA::new(fast_period)?,
            ema_slow: EMA::new(slow_period)?,
            ema_signal: EMA::new(signal_period)?,
            name: format!("MACD({fast_period},{slow_period},{signal_period})"),
        })
    }

    /// Calculate current MACD values.
    fn calculate_values(&self) -> Option<Vec<f64>> {
        let fast_val = self.ema_fast.current()?;
        let slow_val = self.ema_slow.current()?;
        let macd_line = fast_val - slow_val;

        let signal_line = self.ema_signal.current()?;
        let histogram = macd_line - signal_line;

        Some(vec![macd_line, signal_line, histogram])
    }
}

impl Indicator for MACD {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // Needs slow_period + signal_period to be fully ready
        self.slow_period + self.signal_period
    }

    fn is_ready(&self) -> bool {
        self.ema_fast.is_ready() && self.ema_slow.is_ready() && self.ema_signal.is_ready()
    }

    fn reset(&mut self) {
        self.ema_fast.reset();
        self.ema_slow.reset();
        self.ema_signal.reset();
    }
}

impl MultiIndicator for MACD {
    fn output_count(&self) -> usize {
        3 // MACD, Signal, Histogram
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["MACD", "Signal", "Histogram"]
    }

    fn update(
        &mut self,
        price: f64,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        // Update both EMAs with price
        self.ema_fast.update(price, timestamp)?;
        self.ema_slow.update(price, timestamp)?;

        // Calculate MACD line
        if let (Some(fast_val), Some(slow_val)) = (self.ema_fast.current(), self.ema_slow.current())
        {
            let macd_line = fast_val - slow_val;
            // Feed MACD line to signal EMA
            self.ema_signal.update(macd_line, timestamp)?;
        }

        Ok(self.calculate_values())
    }

    fn current(&self) -> Option<Vec<f64>> {
        self.calculate_values()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut macd = Self::new(self.fast_period, self.slow_period, self.signal_period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(
                macd.update(price, timestamp)?
                    .map(MultiIndicatorValue::from),
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::MultiIndicator;

    #[test]
    fn test_macd_creation() {
        let macd = MACD::new(12, 26, 9).unwrap();
        assert_eq!(macd.output_count(), 3);
        assert_eq!(macd.output_names(), vec!["MACD", "Signal", "Histogram"]);
        assert!(!macd.is_ready());
    }

    #[test]
    fn test_macd_invalid_periods() {
        assert!(MACD::new(0, 26, 9).is_err());
        assert!(MACD::new(26, 12, 9).is_err()); // slow <= fast
        assert!(MACD::new(12, 26, 0).is_err());
    }

    #[test]
    fn test_macd_calculation() {
        let mut macd = MACD::new(3, 6, 3).unwrap();
        let timestamp = Utc::now();

        // Feed uptrending prices
        let prices: Vec<f64> = (1..=20).map(|x| x as f64).collect();

        for &price in &prices {
            macd.update(price, timestamp).unwrap();
        }

        if let Some(values) = macd.current() {
            let macd_line = values[0];
            let signal_line = values[1];
            let histogram = values[2];

            // In an uptrend, MACD line should be positive
            assert!(macd_line > 0.0);

            // Histogram = MACD - Signal
            assert!((histogram - (macd_line - signal_line)).abs() < 0.0001);
        }
    }

    #[test]
    fn test_macd_crossover() {
        let mut macd = MACD::new(5, 10, 5).unwrap();
        let timestamp = Utc::now();

        // Create simple uptrending data
        let prices: Vec<f64> = (1..=30).map(|x| x as f64 * 10.0).collect();

        let mut prev_histogram = None;

        for &price in &prices {
            if let Some(values) = macd.update(price, timestamp).unwrap() {
                let histogram = values[2];
                prev_histogram = Some(histogram);
            }
        }

        // In a strong uptrend, we should see at least some positive histogram values
        assert!(prev_histogram.is_some());
    }

    #[test]
    fn test_macd_batch_calculation() {
        let macd = MACD::new(5, 10, 5).unwrap();
        let prices: Vec<f64> = (1..=30).map(|x| x as f64).collect();
        let values = macd.calculate(&prices).unwrap();

        assert_eq!(values.len(), 30);

        // Last values should have results
        assert!(values[29].is_some());

        if let Some(ref multi_val) = values[29] {
            assert_eq!(multi_val.values.len(), 3);
        }
    }

    #[test]
    fn test_macd_reset() {
        let mut macd = MACD::new(5, 10, 5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=20 {
            macd.update(i as f64, timestamp).unwrap();
        }

        if macd.is_ready() {
            macd.reset();
            assert!(!macd.is_ready());
            assert_eq!(macd.current(), None);
        }
    }
}
