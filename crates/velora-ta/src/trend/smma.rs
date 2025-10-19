//! Smoothed Moving Average (SMMA) / Running Moving Average (RMA)
//!
//! Also known as Modified Moving Average (MMA) or Running Moving Average (RMA).
//! Similar to EMA but with a different smoothing factor, resulting in slower response.
//!
//! Formula:
//! SMMA[i] = (SMMA[i-1] * (period - 1) + price[i]) / period
//! First SMMA = SMA of first 'period' prices
//!
//! This is equivalent to EMA with alpha = 1/period instead of 2/(period+1).

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Smoothed Moving Average indicator.
///
/// Also known as Modified Moving Average (MMA) or Running Moving Average (RMA).
/// Smoother than EMA with the same period.
///
/// # Examples
///
/// ```
/// use velora_ta::{SMMA, SingleIndicator};
/// use chrono::Utc;
///
/// let mut smma = SMMA::new(14).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![10.0, 20.0, 30.0, 40.0, 50.0] {
///     if let Some(value) = smma.update(price, timestamp).unwrap() {
///         println!("SMMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SMMA {
    period: usize,
    buffer: CircularBuffer<f64>,
    current_value: Option<f64>,
    alpha: f64,
    name: String,
}

impl SMMA {
    /// Creates a new SMMA indicator with the specified period.
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

        Ok(SMMA {
            period,
            buffer: CircularBuffer::new(period),
            current_value: None,
            alpha: 1.0 / period as f64,
            name: format!("SMMA({period})"),
        })
    }

    /// Initialize SMMA with SMA of first period values.
    fn initialize_sma(&mut self) -> Option<f64> {
        if !self.buffer.is_full() {
            return None;
        }

        let sum: f64 = self.buffer.sum();
        Some(sum / self.period as f64)
    }
}

impl Indicator for SMMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.current_value.is_some()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.current_value = None;
    }
}

impl SingleIndicator for SMMA {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        if let Some(prev_smma) = self.current_value {
            // SMMA[i] = (SMMA[i-1] * (period - 1) + price) / period
            // This is equivalent to: SMMA[i-1] + (price - SMMA[i-1]) / period
            self.current_value = Some(prev_smma + (price - prev_smma) * self.alpha);
        } else {
            // Buffer not full yet, keep collecting
            self.buffer.push(price);

            // Once buffer is full, initialize with SMA
            if self.buffer.is_full() {
                self.current_value = self.initialize_sma();
            }
        }

        Ok(self.current_value)
    }

    fn current(&self) -> Option<f64> {
        self.current_value
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut smma = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(smma.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_smma_creation() {
        let smma = SMMA::new(14).unwrap();
        assert_eq!(smma.warmup_period(), 14);
        assert!(!smma.is_ready());
        assert_eq!(smma.name(), "SMMA(14)");
    }

    #[test]
    fn test_smma_invalid_period() {
        let result = SMMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_smma_initialization() {
        let mut smma = SMMA::new(3).unwrap();
        let timestamp = Utc::now();

        // First 3 prices: 10, 20, 30
        assert_eq!(smma.update(10.0, timestamp).unwrap(), None);
        assert_eq!(smma.update(20.0, timestamp).unwrap(), None);

        // At 3rd price, should initialize with SMA: (10+20+30)/3 = 20.0
        let value = smma.update(30.0, timestamp).unwrap().unwrap();
        assert_eq!(value, 20.0);
    }

    #[test]
    fn test_smma_updates() {
        let mut smma = SMMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Initialize with 10, 20, 30 -> SMA = 20.0
        smma.update(10.0, timestamp).unwrap();
        smma.update(20.0, timestamp).unwrap();
        smma.update(30.0, timestamp).unwrap();

        // Next value: SMMA = (20 * 2 + 40) / 3 = 80/3 = 26.666...
        let value = smma.update(40.0, timestamp).unwrap().unwrap();
        assert!((value - 26.666666).abs() < 0.0001);
    }

    #[test]
    fn test_smma_smoother_than_ema() {
        use crate::EMA;

        let mut smma = SMMA::new(14).unwrap();
        let mut ema = EMA::new(14).unwrap();
        let timestamp = Utc::now();

        // Feed same data to both
        let prices: Vec<f64> = (1..=50).map(|x| x as f64).collect();

        for &price in &prices {
            smma.update(price, timestamp).unwrap();
            ema.update(price, timestamp).unwrap();
        }

        // Both should have values
        let smma_val = smma.current().unwrap();
        let ema_val = ema.current().unwrap();

        // In an uptrend, EMA responds faster (higher value)
        // SMMA is smoother (lower value, more lag)
        assert!(ema_val > smma_val);
    }

    #[test]
    fn test_smma_batch_calculation() {
        let smma = SMMA::new(5).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0, 60.0, 70.0];
        let values = smma.calculate(&prices).unwrap();

        assert_eq!(values.len(), 7);

        // First 4 should be None
        for i in 0..4 {
            assert_eq!(values[i], None);
        }

        // 5th value should be SMA: (10+20+30+40+50)/5 = 30.0
        assert_eq!(values[4], Some(30.0));

        // 6th value: (30*4 + 60)/5 = 180/5 = 36.0
        assert_eq!(values[5], Some(36.0));
    }

    #[test]
    fn test_smma_reset() {
        let mut smma = SMMA::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            smma.update(i as f64, timestamp).unwrap();
        }

        assert!(smma.is_ready());

        smma.reset();
        assert!(!smma.is_ready());
        assert_eq!(smma.current(), None);
    }
}
