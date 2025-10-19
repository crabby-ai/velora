//! Momentum Indicator
//!
//! Simple momentum measures the absolute change in price over a given period.
//!
//! Formula:
//! Momentum = price - price[n periods ago]
//!
//! Positive values indicate upward momentum, negative values indicate downward momentum.
//! Unlike ROC, this is an absolute value, not a percentage.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Momentum indicator.
///
/// Calculates the absolute change in price over a specified period.
/// More sensitive to price changes than ROC for higher-priced assets.
///
/// # Examples
///
/// ```
/// use velora_ta::{Momentum, SingleIndicator};
/// use chrono::Utc;
///
/// let mut momentum = Momentum::new(10).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 105.0, 110.0, 115.0, 120.0] {
///     if let Some(value) = momentum.update(price, timestamp).unwrap() {
///         println!("Momentum: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct Momentum {
    period: usize,
    buffer: CircularBuffer<f64>,
    name: String,
}

impl Momentum {
    /// Creates a new Momentum indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the momentum calculation (must be > 0)
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

        Ok(Momentum {
            period,
            buffer: CircularBuffer::new(period + 1),
            name: format!("Momentum({period})"),
        })
    }

    /// Calculate momentum from current buffer.
    fn calculate_momentum(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let old_price = self.buffer.first()?;
        let current_price = self.buffer.last()?;

        Some(current_price - old_price)
    }
}

impl Indicator for Momentum {
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

impl SingleIndicator for Momentum {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_momentum())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_momentum()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut momentum = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(momentum.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::SingleIndicator;

    #[test]
    fn test_momentum_creation() {
        let momentum = Momentum::new(10).unwrap();
        assert_eq!(momentum.warmup_period(), 11);
        assert!(!momentum.is_ready());
        assert_eq!(momentum.name(), "Momentum(10)");
    }

    #[test]
    fn test_momentum_invalid_period() {
        assert!(Momentum::new(0).is_err());
    }

    #[test]
    fn test_momentum_positive() {
        let mut momentum = Momentum::new(3).unwrap();
        let timestamp = Utc::now();

        // Prices: 100, 105, 110, 115
        momentum.update(100.0, timestamp).unwrap();
        momentum.update(105.0, timestamp).unwrap();
        momentum.update(110.0, timestamp).unwrap();

        // Momentum = 115 - 100 = 15
        let value = momentum.update(115.0, timestamp).unwrap().unwrap();
        assert_eq!(value, 15.0);
    }

    #[test]
    fn test_momentum_negative() {
        let mut momentum = Momentum::new(3).unwrap();
        let timestamp = Utc::now();

        // Prices: 100, 95, 90, 85
        momentum.update(100.0, timestamp).unwrap();
        momentum.update(95.0, timestamp).unwrap();
        momentum.update(90.0, timestamp).unwrap();

        // Momentum = 85 - 100 = -15
        let value = momentum.update(85.0, timestamp).unwrap().unwrap();
        assert_eq!(value, -15.0);
    }

    #[test]
    fn test_momentum_vs_roc() {
        use crate::momentum::ROC;

        let mut momentum = Momentum::new(5).unwrap();
        let mut roc = ROC::new(5).unwrap();
        let timestamp = Utc::now();

        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];

        for &price in &prices {
            momentum.update(price, timestamp).unwrap();
            roc.update(price, timestamp).unwrap();
        }

        let momentum_val = momentum.current().unwrap();
        let roc_val = roc.current().unwrap();

        // Momentum = 110 - 100 = 10
        assert_eq!(momentum_val, 10.0);

        // ROC = ((110 - 100) / 100) * 100 = 10%
        assert_eq!(roc_val, 10.0);

        // They're equal in this case, but ROC is percentage-based
        // so it normalizes across different price levels
    }

    #[test]
    fn test_momentum_batch_calculation() {
        let momentum = Momentum::new(3).unwrap();
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];
        let values = momentum.calculate(&prices).unwrap();

        assert_eq!(values.len(), 6);

        // First 3 should be None
        for i in 0..3 {
            assert_eq!(values[i], None);
        }

        // 4th value: 106 - 100 = 6
        assert_eq!(values[3].unwrap(), 6.0);

        // 5th value: 108 - 102 = 6
        assert_eq!(values[4].unwrap(), 6.0);
    }

    #[test]
    fn test_momentum_reset() {
        let mut momentum = Momentum::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            momentum.update(i as f64 * 10.0, timestamp).unwrap();
        }

        assert!(momentum.is_ready());

        momentum.reset();
        assert!(!momentum.is_ready());
        assert_eq!(momentum.current(), None);
    }
}
