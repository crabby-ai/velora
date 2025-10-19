//! Triple Exponential Moving Average (TEMA)
//!
//! TEMA further reduces lag by applying EMA three times with a specific formula.
//! It's even more responsive than DEMA while maintaining smoothness.
//!
//! Formula:
//! TEMA = 3 * EMA(price) - 3 * EMA(EMA(price)) + EMA(EMA(EMA(price)))
//!
//! This provides minimal lag with good noise filtering.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::EMA,
    IndicatorError, IndicatorResult,
};

/// Triple Exponential Moving Average indicator.
///
/// Reduces lag even further than DEMA by using three EMAs in combination.
///
/// # Examples
///
/// ```
/// use velora_ta::{TEMA, SingleIndicator};
/// use chrono::Utc;
///
/// let mut tema = TEMA::new(10).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![10.0, 20.0, 30.0, 40.0, 50.0] {
///     if let Some(value) = tema.update(price, timestamp).unwrap() {
///         println!("TEMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct TEMA {
    period: usize,
    ema1: EMA, // EMA of price
    ema2: EMA, // EMA of EMA
    ema3: EMA, // EMA of EMA of EMA
    name: String,
}

impl TEMA {
    /// Creates a new TEMA indicator with the specified period.
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

        Ok(TEMA {
            period,
            ema1: EMA::new(period)?,
            ema2: EMA::new(period)?,
            ema3: EMA::new(period)?,
            name: format!("TEMA({period})"),
        })
    }

    /// Calculate TEMA value from current EMAs.
    fn calculate_tema(&self) -> Option<f64> {
        let ema1_val = self.ema1.current()?;
        let ema2_val = self.ema2.current()?;
        let ema3_val = self.ema3.current()?;

        // TEMA = 3 * EMA1 - 3 * EMA2 + EMA3
        Some(3.0 * ema1_val - 3.0 * ema2_val + ema3_val)
    }
}

impl Indicator for TEMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // Needs 3 * period - 2 to be fully warmed up
        3 * self.period - 2
    }

    fn is_ready(&self) -> bool {
        self.ema1.is_ready() && self.ema2.is_ready() && self.ema3.is_ready()
    }

    fn reset(&mut self) {
        self.ema1.reset();
        self.ema2.reset();
        self.ema3.reset();
    }
}

impl SingleIndicator for TEMA {
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
            let ema2_value = self.ema2.update(ema1_val, timestamp)?;

            // If second EMA has a value, feed it to third EMA
            if let Some(ema2_val) = ema2_value {
                self.ema3.update(ema2_val, timestamp)?;
            }
        }

        Ok(self.calculate_tema())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_tema()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut tema = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(tema.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tema_creation() {
        let tema = TEMA::new(10).unwrap();
        assert_eq!(tema.warmup_period(), 28); // 3 * 10 - 2
        assert!(!tema.is_ready());
        assert_eq!(tema.name(), "TEMA(10)");
    }

    #[test]
    fn test_tema_invalid_period() {
        let result = TEMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_tema_reduces_lag_more() {
        use crate::{DEMA, SMA};

        let mut tema = TEMA::new(10).unwrap();
        let mut dema = DEMA::new(10).unwrap();
        let mut ema = EMA::new(10).unwrap();
        let mut sma = SMA::new(10).unwrap();
        let timestamp = Utc::now();

        // Create uptrending prices
        let prices: Vec<f64> = (1..=50).map(|x| x as f64).collect();

        for &price in &prices {
            tema.update(price, timestamp).unwrap();
            dema.update(price, timestamp).unwrap();
            ema.update(price, timestamp).unwrap();
            sma.update(price, timestamp).unwrap();
        }

        let tema_val = tema.current().unwrap();
        let dema_val = dema.current().unwrap();
        let ema_val = ema.current().unwrap();
        let sma_val = sma.current().unwrap();

        // In an uptrend, TEMA should have least lag (highest value),
        // followed by DEMA, then EMA, then SMA (most lag)
        assert!(tema_val > dema_val, "TEMA: {tema_val}, DEMA: {dema_val}");
        assert!(dema_val > ema_val, "DEMA: {dema_val}, EMA: {ema_val}");
        assert!(ema_val > sma_val, "EMA: {ema_val}, SMA: {sma_val}");
    }

    #[test]
    fn test_tema_batch_calculation() {
        let tema = TEMA::new(5).unwrap();
        let prices = vec![
            10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0, 80.0, 90.0, 100.0, 110.0, 120.0, 130.0,
            140.0, 150.0,
        ];
        let values = tema.calculate(&prices).unwrap();

        assert_eq!(values.len(), 15);

        // TEMA should eventually have values (EMA starts producing after first point)
        // The last values should definitely be Some
        assert!(values[13].is_some());
        assert!(values[14].is_some());

        // Count how many None values we have
        let none_count = values.iter().filter(|v| v.is_none()).count();
        // EMA produces values quickly, so TEMA should have fewer Nones than theoretical warmup
        assert!(none_count < 13);
    }

    #[test]
    fn test_tema_reset() {
        let mut tema = TEMA::new(5).unwrap();
        let timestamp = Utc::now();

        // Feed enough data to make it ready
        for i in 1..=15 {
            tema.update(i as f64, timestamp).unwrap();
        }

        assert!(tema.is_ready());

        tema.reset();
        assert!(!tema.is_ready());
        assert_eq!(tema.current(), None);
    }
}
