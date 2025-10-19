//! Weighted Moving Average (WMA)
//!
//! A moving average that assigns linearly decreasing weights to older prices.
//! The most recent price has the highest weight (period), the next has (period-1), etc.
//!
//! Formula:
//! WMA = (P1 * period + P2 * (period-1) + ... + Pn * 1) / (period * (period + 1) / 2)
//!
//! Where P1 is the most recent price and Pn is the oldest price in the period.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Weighted Moving Average indicator.
///
/// Assigns linearly decreasing weights to older prices, giving more importance
/// to recent price movements compared to SMA.
///
/// # Examples
///
/// ```
/// use velora_ta::{WMA, SingleIndicator};
/// use chrono::Utc;
///
/// let mut wma = WMA::new(3).unwrap();
/// let timestamp = Utc::now();
///
/// // Feed prices: 10, 20, 30
/// assert_eq!(wma.update(10.0, timestamp).unwrap(), None);  // Not ready
/// assert_eq!(wma.update(20.0, timestamp).unwrap(), None);  // Not ready
///
/// // WMA = (10*1 + 20*2 + 30*3) / (1+2+3) = 140/6 = 23.33...
/// let value = wma.update(30.0, timestamp).unwrap().unwrap();
/// assert!((value - 23.333333).abs() < 0.0001);
/// ```
#[derive(Debug, Clone)]
pub struct WMA {
    period: usize,
    buffer: CircularBuffer<f64>,
    weight_sum: f64,
    name: String,
}

impl WMA {
    /// Creates a new WMA indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods to average (must be > 0)
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

        // Calculate sum of weights: 1 + 2 + ... + period = period * (period + 1) / 2
        let weight_sum = (period * (period + 1)) as f64 / 2.0;

        Ok(WMA {
            period,
            buffer: CircularBuffer::new(period),
            weight_sum,
            name: format!("WMA({period})"),
        })
    }

    /// Calculate WMA from current buffer.
    fn calculate_wma(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let mut weighted_sum = 0.0;

        // Most recent value gets highest weight (period)
        // Oldest value gets lowest weight (1)
        for (i, &price) in self.buffer.iter().enumerate() {
            let weight = (i + 1) as f64;
            weighted_sum += price * weight;
        }

        Some(weighted_sum / self.weight_sum)
    }
}

impl Indicator for WMA {
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

impl SingleIndicator for WMA {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_wma())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_wma()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut wma = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(wma.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wma_creation() {
        let wma = WMA::new(10).unwrap();
        assert_eq!(wma.warmup_period(), 10);
        assert!(!wma.is_ready());
        assert_eq!(wma.name(), "WMA(10)");
    }

    #[test]
    fn test_wma_invalid_period() {
        let result = WMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_wma_calculation() {
        let mut wma = WMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Feed prices: 10, 20, 30
        assert_eq!(wma.update(10.0, timestamp).unwrap(), None);
        assert_eq!(wma.update(20.0, timestamp).unwrap(), None);

        // WMA = (10*1 + 20*2 + 30*3) / (1+2+3) = 140/6 = 23.333...
        let value = wma.update(30.0, timestamp).unwrap().unwrap();
        assert!((value - 23.333333).abs() < 0.0001);
    }

    #[test]
    fn test_wma_updates() {
        let mut wma = WMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Fill buffer: 10, 20, 30
        wma.update(10.0, timestamp).unwrap();
        wma.update(20.0, timestamp).unwrap();
        wma.update(30.0, timestamp).unwrap();

        // Add 40: buffer becomes [20, 30, 40]
        // WMA = (20*1 + 30*2 + 40*3) / 6 = 200/6 = 33.333...
        let value = wma.update(40.0, timestamp).unwrap().unwrap();
        assert!((value - 33.333333).abs() < 0.0001);
    }

    #[test]
    fn test_wma_batch_calculation() {
        let wma = WMA::new(3).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let values = wma.calculate(&prices).unwrap();

        assert_eq!(values.len(), 5);
        assert_eq!(values[0], None);
        assert_eq!(values[1], None);

        // Value at index 2: (10*1 + 20*2 + 30*3) / 6 = 23.333...
        assert!((values[2].unwrap() - 23.333333).abs() < 0.0001);

        // Value at index 3: (20*1 + 30*2 + 40*3) / 6 = 33.333...
        assert!((values[3].unwrap() - 33.333333).abs() < 0.0001);
    }

    #[test]
    fn test_wma_reset() {
        let mut wma = WMA::new(3).unwrap();
        let timestamp = Utc::now();

        wma.update(10.0, timestamp).unwrap();
        wma.update(20.0, timestamp).unwrap();
        wma.update(30.0, timestamp).unwrap();

        assert!(wma.is_ready());

        wma.reset();
        assert!(!wma.is_ready());
        assert_eq!(wma.current(), None);
    }

    #[test]
    fn test_wma_vs_sma() {
        use crate::SMA;

        let mut wma = WMA::new(5).unwrap();
        let mut sma = SMA::new(5).unwrap();
        let timestamp = Utc::now();

        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        for &price in &prices {
            wma.update(price, timestamp).unwrap();
            sma.update(price, timestamp).unwrap();
        }

        let wma_val = wma.current().unwrap();
        let sma_val = sma.current().unwrap();

        // WMA gives more weight to recent prices, so should be higher
        // with an uptrend
        assert!(wma_val > sma_val);

        // SMA = (10+20+30+40+50)/5 = 30.0
        assert_eq!(sma_val, 30.0);

        // WMA = (10*1 + 20*2 + 30*3 + 40*4 + 50*5) / 15
        //     = (10 + 40 + 90 + 160 + 250) / 15 = 550/15 = 36.666...
        assert!((wma_val - 36.666666).abs() < 0.0001);
    }
}
