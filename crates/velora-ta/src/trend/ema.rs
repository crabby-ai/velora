//! Exponential Moving Average (EMA) indicator.
//!
//! The EMA gives more weight to recent prices, making it more responsive
//! to new information compared to the Simple Moving Average (SMA).

use crate::utils::math::ema_multiplier;
use crate::{Indicator, IndicatorError, IndicatorResult, SingleIndicator};
use chrono::{DateTime, Utc};

/// Exponential Moving Average indicator.
///
/// The EMA applies exponentially decreasing weights to older prices.
/// More recent prices have greater influence on the average.
///
/// Formula:
/// - EMA(today) = (Price(today) × k) + (EMA(yesterday) × (1 - k))
/// - where k = 2 / (period + 1)
///
/// # Example
///
/// ```ignore
/// use velora_strategy::indicators::{EMA, SingleIndicator};
///
/// let mut ema = EMA::new(10)?;
///
/// for price in prices {
///     if let Some(value) = ema.update(price, timestamp)? {
///         println!("EMA(10): {}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct EMA {
    /// The number of periods for the EMA
    period: usize,
    /// The smoothing multiplier: 2 / (period + 1)
    multiplier: f64,
    /// Current EMA value (None until initialized)
    current_value: Option<f64>,
    /// Count of data points received
    count: usize,
    /// Indicator name
    name: String,
}

impl EMA {
    /// Create a new EMA indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the EMA (must be > 0)
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
            multiplier: ema_multiplier(period),
            current_value: None,
            count: 0,
            name: format!("EMA({period})"),
        })
    }

    /// Get the period of this EMA.
    pub fn period(&self) -> usize {
        self.period
    }

    /// Get the smoothing multiplier.
    pub fn multiplier(&self) -> f64 {
        self.multiplier
    }
}

impl Indicator for EMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // EMA technically starts producing values immediately,
        // but we wait for 'period' data points for stability
        self.period
    }

    fn is_ready(&self) -> bool {
        self.count >= self.period
    }

    fn reset(&mut self) {
        self.current_value = None;
        self.count = 0;
    }
}

impl SingleIndicator for EMA {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        // Validate price
        if !price.is_finite() || price < 0.0 {
            return Err(IndicatorError::InvalidPrice(format!(
                "Price must be finite and non-negative, got {price}"
            )));
        }

        self.count += 1;

        match self.current_value {
            None => {
                // First value: use price as initial EMA
                self.current_value = Some(price);
                if self.count >= self.period {
                    Ok(Some(price))
                } else {
                    Ok(None)
                }
            }
            Some(prev_ema) => {
                // EMA formula: EMA = (Price * k) + (EMA_prev * (1 - k))
                let new_ema = (price * self.multiplier) + (prev_ema * (1.0 - self.multiplier));
                self.current_value = Some(new_ema);

                if self.count >= self.period {
                    Ok(Some(new_ema))
                } else {
                    Ok(None)
                }
            }
        }
    }

    fn current(&self) -> Option<f64> {
        if self.is_ready() {
            self.current_value
        } else {
            None
        }
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut result = Vec::with_capacity(prices.len());
        let mut ema = Self::new(self.period)?;

        for &price in prices {
            result.push(ema.update(price, Utc::now())?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ema_creation() {
        let ema = EMA::new(20).unwrap();
        assert_eq!(ema.name(), "EMA(20)");
        assert_eq!(ema.period(), 20);
        assert_eq!(ema.warmup_period(), 20);
        assert!(!ema.is_ready());

        // Verify multiplier: 2/(20+1) = 0.095238...
        assert!((ema.multiplier() - 0.095238).abs() < 0.001);
    }

    #[test]
    fn test_ema_invalid_period() {
        let result = EMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_ema_simple_calculation() {
        let mut ema = EMA::new(3).unwrap();
        let timestamp = Utc::now();

        // First price initializes EMA
        assert_eq!(ema.update(10.0, timestamp).unwrap(), None);
        assert!(!ema.is_ready());

        assert_eq!(ema.update(20.0, timestamp).unwrap(), None);
        assert!(!ema.is_ready());

        // Third update returns a value
        let result = ema.update(30.0, timestamp).unwrap();
        assert!(result.is_some());
        assert!(ema.is_ready());
    }

    #[test]
    fn test_ema_responds_faster_than_sma() {
        use crate::trend::sma::SMA;

        let mut ema = EMA::new(5).unwrap();
        let mut sma = SMA::new(5).unwrap();
        let timestamp = Utc::now();

        // Initialize both with same values
        let prices = vec![10.0, 10.0, 10.0, 10.0, 10.0];
        for price in prices {
            ema.update(price, timestamp).unwrap();
            sma.update(price, timestamp).unwrap();
        }

        // Both should be at 10.0
        assert_eq!(ema.current(), Some(10.0));
        assert_eq!(sma.current(), Some(10.0));

        // Add a spike - EMA should react more
        ema.update(20.0, timestamp).unwrap();
        sma.update(20.0, timestamp).unwrap();

        let ema_val = ema.current().unwrap();
        let sma_val = sma.current().unwrap();

        // EMA should be higher than SMA (more responsive)
        assert!(ema_val > sma_val);
        println!("EMA: {ema_val}, SMA: {sma_val}");
    }

    #[test]
    fn test_ema_reset() {
        let mut ema = EMA::new(3).unwrap();
        let timestamp = Utc::now();

        ema.update(10.0, timestamp).unwrap();
        ema.update(20.0, timestamp).unwrap();
        ema.update(30.0, timestamp).unwrap();

        assert!(ema.is_ready());
        assert!(ema.current().is_some());

        ema.reset();
        assert!(!ema.is_ready());
        assert_eq!(ema.current(), None);
    }

    #[test]
    fn test_ema_batch_calculation() {
        let ema = EMA::new(3).unwrap();
        let prices = vec![10.0, 11.0, 12.0, 13.0, 14.0];

        let results = ema.calculate(&prices).unwrap();

        assert_eq!(results.len(), 5);
        assert_eq!(results[0], None); // Warmup
        assert_eq!(results[1], None); // Warmup
        assert!(results[2].is_some()); // First valid value
        assert!(results[3].is_some());
        assert!(results[4].is_some());

        // Values should be increasing
        assert!(results[2].unwrap() < results[3].unwrap());
        assert!(results[3].unwrap() < results[4].unwrap());
    }

    #[test]
    fn test_ema_invalid_price() {
        let mut ema = EMA::new(3).unwrap();
        let timestamp = Utc::now();

        assert!(ema.update(f64::NAN, timestamp).is_err());
        assert!(ema.update(f64::INFINITY, timestamp).is_err());
        assert!(ema.update(-10.0, timestamp).is_err());
    }

    #[test]
    fn test_ema_multiplier_calculation() {
        let ema9 = EMA::new(9).unwrap();
        assert!((ema9.multiplier() - 0.2).abs() < 0.001); // 2/(9+1) = 0.2

        let ema14 = EMA::new(14).unwrap();
        assert!((ema14.multiplier() - 0.133333).abs() < 0.001); // 2/(14+1) ≈ 0.133
    }

    #[test]
    fn test_ema_converges_to_price() {
        let mut ema = EMA::new(5).unwrap();
        let timestamp = Utc::now();

        // Feed constant price
        for _ in 0..100 {
            ema.update(100.0, timestamp).unwrap();
        }

        // EMA should converge to 100.0
        let current = ema.current().unwrap();
        assert!((current - 100.0).abs() < 0.001);
    }
}
