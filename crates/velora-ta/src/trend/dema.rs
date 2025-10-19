//! Double Exponential Moving Average (DEMA)
//!
//! DEMA reduces lag by applying EMA twice with a specific formula.
//! It's more responsive than a single EMA while being smoother than just doubling the weight.
//!
//! Formula:
//! DEMA = 2 * EMA(price) - EMA(EMA(price))
//!
//! This reduces lag while maintaining smoothness.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::EMA,
    IndicatorError, IndicatorResult,
};

/// Double Exponential Moving Average indicator.
///
/// Reduces lag compared to a single EMA by using two EMAs in combination.
///
/// # Examples
///
/// ```
/// use velora_ta::{DEMA, SingleIndicator};
/// use chrono::Utc;
///
/// let mut dema = DEMA::new(10).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![10.0, 20.0, 30.0, 40.0, 50.0] {
///     if let Some(value) = dema.update(price, timestamp).unwrap() {
///         println!("DEMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct DEMA {
    period: usize,
    ema1: EMA, // EMA of price
    ema2: EMA, // EMA of EMA
    name: String,
}

impl DEMA {
    /// Creates a new DEMA indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for the underlying EMAs (must be > 0)
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

        Ok(DEMA {
            period,
            ema1: EMA::new(period)?,
            ema2: EMA::new(period)?,
            name: format!("DEMA({period})"),
        })
    }

    /// Calculate DEMA value from current EMAs.
    fn calculate_dema(&self) -> Option<f64> {
        let ema1_val = self.ema1.current()?;
        let ema2_val = self.ema2.current()?;

        // DEMA = 2 * EMA1 - EMA2
        Some(2.0 * ema1_val - ema2_val)
    }
}

impl Indicator for DEMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // Needs 2 * period - 1 to be fully warmed up
        // (period for first EMA, then period for second EMA)
        2 * self.period - 1
    }

    fn is_ready(&self) -> bool {
        self.ema1.is_ready() && self.ema2.is_ready()
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
    }
}

impl SingleIndicator for DEMA {
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        // Update first EMA with price
        let ema1_value = self.ema1.update(price, timestamp)?;

        // If first EMA has a value, feed it to second EMA
        if let Some(ema1_val) = ema1_value {
            self.ema2.update(ema1_val, timestamp)?;
        }

        Ok(self.calculate_dema())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_dema()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut dema = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(dema.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dema_creation() {
        let dema = DEMA::new(10).unwrap();
        assert_eq!(dema.warmup_period(), 19); // 2 * 10 - 1
        assert!(!dema.is_ready());
        assert_eq!(dema.name(), "DEMA(10)");
    }

    #[test]
    fn test_dema_invalid_period() {
        let result = DEMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_dema_reduces_lag() {
        use crate::SMA;

        let mut dema = DEMA::new(10).unwrap();
        let mut ema = EMA::new(10).unwrap();
        let mut sma = SMA::new(10).unwrap();
        let timestamp = Utc::now();

        // Create uptrending prices
        let prices: Vec<f64> = (1..=30).map(|x| x as f64).collect();

        for &price in &prices {
            dema.update(price, timestamp).unwrap();
            ema.update(price, timestamp).unwrap();
            sma.update(price, timestamp).unwrap();
        }

        let dema_val = dema.current().unwrap();
        let ema_val = ema.current().unwrap();
        let sma_val = sma.current().unwrap();

        // In an uptrend, DEMA should be highest (least lag),
        // then EMA, then SMA (most lag)
        assert!(dema_val > ema_val);
        assert!(ema_val > sma_val);
    }

    #[test]
    fn test_dema_batch_calculation() {
        let dema = DEMA::new(5).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0];
        let values = dema.calculate(&prices).unwrap();

        assert_eq!(values.len(), 10);

        // DEMA should eventually have values (EMA starts producing after first point)
        // The last value should definitely be Some
        assert!(values[9].is_some());

        // Count how many None values we have
        let none_count = values.iter().filter(|v| v.is_none()).count();
        // EMA produces values quickly, so DEMA should have fewer Nones than the theoretical warmup
        assert!(none_count < 9);
    }

    #[test]
    fn test_dema_reset() {
        let mut dema = DEMA::new(5).unwrap();
        let timestamp = Utc::now();

        // Feed enough data to make it ready
        for i in 1..=10 {
            dema.update(i as f64, timestamp).unwrap();
        }

        assert!(dema.is_ready());

        dema.reset();
        assert!(!dema.is_ready());
        assert_eq!(dema.current(), None);
    }
}
