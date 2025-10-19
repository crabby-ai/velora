//! Rate of Change (ROC)
//!
//! Measures the percentage change in price over a given period.
//!
//! Formula:
//! ROC = ((price - price[n periods ago]) / price[n periods ago]) * 100
//!
//! Positive values indicate upward momentum, negative values indicate downward momentum.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Rate of Change indicator.
///
/// Calculates the percentage change in price over a specified period.
/// Values above 0 indicate positive momentum, below 0 indicate negative momentum.
///
/// # Examples
///
/// ```
/// use velora_ta::{ROC, SingleIndicator};
/// use chrono::Utc;
///
/// let mut roc = ROC::new(10).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 105.0, 110.0, 115.0, 120.0] {
///     if let Some(value) = roc.update(price, timestamp).unwrap() {
///         println!("ROC: {:.2}%", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ROC {
    period: usize,
    buffer: CircularBuffer<f64>,
    name: String,
}

impl ROC {
    /// Creates a new ROC indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the rate of change calculation (must be > 0)
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

        Ok(ROC {
            period,
            buffer: CircularBuffer::new(period + 1), // Need period + 1 to compare
            name: format!("ROC({period})"),
        })
    }

    /// Calculate ROC from current buffer.
    fn calculate_roc(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let old_price = self.buffer.first()?;
        let current_price = self.buffer.last()?;

        if old_price == 0.0 {
            return None; // Avoid division by zero
        }

        // ROC = ((current - old) / old) * 100
        let roc = ((current_price - old_price) / old_price) * 100.0;
        Some(roc)
    }
}

impl Indicator for ROC {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full()
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl SingleIndicator for ROC {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() || price < 0.0 {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite positive number".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_roc())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_roc()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut roc = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(roc.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SingleIndicator;

    #[test]
    fn test_roc_creation() {
        let roc = ROC::new(10).unwrap();
        assert_eq!(roc.warmup_period(), 11);
        assert!(!roc.is_ready());
        assert_eq!(roc.name(), "ROC(10)");
    }

    #[test]
    fn test_roc_invalid_period() {
        assert!(ROC::new(0).is_err());
    }

    #[test]
    fn test_roc_positive_momentum() {
        let mut roc = ROC::new(3).unwrap();
        let timestamp = Utc::now();

        // Prices: 100, 105, 110, 115
        roc.update(100.0, timestamp).unwrap();
        roc.update(105.0, timestamp).unwrap();
        roc.update(110.0, timestamp).unwrap();

        // ROC = ((115 - 100) / 100) * 100 = 15%
        let value = roc.update(115.0, timestamp).unwrap().unwrap();
        assert!((value - 15.0).abs() < 0.0001);
    }

    #[test]
    fn test_roc_negative_momentum() {
        let mut roc = ROC::new(3).unwrap();
        let timestamp = Utc::now();

        // Prices: 100, 95, 90, 85
        roc.update(100.0, timestamp).unwrap();
        roc.update(95.0, timestamp).unwrap();
        roc.update(90.0, timestamp).unwrap();

        // ROC = ((85 - 100) / 100) * 100 = -15%
        let value = roc.update(85.0, timestamp).unwrap().unwrap();
        assert!((value - (-15.0)).abs() < 0.0001);
    }

    #[test]
    fn test_roc_no_change() {
        let mut roc = ROC::new(5).unwrap();
        let timestamp = Utc::now();

        // All same price
        for _ in 0..=5 {
            roc.update(100.0, timestamp).unwrap();
        }

        let value = roc.current().unwrap();
        assert_eq!(value, 0.0); // No change = 0%
    }

    #[test]
    fn test_roc_batch_calculation() {
        let roc = ROC::new(5).unwrap();
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0, 112.0, 114.0];
        let values = roc.calculate(&prices).unwrap();

        assert_eq!(values.len(), 8);

        // Should have None values during warmup
        for i in 0..5 {
            assert_eq!(values[i], None);
        }

        // 6th value: ((110 - 100) / 100) * 100 = 10%
        assert!((values[5].unwrap() - 10.0).abs() < 0.0001);
    }

    #[test]
    fn test_roc_reset() {
        let mut roc = ROC::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            roc.update(i as f64 * 10.0, timestamp).unwrap();
        }

        assert!(roc.is_ready());

        roc.reset();
        assert!(!roc.is_ready());
        assert_eq!(roc.current(), None);
    }
}
