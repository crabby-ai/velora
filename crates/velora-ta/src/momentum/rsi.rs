//! Relative Strength Index (RSI) indicator.
//!
//! RSI measures the magnitude of recent price changes to evaluate
//! overbought or oversold conditions on a scale of 0 to 100.
//!
//! Developed by J. Welles Wilder Jr., introduced in "New Concepts in Technical Trading Systems" (1978).

use crate::trend::EMA;
use crate::{Indicator, IndicatorError, IndicatorResult, SingleIndicator};
use chrono::{DateTime, Utc};

/// Relative Strength Index (RSI) indicator.
///
/// RSI oscillates between 0 and 100, with traditional interpretation:
/// - RSI > 70: Overbought (potential sell signal)
/// - RSI < 30: Oversold (potential buy signal)
/// - RSI around 50: Neutral
///
/// Formula:
/// ```text
/// RSI = 100 - (100 / (1 + RS))
/// where RS = Average Gain / Average Loss
/// ```
///
/// The averages are calculated using EMA (Exponential Moving Average).
///
/// # Example
///
/// ```ignore
/// use velora_strategy::indicators::{RSI, SingleIndicator};
///
/// let mut rsi = RSI::new(14)?;
///
/// for price in prices {
///     if let Some(value) = rsi.update(price, timestamp)? {
///         if value > 70.0 {
///             println!("Overbought: RSI = {:.2}", value);
///         } else if value < 30.0 {
///             println!("Oversold: RSI = {:.2}", value);
///         }
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct RSI {
    /// The number of periods for RSI calculation
    period: usize,
    /// EMA of gains
    avg_gain: EMA,
    /// EMA of losses
    avg_loss: EMA,
    /// Previous price (needed to calculate gain/loss)
    prev_price: Option<f64>,
    /// Number of data points processed
    count: usize,
    /// Indicator name
    name: String,
}

impl RSI {
    /// Create a new RSI indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for RSI calculation (typically 14)
    ///
    /// # Errors
    ///
    /// Returns `IndicatorError::InvalidParameter` if period is 0.
    ///
    /// # Typical Periods
    ///
    /// - 14 periods: Standard (Wilder's original)
    /// - 9 periods: More sensitive, more signals
    /// - 25 periods: Less sensitive, fewer signals
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(Self {
            period,
            avg_gain: EMA::new(period)?,
            avg_loss: EMA::new(period)?,
            prev_price: None,
            count: 0,
            name: format!("RSI({period})"),
        })
    }

    /// Get the period of this RSI.
    pub fn period(&self) -> usize {
        self.period
    }

    /// Calculate RSI from RS (Relative Strength).
    ///
    /// RSI = 100 - (100 / (1 + RS))
    fn calculate_rsi(avg_gain: f64, avg_loss: f64) -> f64 {
        // Handle edge cases
        if avg_loss == 0.0 {
            // No losses means RSI = 100 (maximum)
            return 100.0;
        }

        if avg_gain == 0.0 {
            // No gains means RSI = 0 (minimum)
            return 0.0;
        }

        let rs = avg_gain / avg_loss;
        100.0 - (100.0 / (1.0 + rs))
    }
}

impl Indicator for RSI {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // Need period + 1 to start calculating (need previous price for first change)
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.count >= self.warmup_period()
    }

    fn reset(&mut self) {
        self.avg_gain.reset();
        self.avg_loss.reset();
        self.prev_price = None;
        self.count = 0;
    }
}

impl SingleIndicator for RSI {
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        // Validate price
        if !price.is_finite() || price < 0.0 {
            return Err(IndicatorError::InvalidPrice(format!(
                "Price must be finite and non-negative, got {price}"
            )));
        }

        self.count += 1;

        // Need previous price to calculate change
        let prev = match self.prev_price {
            None => {
                self.prev_price = Some(price);
                return Ok(None);
            }
            Some(p) => p,
        };

