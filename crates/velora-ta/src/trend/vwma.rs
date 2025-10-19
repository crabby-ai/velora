//! Volume-Weighted Moving Average (VWMA)
//!
//! A moving average that weights prices by their volume.
//! Prices with higher volume have more influence on the average.
//!
//! Formula:
//! VWMA = Sum(price * volume) / Sum(volume)
//!
//! This gives more weight to prices where more trading activity occurred.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, VolumeIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Data point for VWMA calculation.
#[derive(Debug, Clone, Copy, Default)]
struct VwmaPoint {
    price: f64,
    volume: f64,
}

/// Volume-Weighted Moving Average indicator.
///
/// Weights prices by their volume, giving more importance to prices
/// where significant trading activity occurred.
///
/// # Examples
///
/// ```
/// use velora_ta::{VWMA, VolumeIndicator};
/// use chrono::Utc;
///
/// let mut vwma = VWMA::new(10).unwrap();
/// let timestamp = Utc::now();
///
/// // Feed price-volume pairs
/// if let Some(value) = vwma.update_with_volume(50000.0, 100.0, timestamp).unwrap() {
///     println!("VWMA: {:.2}", value);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VWMA {
    period: usize,
    buffer: CircularBuffer<VwmaPoint>,
    name: String,
}

impl VWMA {
    /// Creates a new VWMA indicator with the specified period.
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

        Ok(VWMA {
            period,
            buffer: CircularBuffer::new(period),
            name: format!("VWMA({period})"),
        })
    }

    /// Calculate VWMA from current buffer.
    fn calculate_vwma(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let mut price_volume_sum = 0.0;
        let mut volume_sum = 0.0;

        for point in self.buffer.iter() {
            price_volume_sum += point.price * point.volume;
            volume_sum += point.volume;
        }

        if volume_sum == 0.0 {
            return None; // Avoid division by zero
        }

        Some(price_volume_sum / volume_sum)
    }

    /// Get the current indicator value without updating.
    ///
    /// Returns `None` if the indicator hasn't received enough data yet.
    pub fn current(&self) -> Option<f64> {
        self.calculate_vwma()
    }
}

impl Indicator for VWMA {
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

impl VolumeIndicator for VWMA {
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
            return Err(IndicatorError::InvalidPrice(
                "Volume must be a finite non-negative number".to_string(),
            ));
        }

        self.buffer.push(VwmaPoint { price, volume });
        Ok(self.calculate_vwma())
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

        let mut vwma = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for i in 0..prices.len() {
            result.push(vwma.update_with_volume(prices[i], volumes[i], timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwma_creation() {
        let vwma = VWMA::new(10).unwrap();
        assert_eq!(vwma.warmup_period(), 10);
        assert!(!vwma.is_ready());
        assert_eq!(vwma.name(), "VWMA(10)");
    }

    #[test]
    fn test_vwma_invalid_period() {
        let result = VWMA::new(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_vwma_calculation() {
        let mut vwma = VWMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Price: 10, Volume: 100
        // Price: 20, Volume: 200
        // Price: 30, Volume: 300
        assert_eq!(
            vwma.update_with_volume(10.0, 100.0, timestamp).unwrap(),
            None
        );
        assert_eq!(
            vwma.update_with_volume(20.0, 200.0, timestamp).unwrap(),
            None
        );

        // VWMA = (10*100 + 20*200 + 30*300) / (100+200+300)
        //      = (1000 + 4000 + 9000) / 600 = 14000/600 = 23.333...
        let value = vwma
            .update_with_volume(30.0, 300.0, timestamp)
            .unwrap()
            .unwrap();
        assert!((value - 23.333333).abs() < 0.0001);
    }

    #[test]
    fn test_vwma_uniform_volume() {
        use crate::{SingleIndicator, SMA};

        let mut vwma = VWMA::new(5).unwrap();
        let mut sma = SMA::new(5).unwrap();
        let timestamp = Utc::now();

        // With uniform volume, VWMA should equal SMA
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];

        for &price in &prices {
            vwma.update_with_volume(price, 100.0, timestamp).unwrap(); // Uniform volume
            sma.update(price, timestamp).unwrap();
        }

        let vwma_val = vwma.current().unwrap();
        let sma_val = sma.current().unwrap();

        // Should be equal with uniform volumes
        assert!((vwma_val - sma_val).abs() < 0.0001);
    }

    #[test]
    fn test_vwma_high_volume_influence() {
        let mut vwma = VWMA::new(3).unwrap();
        let timestamp = Utc::now();

        // Price 50 with very high volume should dominate
        vwma.update_with_volume(10.0, 1.0, timestamp).unwrap();
        vwma.update_with_volume(20.0, 1.0, timestamp).unwrap();
        let value = vwma
            .update_with_volume(50.0, 1000.0, timestamp)
            .unwrap()
            .unwrap();

        // VWMA should be very close to 50 due to high volume
        assert!(value > 49.0);
    }

    #[test]
    fn test_vwma_batch_calculation() {
        let vwma = VWMA::new(3).unwrap();
        let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
        let volumes = vec![100.0, 200.0, 300.0, 400.0, 500.0];
        let values = vwma.calculate_with_volume(&prices, &volumes).unwrap();

        assert_eq!(values.len(), 5);
        assert_eq!(values[0], None);
        assert_eq!(values[1], None);
        assert!(values[2].is_some());
    }

    #[test]
    fn test_vwma_mismatched_lengths() {
        let vwma = VWMA::new(3).unwrap();
        let prices = vec![10.0, 20.0, 30.0];
        let volumes = vec![100.0, 200.0]; // Different length

        let result = vwma.calculate_with_volume(&prices, &volumes);
        assert!(result.is_err());
    }

    #[test]
    fn test_vwma_reset() {
        let mut vwma = VWMA::new(3).unwrap();
        let timestamp = Utc::now();

        vwma.update_with_volume(10.0, 100.0, timestamp).unwrap();
        vwma.update_with_volume(20.0, 200.0, timestamp).unwrap();
        vwma.update_with_volume(30.0, 300.0, timestamp).unwrap();

        assert!(vwma.is_ready());

        vwma.reset();
        assert!(!vwma.is_ready());
        assert_eq!(vwma.current(), None);
    }
}
