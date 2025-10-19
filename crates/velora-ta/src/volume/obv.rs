//! On-Balance Volume (OBV)
//!
//! OBV is a cumulative volume indicator that adds volume on up days
//! and subtracts volume on down days.
//!
//! Formula:
//! - If close > previous close: OBV = OBV_prev + volume
//! - If close < previous close: OBV = OBV_prev - volume
//! - If close = previous close: OBV = OBV_prev
//!
//! Rising OBV confirms uptrends, falling OBV confirms downtrends.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, VolumeIndicator},
    IndicatorError, IndicatorResult,
};

/// On-Balance Volume indicator.
///
/// Cumulative volume indicator that confirms price trends.
/// Divergences between OBV and price can signal reversals.
///
/// # Examples
///
/// ```
/// use velora_ta::{OBV, VolumeIndicator};
/// use chrono::Utc;
///
/// let mut obv = OBV::new();
/// let timestamp = Utc::now();
///
/// // Rising prices with volume
/// obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
/// obv.update_with_volume(105.0, 1500.0, timestamp).unwrap();
/// obv.update_with_volume(110.0, 2000.0, timestamp).unwrap();
///
/// if let Some(value) = obv.current() {
///     println!("OBV: {:.0}", value);  // Should be positive and increasing
/// }
/// ```
#[derive(Debug, Clone)]
pub struct OBV {
    obv_value: f64,
    previous_close: Option<f64>,
    count: usize,
    name: String,
}

impl OBV {
    /// Creates a new OBV indicator.
    pub fn new() -> Self {
        OBV {
            obv_value: 0.0,
            previous_close: None,
            count: 0,
            name: "OBV".to_string(),
        }
    }
}

impl Default for OBV {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for OBV {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        2 // Needs at least 2 data points to compare
    }

    fn is_ready(&self) -> bool {
        self.count >= 2
    }

    fn reset(&mut self) {
        self.obv_value = 0.0;
        self.previous_close = None;
        self.count = 0;
    }
}

impl VolumeIndicator for OBV {
    fn update_with_volume(
        &mut self,
        price: f64,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        if !volume.is_finite() || volume < 0.0 {
            return Err(IndicatorError::InvalidInput(
                "Volume must be a finite non-negative number".to_string(),
            ));
        }

        self.count += 1;

        if let Some(prev_close) = self.previous_close {
            if price > prev_close {
                self.obv_value += volume;
            } else if price < prev_close {
                self.obv_value -= volume;
            }
            // If price == prev_close, OBV stays the same
        }

        self.previous_close = Some(price);

        if self.is_ready() {
            Ok(Some(self.obv_value))
        } else {
            Ok(None)
        }
    }

    fn calculate_with_volume(
        &self,
        prices: &[f64],
        volumes: &[f64],
    ) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.len() != volumes.len() {
            return Err(IndicatorError::InvalidParameter(
                "Prices and volumes must have the same length".to_string(),
            ));
        }

        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut obv = Self::new();
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for i in 0..prices.len() {
            result.push(obv.update_with_volume(prices[i], volumes[i], timestamp)?);
        }

        Ok(result)
    }
}

impl OBV {
    /// Get the current OBV value.
    pub fn current(&self) -> Option<f64> {
        if self.is_ready() {
            Some(self.obv_value)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obv_creation() {
        let obv = OBV::new();
        assert_eq!(obv.warmup_period(), 2);
        assert!(!obv.is_ready());
        assert_eq!(obv.name(), "OBV");
    }

    #[test]
    fn test_obv_uptrend() {
        let mut obv = OBV::new();
        let timestamp = Utc::now();

        // Rising prices with volume
        obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        obv.update_with_volume(105.0, 1500.0, timestamp).unwrap();
        obv.update_with_volume(110.0, 2000.0, timestamp).unwrap();

        let value = obv.current().unwrap();
        // OBV = +1500 +2000 = 3500
        assert_eq!(value, 3500.0);
    }

    #[test]
    fn test_obv_downtrend() {
        let mut obv = OBV::new();
        let timestamp = Utc::now();

        // Falling prices with volume
        obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        obv.update_with_volume(95.0, 1500.0, timestamp).unwrap();
        obv.update_with_volume(90.0, 2000.0, timestamp).unwrap();

        let value = obv.current().unwrap();
        // OBV = -1500 -2000 = -3500
        assert_eq!(value, -3500.0);
    }

    #[test]
    fn test_obv_mixed() {
        let mut obv = OBV::new();
        let timestamp = Utc::now();

        obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        obv.update_with_volume(105.0, 2000.0, timestamp).unwrap(); // +2000
        obv.update_with_volume(103.0, 1500.0, timestamp).unwrap(); // -1500
        obv.update_with_volume(107.0, 3000.0, timestamp).unwrap(); // +3000

        let value = obv.current().unwrap();
        // OBV = 0 + 2000 - 1500 + 3000 = 3500
        assert_eq!(value, 3500.0);
    }

    #[test]
    fn test_obv_no_change() {
        let mut obv = OBV::new();
        let timestamp = Utc::now();

        obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        obv.update_with_volume(100.0, 2000.0, timestamp).unwrap(); // No change
        obv.update_with_volume(100.0, 3000.0, timestamp).unwrap(); // No change

        let value = obv.current().unwrap();
        // OBV stays at 0 when price doesn't change
        assert_eq!(value, 0.0);
    }

    #[test]
    fn test_obv_batch_calculation() {
        let obv = OBV::new();
        let prices = vec![100.0, 105.0, 103.0, 108.0, 106.0];
        let volumes = vec![1000.0, 1500.0, 1200.0, 1800.0, 1600.0];
        let values = obv.calculate_with_volume(&prices, &volumes).unwrap();

        assert_eq!(values.len(), 5);
        assert_eq!(values[0], None); // First value, no previous close
        assert!(values[1].is_some());
        assert!(values[4].is_some());
    }

    #[test]
    fn test_obv_reset() {
        let mut obv = OBV::new();
        let timestamp = Utc::now();

        obv.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        obv.update_with_volume(105.0, 2000.0, timestamp).unwrap();

        assert!(obv.is_ready());

        obv.reset();
        assert!(!obv.is_ready());
        assert_eq!(obv.current(), None);
        assert_eq!(obv.obv_value, 0.0);
    }
}
