//! Stochastic Oscillator
//!
//! The Stochastic Oscillator compares the closing price to its price range over a given period.
//! It outputs two lines: %K (fast) and %D (slow, which is SMA of %K).
//!
//! Formula:
//! %K = ((Close - Lowest Low) / (Highest High - Lowest Low)) * 100
//! %D = SMA(%K, smooth_period)
//!
//! Values range from 0 to 100. Above 80 is overbought, below 20 is oversold.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::SMA,
    types::{MultiIndicatorValue, OhlcBar},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Stochastic Oscillator indicator.
///
/// Outputs two values: %K (fast line) and %D (slow line, SMA of %K).
/// Used to identify overbought/oversold conditions.
///
/// # Examples
///
/// ```
/// use velora_ta::{Stochastic, MultiIndicator};
/// use velora_ta::types::OhlcBar;
/// use chrono::Utc;
///
/// let mut stoch = Stochastic::new(14, 3).unwrap();
/// let timestamp = Utc::now();
///
/// let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
/// if let Some(values) = stoch.update_ohlc(&bar, timestamp).unwrap() {
///     println!("%K: {:.2}, %D: {:.2}", values[0], values[1]);
///     if values[0] > 80.0 {
///         println!("Overbought!");
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Stochastic {
    period: usize,
    smooth_period: usize,
    high_buffer: CircularBuffer<f64>,
    low_buffer: CircularBuffer<f64>,
    close_buffer: CircularBuffer<f64>,
    sma_d: SMA,
    name: String,
}

impl Stochastic {
    /// Creates a new Stochastic Oscillator.
    ///
    /// # Arguments
    ///
    /// * `period` - Lookback period for %K calculation (typically 14)
    /// * `smooth_period` - Period for %D SMA (typically 3)
    ///
    /// # Errors
    ///
    /// Returns an error if any period is 0.
    pub fn new(period: usize, smooth_period: usize) -> IndicatorResult<Self> {
        if period == 0 || smooth_period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "All periods must be greater than 0".to_string(),
            ));
        }

        Ok(Stochastic {
            period,
            smooth_period,
            high_buffer: CircularBuffer::new(period),
            low_buffer: CircularBuffer::new(period),
            close_buffer: CircularBuffer::new(period),
            sma_d: SMA::new(smooth_period)?,
            name: format!("Stochastic({period},{smooth_period})"),
        })
    }

    /// Update the indicator with OHLC data.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
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

        // Calculate %K
        if let Some(k_value) = self.calculate_k() {
            // Feed %K to %D SMA
            self.sma_d.update(k_value, timestamp)?;
        }

        Ok(self.calculate_values())
    }

    /// Calculate %K value.
    fn calculate_k(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let highest_high = self.high_buffer.max()?;
        let lowest_low = self.low_buffer.min()?;
        let current_close = self.close_buffer.last()?;

        let range = highest_high - lowest_low;
        if range == 0.0 {
            return Some(50.0); // Neutral if no price movement
        }

        let k = ((current_close - lowest_low) / range) * 100.0;
        Some(k.clamp(0.0, 100.0))
    }

    /// Calculate current Stochastic values.
    fn calculate_values(&self) -> Option<Vec<f64>> {
        let k_value = self.calculate_k()?;
        let d_value = self.sma_d.current()?;

        Some(vec![k_value, d_value])
    }
}

impl Indicator for Stochastic {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + self.smooth_period - 1
    }

    fn is_ready(&self) -> bool {
        self.high_buffer.is_full() && self.low_buffer.is_full() && self.close_buffer.is_full()
    }

    fn reset(&mut self) {
        self.high_buffer.clear();
        self.low_buffer.clear();
        self.close_buffer.clear();
        self.sma_d.reset();
    }
}

impl MultiIndicator for Stochastic {
    fn output_count(&self) -> usize {
        2 // %K and %D
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["%K", "%D"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        // Stochastic needs OHLC data, not just close price
        // This method is here to satisfy the trait but shouldn't be used
        Err(IndicatorError::NotInitialized(
            "Stochastic requires OHLC data. Use update_ohlc() instead.".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        self.calculate_values()
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        // Stochastic needs OHLC data, not just close prices
        Err(IndicatorError::NotInitialized(
            "Stochastic requires OHLC data. Use calculate_ohlc() instead.".to_string(),
        ))
    }
}

impl Stochastic {
    /// Calculate Stochastic values for historical OHLC data (batch mode).
    ///
    /// # Arguments
    ///
    /// * `bars` - Historical OHLC data
    ///
    /// # Returns
    ///
    /// Vector of multi-indicator values. Early values (during warmup) will be `None`.
    pub fn calculate_ohlc(
        &self,
        bars: &[OhlcBar],
    ) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut stoch = Self::new(self.period, self.smooth_period)?;
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for bar in bars {
            result.push(
                stoch
                    .update_ohlc(bar, timestamp)?
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
    fn test_stochastic_creation() {
        let stoch = Stochastic::new(14, 3).unwrap();
        assert_eq!(stoch.output_count(), 2);
        assert_eq!(stoch.output_names(), vec!["%K", "%D"]);
        assert!(!stoch.is_ready());
    }

    #[test]
    fn test_stochastic_invalid_periods() {
        assert!(Stochastic::new(0, 3).is_err());
        assert!(Stochastic::new(14, 0).is_err());
    }

    #[test]
    fn test_stochastic_overbought() {
        let mut stoch = Stochastic::new(5, 3).unwrap();
        let timestamp = Utc::now();

        // Create bars where close is at the high (strong uptrend)
        for i in 0..10 {
            let price = 100.0 + i as f64 * 5.0;
            let bar = OhlcBar::new(price, price + 2.0, price - 1.0, price + 2.0);
            stoch.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(values) = stoch.current() {
            let k = values[0];
            // When close is consistently at high, %K should be near 100
            assert!(k > 80.0, "%K should be overbought, got {k}");
        }
    }

    #[test]
    fn test_stochastic_oversold() {
        let mut stoch = Stochastic::new(5, 3).unwrap();
        let timestamp = Utc::now();

        // Create bars where close is at the low (strong downtrend)
        for i in 0..10 {
            let price = 100.0 - i as f64 * 5.0;
            let bar = OhlcBar::new(price, price + 1.0, price - 2.0, price - 2.0);
            stoch.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(values) = stoch.current() {
            let k = values[0];
            // When close is consistently at low, %K should be near 0
            assert!(k < 20.0, "%K should be oversold, got {k}");
        }
    }

    #[test]
    fn test_stochastic_batch_calculation() {
        let stoch = Stochastic::new(5, 3).unwrap();

        let bars: Vec<OhlcBar> = (1..=15)
            .map(|i| {
                let price = i as f64 * 10.0;
                OhlcBar::new(price, price + 5.0, price - 5.0, price + 2.0)
            })
            .collect();

        let values = stoch.calculate_ohlc(&bars).unwrap();
        assert_eq!(values.len(), 15);

        // Last value should have results
        assert!(values[14].is_some());
    }

    #[test]
    fn test_stochastic_reset() {
        let mut stoch = Stochastic::new(5, 3).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            let bar = OhlcBar::new(i as f64, i as f64 + 1.0, i as f64 - 1.0, i as f64);
            stoch.update_ohlc(&bar, timestamp).unwrap();
        }

        stoch.reset();
        assert!(!stoch.is_ready());
        assert_eq!(stoch.current(), None);
    }
}
