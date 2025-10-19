//! Williams %R
//!
//! Williams %R is a momentum indicator similar to the Stochastic Oscillator,
//! but inverted and scaled from 0 to -100.
//!
//! Formula:
//! Williams %R = ((Highest High - Close) / (Highest High - Lowest Low)) * -100
//!
//! Values range from 0 to -100. Above -20 is overbought, below -80 is oversold.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    types::OhlcBar,
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Williams %R indicator.
///
/// Momentum indicator that measures overbought/oversold levels.
/// Values near 0 indicate overbought, values near -100 indicate oversold.
///
/// # Examples
///
/// ```
/// use velora_ta::{WilliamsR, SingleIndicator};
/// use velora_ta::types::OhlcBar;
/// use chrono::Utc;
///
/// let mut williams = WilliamsR::new(14).unwrap();
/// let timestamp = Utc::now();
///
/// let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
/// if let Some(value) = williams.update_ohlc(&bar, timestamp).unwrap() {
///     if value > -20.0 {
///         println!("Overbought: {:.2}", value);
///     } else if value < -80.0 {
///         println!("Oversold: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct WilliamsR {
    period: usize,
    high_buffer: CircularBuffer<f64>,
    low_buffer: CircularBuffer<f64>,
    close_buffer: CircularBuffer<f64>,
    name: String,
}

impl WilliamsR {
    /// Creates a new Williams %R indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Lookback period (typically 14, must be > 0)
    ///
    /// # Errors
    ///
    /// Returns an error if period is 0.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(WilliamsR {
            period,
            high_buffer: CircularBuffer::new(period),
            low_buffer: CircularBuffer::new(period),
            close_buffer: CircularBuffer::new(period),
            name: format!("Williams%R({period})"),
        })
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

        self.high_buffer.push(bar.high);
        self.low_buffer.push(bar.low);
        self.close_buffer.push(bar.close);

        Ok(self.calculate_williams_r())
    }

    /// Calculate Williams %R value.
    fn calculate_williams_r(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let highest_high = self.high_buffer.max()?;
        let lowest_low = self.low_buffer.min()?;
        let current_close = self.close_buffer.last()?;

        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Some(-50.0); // Neutral if no price movement
        }

        let williams_r = ((highest_high - current_close) / range) * -100.0;
        Some(williams_r.clamp(-100.0, 0.0))
    }
}

impl Indicator for WilliamsR {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.high_buffer.is_full() && self.low_buffer.is_full() && self.close_buffer.is_full()
    }

    fn reset(&mut self) {
        self.high_buffer.clear();
        self.low_buffer.clear();
        self.close_buffer.clear();
    }
}

impl SingleIndicator for WilliamsR {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        // Williams %R needs OHLC data, not just close price
        Err(IndicatorError::NotInitialized(
            "Williams %R requires OHLC data. Use update_ohlc() instead.".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        self.calculate_williams_r()
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        // Williams %R needs OHLC data, not just close prices
        Err(IndicatorError::NotInitialized(
            "Williams %R requires OHLC data. Use calculate_ohlc() instead.".to_string(),
        ))
    }
}

impl WilliamsR {
    /// Calculate Williams %R values for historical OHLC data (batch mode).
    ///
    /// # Arguments
    ///
    /// * `bars` - Historical OHLC data
    ///
    /// # Returns
    ///
    /// Vector of indicator values. Early values (during warmup) will be `None`.
    pub fn calculate_ohlc(&self, bars: &[OhlcBar]) -> IndicatorResult<Vec<Option<f64>>> {
        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut williams = Self::new(self.period)?;
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for bar in bars {
            result.push(williams.update_ohlc(bar, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::{MultiIndicator, SingleIndicator};

    #[test]
    fn test_williams_r_creation() {
        let williams = WilliamsR::new(14).unwrap();
        assert_eq!(williams.warmup_period(), 14);
        assert!(!williams.is_ready());
        assert_eq!(williams.name(), "Williams%R(14)");
    }

    #[test]
    fn test_williams_r_invalid_period() {
        assert!(WilliamsR::new(0).is_err());
    }

    #[test]
    fn test_williams_r_overbought() {
        let mut williams = WilliamsR::new(5).unwrap();
        let timestamp = Utc::now();

        // Create bars where close is at the high
        for i in 0..10 {
            let price = 100.0 + i as f64 * 5.0;
            let bar = OhlcBar::new(price, price + 2.0, price - 1.0, price + 2.0);
            williams.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(value) = williams.current() {
            // When close is at high, Williams %R should be near 0 (overbought)
            assert!(
                value > -20.0,
                "Williams %R should be overbought, got {value}"
            );
        }
    }

    #[test]
    fn test_williams_r_oversold() {
        let mut williams = WilliamsR::new(5).unwrap();
        let timestamp = Utc::now();

        // Create bars where close is at the low
        for i in 0..10 {
            let price = 100.0 - i as f64 * 5.0;
            let bar = OhlcBar::new(price, price + 1.0, price - 2.0, price - 2.0);
            williams.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(value) = williams.current() {
            // When close is at low, Williams %R should be near -100 (oversold)
            assert!(value < -80.0, "Williams %R should be oversold, got {value}");
        }
    }

    #[test]
    fn test_williams_r_vs_stochastic() {
        use crate::momentum::Stochastic;

        let mut williams = WilliamsR::new(5).unwrap();
        let mut stoch = Stochastic::new(5, 1).unwrap(); // %D period = 1 to match %K
        let timestamp = Utc::now();

        let bars: Vec<OhlcBar> = (1..=10)
            .map(|i| {
                let price = i as f64 * 10.0;
                OhlcBar::new(price, price + 5.0, price - 5.0, price + 2.0)
            })
            .collect();

        for bar in &bars {
            williams.update_ohlc(bar, timestamp).unwrap();
            stoch.update_ohlc(bar, timestamp).unwrap();
        }

        if let (Some(williams_val), Some(stoch_vals)) = (williams.current(), stoch.current()) {
            let stoch_k = stoch_vals[0];

            // Williams %R = -100 + Stochastic %K (approximately)
            // Or: Williams %R + 100 ≈ Stochastic %K
            let converted = williams_val + 100.0;
            assert!(
                (converted - stoch_k).abs() < 0.1,
                "Williams %R ({williams_val}) + 100 should ≈ Stochastic %K ({stoch_k})"
            );
        }
    }

    #[test]
    fn test_williams_r_batch_calculation() {
        let williams = WilliamsR::new(5).unwrap();

        let bars: Vec<OhlcBar> = (1..=10)
            .map(|i| {
                let price = i as f64 * 10.0;
                OhlcBar::new(price, price + 5.0, price - 5.0, price + 2.0)
            })
            .collect();

        let values = williams.calculate_ohlc(&bars).unwrap();
        assert_eq!(values.len(), 10);

        // First 4 should be None
        for i in 0..4 {
            assert_eq!(values[i], None);
        }

        // Should have values after warmup
        assert!(values[4].is_some());
    }

    #[test]
    fn test_williams_r_reset() {
        let mut williams = WilliamsR::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            let bar = OhlcBar::new(i as f64, i as f64 + 1.0, i as f64 - 1.0, i as f64);
            williams.update_ohlc(&bar, timestamp).unwrap();
        }

        assert!(williams.is_ready());

        williams.reset();
        assert!(!williams.is_ready());
        assert_eq!(williams.current(), None);
    }
}
