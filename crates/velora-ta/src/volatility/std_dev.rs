//! Standard Deviation
//!
//! Standard deviation measures the dispersion of prices around their mean.
//! Higher values indicate more volatile prices.
//!
//! Formula:
//! StdDev = sqrt(sum((price - mean)^2) / period)

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Standard Deviation indicator.
///
/// Measures price dispersion around the mean.
/// Higher values indicate higher volatility.
///
/// # Examples
///
/// ```
/// use velora_ta::{StdDev, SingleIndicator};
/// use chrono::Utc;
///
/// let mut std_dev = StdDev::new(20).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 105.0, 95.0, 110.0, 90.0] {
///     if let Some(value) = std_dev.update(price, timestamp).unwrap() {
///         println!("StdDev: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct StdDev {
    period: usize,
    buffer: CircularBuffer<f64>,
    name: String,
}

impl StdDev {
    /// Creates a new Standard Deviation indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for calculation (must be > 1)
    ///
    /// # Errors
    ///
    /// Returns an error if period is 0 or 1.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period <= 1 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 1".to_string(),
            ));
        }

        Ok(StdDev {
            period,
            buffer: CircularBuffer::new(period),
            name: format!("StdDev({period})"),
        })
    }

    /// Get current standard deviation value.
    fn calculate_std_dev(&self) -> Option<f64> {
        self.buffer.std_dev()
    }
}

impl Indicator for StdDev {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full()
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl SingleIndicator for StdDev {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_std_dev())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_std_dev()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut std_dev = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(std_dev.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_dev_creation() {
        let std_dev = StdDev::new(20).unwrap();
        assert_eq!(std_dev.warmup_period(), 20);
        assert!(!std_dev.is_ready());
        assert_eq!(std_dev.name(), "StdDev(20)");
    }

    #[test]
    fn test_std_dev_invalid_period() {
        assert!(StdDev::new(0).is_err());
        assert!(StdDev::new(1).is_err());
    }

    #[test]
    fn test_std_dev_no_volatility() {
        let mut std_dev = StdDev::new(5).unwrap();
        let timestamp = Utc::now();

        // All same price = no volatility
        for _ in 0..5 {
            std_dev.update(100.0, timestamp).unwrap();
        }

        let value = std_dev.current().unwrap();
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_std_dev_high_volatility() {
        let mut std_dev = StdDev::new(5).unwrap();
        let timestamp = Utc::now();

        // High variation in prices
        let prices = vec![90.0, 110.0, 85.0, 115.0, 95.0];

        for &price in &prices {
            std_dev.update(price, timestamp).unwrap();
        }

        let value = std_dev.current().unwrap();
        // Should have significant standard deviation
        assert!(value > 10.0);
    }

    #[test]
    fn test_std_dev_batch_calculation() {
        let std_dev = StdDev::new(5).unwrap();
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];
        let values = std_dev.calculate(&prices).unwrap();

        assert_eq!(values.len(), 6);

        // CircularBuffer can calculate std_dev before being full,
        // so we just verify we get values eventually
        // The last values should definitely have results
        assert!(values[4].is_some());
        assert!(values[5].is_some());
    }

    #[test]
    fn test_std_dev_reset() {
        let mut std_dev = StdDev::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            std_dev.update(i as f64, timestamp).unwrap();
        }

        assert!(std_dev.is_ready());

        std_dev.reset();
        assert!(!std_dev.is_ready());
    }
}