        // Calculate price change
        let change = price - prev;
        self.prev_price = Some(price);

        // Separate into gain and loss
        let gain = if change > 0.0 { change } else { 0.0 };
        let loss = if change < 0.0 { -change } else { 0.0 };

        // Update EMAs
        self.avg_gain.update(gain, timestamp)?;
        self.avg_loss.update(loss, timestamp)?;

        // Calculate RSI if we have enough data
        if self.is_ready() {
            let avg_gain = self.avg_gain.current().unwrap_or(0.0);
            let avg_loss = self.avg_loss.current().unwrap_or(0.0);
            let rsi = Self::calculate_rsi(avg_gain, avg_loss);
            Ok(Some(rsi))
        } else {
            Ok(None)
        }
    }

    fn current(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let avg_gain = self.avg_gain.current()?;
        let avg_loss = self.avg_loss.current()?;
        Some(Self::calculate_rsi(avg_gain, avg_loss))
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(prices.len());
        let mut rsi = Self::new(self.period)?;

        for &price in prices {
            result.push(rsi.update(price, Utc::now())?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rsi_creation() {
        let rsi = RSI::new(14).unwrap();
        assert_eq!(rsi.name(), "RSI(14)");
        assert_eq!(rsi.period(), 14);
        assert_eq!(rsi.warmup_period(), 15); // period + 1
        assert!(!rsi.is_ready());
    }

    #[test]
    fn test_rsi_invalid_period() {
        let result = RSI::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_rsi_all_gains() {
        let mut rsi = RSI::new(3).unwrap();
        let timestamp = Utc::now();

        // Start at 100
        rsi.update(100.0, timestamp).unwrap();

        // All prices going up
        rsi.update(101.0, timestamp).unwrap();
        rsi.update(102.0, timestamp).unwrap();
        let result = rsi.update(103.0, timestamp).unwrap();

        // All gains, no losses -> RSI should be 100
        assert!(result.is_some());
        let rsi_value = result.unwrap();
        assert_eq!(rsi_value, 100.0);
    }

    #[test]
    fn test_rsi_all_losses() {
        let mut rsi = RSI::new(3).unwrap();
        let timestamp = Utc::now();

        // Start at 100
        rsi.update(100.0, timestamp).unwrap();

        // All prices going down
        rsi.update(99.0, timestamp).unwrap();
        rsi.update(98.0, timestamp).unwrap();
        let result = rsi.update(97.0, timestamp).unwrap();

        // All losses, no gains -> RSI should be 0
        assert!(result.is_some());
        let rsi_value = result.unwrap();
        assert_eq!(rsi_value, 0.0);
    }

    #[test]
    fn test_rsi_mixed_changes() {
        let mut rsi = RSI::new(5).unwrap();
        let timestamp = Utc::now();

        // Price sequence with mixed gains/losses
        let prices = vec![100.0, 102.0, 101.0, 103.0, 102.5, 104.0];

        let mut results = Vec::new();
        for price in prices {
            results.push(rsi.update(price, timestamp).unwrap());
        }

        // First 5 should be None (warmup)
        assert!(results[0].is_none());
        assert!(results[1].is_none());
        assert!(results[2].is_none());
        assert!(results[3].is_none());
        assert!(results[4].is_none());

        // 6th should have RSI value between 0 and 100
        assert!(results[5].is_some());
        let rsi_value = results[5].unwrap();
        assert!((0.0..=100.0).contains(&rsi_value));

        // With more gains than losses, RSI should be > 50
        assert!(rsi_value > 50.0);
    }

    #[test]
    fn test_rsi_calculate_rsi_function() {
        // Test the RSI calculation formula

        // Equal gains and losses -> RS = 1 -> RSI = 50
        let rsi = RSI::calculate_rsi(1.0, 1.0);
        assert_eq!(rsi, 50.0);

        // More gains than losses -> RSI > 50
        let rsi = RSI::calculate_rsi(2.0, 1.0);
        assert!(rsi > 50.0);
        assert!((rsi - 66.666666).abs() < 0.001); // RS=2 -> RSI ≈ 66.67

        // More losses than gains -> RSI < 50
        let rsi = RSI::calculate_rsi(1.0, 2.0);
        assert!(rsi < 50.0);
        assert!((rsi - 33.333333).abs() < 0.001); // RS=0.5 -> RSI ≈ 33.33

        // No losses -> RSI = 100
        let rsi = RSI::calculate_rsi(10.0, 0.0);
        assert_eq!(rsi, 100.0);

        // No gains -> RSI = 0
        let rsi = RSI::calculate_rsi(0.0, 10.0);
        assert_eq!(rsi, 0.0);
    }

    #[test]
    fn test_rsi_reset() {
        let mut rsi = RSI::new(3).unwrap();
        let timestamp = Utc::now();

        // Build up some state
        rsi.update(100.0, timestamp).unwrap();
        rsi.update(101.0, timestamp).unwrap();
        rsi.update(102.0, timestamp).unwrap();
        rsi.update(103.0, timestamp).unwrap();

        assert!(rsi.is_ready());

        // Reset
        rsi.reset();
        assert!(!rsi.is_ready());
        assert_eq!(rsi.current(), None);
    }

    #[test]
    fn test_rsi_batch_calculation() {
        let rsi = RSI::new(3).unwrap();

        // Prices trending up
        let prices = vec![100.0, 101.0, 102.0, 103.0, 104.0];
        let results = rsi.calculate(&prices).unwrap();

        assert_eq!(results.len(), 5);

        // First values should be None during warmup
        // With period=3: need prev price (1) + 3 EMA periods = 4 total
        assert!(results[0].is_none()); // No prev price yet
        assert!(results[1].is_none()); // EMA not ready (1/3)
        assert!(results[2].is_none()); // EMA not ready (2/3)

        // At index 3, EMA is ready (has 3 data points), so RSI returns
        assert!(results[3].is_some());

        // Later values should also have RSI
        assert!(results[4].is_some());
        let rsi_value = results[4].unwrap();

        // All gains -> RSI should be high
        assert!(rsi_value > 70.0);
    }

    #[test]
    fn test_rsi_invalid_price() {
        let mut rsi = RSI::new(14).unwrap();
        let timestamp = Utc::now();

        assert!(rsi.update(f64::NAN, timestamp).is_err());
        assert!(rsi.update(f64::INFINITY, timestamp).is_err());
        assert!(rsi.update(-10.0, timestamp).is_err());
    }

    #[test]
    fn test_rsi_range_bounds() {
        let mut rsi = RSI::new(5).unwrap();
        let timestamp = Utc::now();

        // Test with random-ish price movements
        let prices = vec![
            100.0, 102.0, 101.0, 103.0, 102.5, 104.0, 103.5, 105.0, 104.0, 106.0,
        ];

        for price in prices {
            if let Some(value) = rsi.update(price, timestamp).unwrap() {
                // RSI must always be between 0 and 100
                assert!(value >= 0.0, "RSI value {value} is below 0");
                assert!(value <= 100.0, "RSI value {value} is above 100");
            }
        }
    }

    #[test]
    fn test_rsi_current_value() {
        let mut rsi = RSI::new(3).unwrap();
        let timestamp = Utc::now();

        assert_eq!(rsi.current(), None);

        rsi.update(100.0, timestamp).unwrap();
        rsi.update(101.0, timestamp).unwrap();
        rsi.update(102.0, timestamp).unwrap();
        assert_eq!(rsi.current(), None); // Still in warmup

        rsi.update(103.0, timestamp).unwrap();
        assert!(rsi.current().is_some());
        assert_eq!(rsi.current().unwrap(), 100.0); // All gains
    }
}
