//! Commodity Channel Index (CCI)
//!
//! CCI measures the current price relative to an average price over a given period.
//! It can be used to identify overbought/oversold conditions and trend changes.
//!
//! Formula:
//! Typical Price = (High + Low + Close) / 3
//! CCI = (Typical Price - SMA(Typical Price)) / (0.015 * Mean Deviation)
//!
//! For price-only version (using close as typical price):
//! CCI = (Price - SMA(Price)) / (0.015 * Mean Deviation)
//!
//! Values above +100 indicate overbought, below -100 indicate oversold.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::SMA,
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Commodity Channel Index indicator.
///
/// Measures price deviation from the average. Values above +100 suggest
/// overbought conditions, below -100 suggest oversold.
///
/// # Examples
///
/// ```
/// use velora_ta::{CCI, SingleIndicator};
/// use chrono::Utc;
///
/// let mut cci = CCI::new(20).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 105.0, 110.0, 115.0, 120.0] {
///     if let Some(value) = cci.update(price, timestamp).unwrap() {
///         if value > 100.0 {
///             println!("Overbought: {:.2}", value);
///         } else if value < -100.0 {
///             println!("Oversold: {:.2}", value);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct CCI {
    period: usize,
    sma: SMA,
    buffer: CircularBuffer<f64>,
    constant: f64, // 0.015 constant
    name: String,
}

impl CCI {
    /// Creates a new CCI indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the calculation (typically 20, must be > 0)
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

        Ok(CCI {
            period,
            sma: SMA::new(period)?,
            buffer: CircularBuffer::new(period),
            constant: 0.015,
            name: format!("CCI({period})"),
        })
    }

    /// Calculate mean deviation.
    fn calculate_mean_deviation(&self, sma_value: f64) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let sum_abs_deviation: f64 = self
            .buffer
            .iter()
            .map(|&price| (price - sma_value).abs())
            .sum();

        Some(sum_abs_deviation / self.period as f64)
    }

    /// Calculate CCI from current buffer.
    fn calculate_cci(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let current_price = self.buffer.last()?;
        let sma_value = self.sma.current()?;
        let mean_deviation = self.calculate_mean_deviation(sma_value)?;

        if mean_deviation == 0.0 {
            return Some(0.0); // Avoid division by zero
        }

        let cci = (current_price - sma_value) / (self.constant * mean_deviation);
        Some(cci)
    }
}

impl Indicator for CCI {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full() && self.sma.is_ready()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.sma.reset();
    }
}

impl SingleIndicator for CCI {
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        self.buffer.push(price);
        self.sma.update(price, timestamp)?;

        Ok(self.calculate_cci())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_cci()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut cci = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(cci.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SingleIndicator;

    #[test]
    fn test_cci_creation() {
        let cci = CCI::new(20).unwrap();
        assert_eq!(cci.warmup_period(), 20);
        assert!(!cci.is_ready());
        assert_eq!(cci.name(), "CCI(20)");
    }

    #[test]
    fn test_cci_invalid_period() {
        assert!(CCI::new(0).is_err());
    }

    #[test]
    fn test_cci_at_average() {
        let mut cci = CCI::new(5).unwrap();
        let timestamp = Utc::now();

        // Feed constant prices
        for _ in 0..5 {
            cci.update(100.0, timestamp).unwrap();
        }

        // When price equals SMA and there's no deviation, CCI should be 0
        let value = cci.current().unwrap();
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_cci_above_average() {
        let mut cci = CCI::new(5).unwrap();
        let timestamp = Utc::now();

        // Prices: 100, 100, 100, 100, 100, then 120 (spike up)
        for _ in 0..5 {
            cci.update(100.0, timestamp).unwrap();
        }

        // Now add higher price
        cci.update(120.0, timestamp).unwrap();

        if let Some(value) = cci.current() {
            // CCI should be positive (price above average)
            assert!(value > 0.0);
        }
    }

    #[test]
    fn test_cci_overbought_oversold() {
        let mut cci = CCI::new(10).unwrap();
        let timestamp = Utc::now();

        // Create trending prices
        let prices: Vec<f64> = (1..=30).map(|x| x as f64 * 10.0).collect();

        for &price in &prices {
            cci.update(price, timestamp).unwrap();
        }

        // In a strong uptrend, CCI can go well above 100
        if let Some(value) = cci.current() {
            // Should be positive in uptrend
            assert!(value > 0.0);
        }
    }

    #[test]
    fn test_cci_batch_calculation() {
        let cci = CCI::new(5).unwrap();
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 112.0];
        let values = cci.calculate(&prices).unwrap();

        assert_eq!(values.len(), 7);

        // First 4 should be None
        for i in 0..4 {
            assert_eq!(values[i], None);
        }

        // Should have values after warmup
        assert!(values[4].is_some());
        assert!(values[5].is_some());
    }

    #[test]
    fn test_cci_reset() {
        let mut cci = CCI::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            cci.update(i as f64 * 10.0, timestamp).unwrap();
        }

        assert!(cci.is_ready());

        cci.reset();
        assert!(!cci.is_ready());
        assert_eq!(cci.current(), None);
    }
}
