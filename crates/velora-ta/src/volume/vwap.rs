//! Volume-Weighted Average Price (VWAP)
//!
//! VWAP is the average price weighted by volume over a period (typically a trading day).
//! It's commonly used as a benchmark for institutional trading.
//!
//! Formula:
//! VWAP = Sum(Typical Price * Volume) / Sum(Volume)
//! Typical Price = (High + Low + Close) / 3
//!
//! VWAP resets at the start of each period (e.g., daily).

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, VolumeIndicator},
    types::OhlcBar,
    IndicatorError, IndicatorResult,
};

/// Volume-Weighted Average Price indicator.
///
/// Calculates the average price weighted by volume, typically over a trading session.
/// Used as a benchmark for execution quality.
///
/// # Examples
///
/// ```
/// use velora_ta::{VWAP, VolumeIndicator};
/// use velora_ta::types::OhlcBar;
/// use chrono::Utc;
///
/// let mut vwap = VWAP::new();
/// let timestamp = Utc::now();
///
/// let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
/// let typical_price = bar.typical_price();  // (105 + 95 + 102) / 3
///
/// if let Some(value) = vwap.update_ohlc(&bar, 1000.0, timestamp).unwrap() {
///     println!("VWAP: {:.2}", value);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct VWAP {
    price_volume_sum: f64,
    volume_sum: f64,
    count: usize,
    name: String,
}

impl VWAP {
    /// Creates a new VWAP indicator.
    pub fn new() -> Self {
        VWAP {
            price_volume_sum: 0.0,
            volume_sum: 0.0,
            count: 0,
            name: "VWAP".to_string(),
        }
    }

    /// Update VWAP with OHLC bar and volume.
    ///
    /// Uses typical price: (High + Low + Close) / 3
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if !bar.high.is_finite() || !bar.low.is_finite() || !bar.close.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "OHLC values must be finite numbers".to_string(),
            ));
        }

        if !volume.is_finite() || volume < 0.0 {
            return Err(IndicatorError::InvalidInput(
                "Volume must be a finite non-negative number".to_string(),
            ));
        }

        let typical_price = bar.typical_price();
        self.price_volume_sum += typical_price * volume;
        self.volume_sum += volume;
        self.count += 1;

        Ok(self.calculate_vwap())
    }

    /// Calculate VWAP from accumulated sums.
    fn calculate_vwap(&self) -> Option<f64> {
        if self.volume_sum == 0.0 {
            return None;
        }

        Some(self.price_volume_sum / self.volume_sum)
    }

    /// Get the current VWAP value.
    pub fn current(&self) -> Option<f64> {
        self.calculate_vwap()
    }

    /// Calculate VWAP for OHLC bars with volumes.
    pub fn calculate_ohlc(
        &self,
        bars: &[OhlcBar],
        volumes: &[f64],
    ) -> IndicatorResult<Vec<Option<f64>>> {
        if bars.len() != volumes.len() {
            return Err(IndicatorError::InvalidParameter(
                "Bars and volumes must have the same length".to_string(),
            ));
        }

        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut vwap = Self::new();
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for i in 0..bars.len() {
            result.push(vwap.update_ohlc(&bars[i], volumes[i], timestamp)?);
        }

        Ok(result)
    }
}

impl Default for VWAP {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for VWAP {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        1 // Starts calculating from first bar
    }

    fn is_ready(&self) -> bool {
        self.count >= 1
    }

    fn reset(&mut self) {
        self.price_volume_sum = 0.0;
        self.volume_sum = 0.0;
        self.count = 0;
    }
}

impl VolumeIndicator for VWAP {
    fn update_with_volume(
        &mut self,
        price: f64,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        // For simple VWAP using just close price
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

        self.price_volume_sum += price * volume;
        self.volume_sum += volume;
        self.count += 1;

        Ok(self.calculate_vwap())
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

        let mut vwap = Self::new();
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for i in 0..prices.len() {
            result.push(vwap.update_with_volume(prices[i], volumes[i], timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vwap_creation() {
        let vwap = VWAP::new();
        assert_eq!(vwap.warmup_period(), 1);
        assert!(!vwap.is_ready());
        assert_eq!(vwap.name(), "VWAP");
    }

    #[test]
    fn test_vwap_simple() {
        let mut vwap = VWAP::new();
        let timestamp = Utc::now();

        // Price 100, Volume 1000 -> VWAP = 100
        let value = vwap
            .update_with_volume(100.0, 1000.0, timestamp)
            .unwrap()
            .unwrap();
        assert_eq!(value, 100.0);

        // Price 110, Volume 2000
        // VWAP = (100*1000 + 110*2000) / (1000+2000) = 320000/3000 = 106.666...
        let value = vwap
            .update_with_volume(110.0, 2000.0, timestamp)
            .unwrap()
            .unwrap();
        assert!((value - 106.666666).abs() < 0.0001);
    }

    #[test]
    fn test_vwap_ohlc() {
        let mut vwap = VWAP::new();
        let timestamp = Utc::now();

        let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
        // Typical price = (105 + 95 + 102) / 3 = 100.666...

        let value = vwap.update_ohlc(&bar, 1000.0, timestamp).unwrap().unwrap();
        assert!((value - 100.666666).abs() < 0.0001);
    }

    #[test]
    fn test_vwap_cumulative() {
        let mut vwap = VWAP::new();
        let timestamp = Utc::now();

        // Build up VWAP over multiple ticks
        vwap.update_with_volume(100.0, 100.0, timestamp).unwrap();
        vwap.update_with_volume(101.0, 200.0, timestamp).unwrap();
        vwap.update_with_volume(102.0, 300.0, timestamp).unwrap();

        // VWAP = (100*100 + 101*200 + 102*300) / (100+200+300)
        //      = (10000 + 20200 + 30600) / 600 = 60800/600 = 101.333...
        let value = vwap.current().unwrap();
        assert!((value - 101.333333).abs() < 0.0001);
    }

    #[test]
    fn test_vwap_batch_calculation() {
        let vwap = VWAP::new();
        let prices = vec![100.0, 105.0, 103.0, 108.0, 106.0];
        let volumes = vec![1000.0, 1500.0, 1200.0, 1800.0, 1600.0];
        let values = vwap.calculate_with_volume(&prices, &volumes).unwrap();

        assert_eq!(values.len(), 5);
        // All should have values (VWAP starts from first bar)
        for value in &values {
            assert!(value.is_some());
        }
    }

    #[test]
    fn test_vwap_reset() {
        let mut vwap = VWAP::new();
        let timestamp = Utc::now();

        vwap.update_with_volume(100.0, 1000.0, timestamp).unwrap();
        vwap.update_with_volume(105.0, 2000.0, timestamp).unwrap();

        assert!(vwap.is_ready());

        vwap.reset();
        assert!(!vwap.is_ready());
        assert_eq!(vwap.current(), None);
    }
}
