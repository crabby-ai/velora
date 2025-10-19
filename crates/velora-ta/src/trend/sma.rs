//! Simple Moving Average (SMA) indicator.
//!
//! The SMA is the unweighted mean of the previous n data points.
//! It's one of the simplest and most commonly used technical indicators.

use crate::{CircularBuffer, Indicator, IndicatorError, IndicatorResult, SingleIndicator};
use chrono::{DateTime, Utc};

/// Simple Moving Average indicator.
///
/// Calculates the arithmetic mean of prices over a specified period.
/// Each price has equal weight in the calculation.
///
/// Formula: SMA = (P1 + P2 + ... + Pn) / n
///
/// # Example
///
/// ```ignore
/// use velora_strategy::indicators::{SMA, SingleIndicator};
///
/// let mut sma = SMA::new(3)?;
///
/// // Not enough data yet
/// assert_eq!(sma.update(10.0, timestamp)?, None);
/// assert_eq!(sma.update(20.0, timestamp)?, None);
///
/// // Now we have enough data
/// assert_eq!(sma.update(30.0, timestamp)?, Some(20.0));  // (10+20+30)/3 = 20
/// assert_eq!(sma.update(40.0, timestamp)?, Some(30.0));  // (20+30+40)/3 = 30
/// ```
#[derive(Debug, Clone)]
pub struct SMA {
    /// The number of periods for the moving average
    period: usize,
    /// Circular buffer to store recent prices
    prices: CircularBuffer<f64>,
    /// Cached sum for efficient calculation
    sum: f64,
    /// Indicator name
    name: String,
}

impl SMA {
    /// Create a new SMA indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the moving average (must be > 0)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if period is 0.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            period,
            prices: CircularBuffer::new(period),
            sum: 0.0,
            name: format!("SMA({period})"),
        })
    }

    /// Get the period of this SMA.
    pub fn period(&self) -> usize {
        self.period
    }
}

impl Indicator for SMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.prices.is_full()
    }

    fn reset(&mut self) {
        self.prices.clear();
        self.sum = 0.0;
    }
}

impl SingleIndicator for SMA {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        // Validate price
        if !price.is_finite() || price < 0.0 {
            return Err(IndicatorError::InvalidPrice(format!(
                "Price must be finite and non-negative, got {price}"
            )));
        }

        // If buffer is full, subtract the value being removed
        if self.prices.is_full() {
            if let Some(old_price) = self.prices.first() {
                self.sum -= old_price;
            }
        }

        // Add new price
        self.prices.push(price);
        self.sum += price;

        // Return SMA if we have enough data
        if self.prices.is_full() {
            Ok(Some(self.sum / self.period as f64))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> Option<f64> {
        if self.prices.is_full() {
            Some(self.sum / self.period as f64)
        } else {
            None
        }
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(prices.len());
        let mut sma = Self::new(self.period)?;

        for &price in prices {
            result.push(sma.update(price, Utc::now())?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_creation() {
        let sma = SMA::new(20).unwrap();
        assert_eq!(sma.name(), "SMA(20)");
        assert_eq!(sma.period(), 20);
        assert_eq!(sma.warmup_period(), 20);
        assert!(!sma.is_ready());
    }

    #[test]
    fn test_sma_invalid_period() {
        let result = SMA::new(0);
        assert!(result.is_err());
        match result {
            Err(IndicatorError::InvalidParameter(_)) => {}
            _ => panic!("Expected InvalidParameter error"),
        }
    }

    #[test]
    fn test_sma_simple_calculation() {
        let mut sma = SMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Not enough data yet
        assert_eq!(sma.update(10.0, timestamp).unwrap(), None);
        assert!(!sma.is_ready());

        assert_eq!(sma.update(20.0, timestamp).unwrap(), None);
        assert!(!sma.is_ready());

        // Now we have enough data: (10 + 20 + 30) / 3 = 20
        assert_eq!(sma.update(30.0, timestamp).unwrap(), Some(20.0));
        assert!(sma.is_ready());

        // Rolling window: (20 + 30 + 40) / 3 = 30
        assert_eq!(sma.update(40.0, timestamp).unwrap(), Some(30.0));

        // Current value should be 30
        assert_eq!(sma.current(), Some(30.0));
    }

    #[test]
    fn test_sma_batch_calculation() {
        let sma = SMA::new(3).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        let results = sma.calculate(&prices).unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0], None); // Not enough data
        assert_eq!(results[1], None); // Not enough data
        assert_eq!(results[2], Some(20.0)); // (10+20+30)/3 = 20
        assert_eq!(results[3], Some(30.0)); // (20+30+40)/3 = 30
        assert_eq!(results[4], Some(40.0)); // (30+40+50)/3 = 40
    }

    #[test]
    fn test_sma_reset() {
        let mut sma = SMA::new(3).unwrap();
        let timestamp = Utc::now();

        sma.update(10.0, timestamp).unwrap();
        sma.update(20.0, timestamp).unwrap();
        sma.update(30.0, timestamp).unwrap();

        assert!(sma.is_ready());
        assert_eq!(sma.current(), Some(20.0));

        sma.reset();
        assert!(!sma.is_ready());
        assert_eq!(sma.current(), None);
    }

    #[test]
    fn test_sma_invalid_price() {
        let mut sma = SMA::new(3).unwrap();
        let timestamp = Utc::now();

        let result = sma.update(f64::NAN, timestamp);
        assert!(result.is_err());

        let result = sma.update(f64::INFINITY, timestamp);
        assert!(result.is_err());

        let result = sma.update(-10.0, timestamp);
        assert!(result.is_err());
    }

    #[test]
    fn test_sma_single_period() {
        let mut sma = SMA::new(1).unwrap();
        let timestamp = Utc::now();

        // With period=1, SMA should just return the current price
        assert_eq!(sma.update(100.0, timestamp).unwrap(), Some(100.0));
        assert_eq!(sma.update(200.0, timestamp).unwrap(), Some(200.0));
        assert_eq!(sma.update(150.0, timestamp).unwrap(), Some(150.0));
    }

    #[test]
    fn test_sma_large_period() {
        let mut sma = SMA::new(100).unwrap();
        let timestamp = Utc::now();

        // Fill with prices
        for i in 1..=99 {
            assert_eq!(sma.update(i as f64, timestamp).unwrap(), None);
        }

        // 100th update should return SMA
        let result = sma.update(100.0, timestamp).unwrap();
        assert!(result.is_some());

        // SMA of 1..=100 = 50.5
        let expected = 50.5;
        let actual = result.unwrap();
        assert!((actual - expected).abs() < 0.001);
    }
}
