//! Bollinger Bands
//!
//! Bollinger Bands consist of a middle band (SMA) and two outer bands
//! that are standard deviations away from the middle band.
//!
//! Formula:
//! - Middle Band = SMA(price, period)
//! - Upper Band = Middle Band + (std_dev_multiplier * StdDev)
//! - Lower Band = Middle Band - (std_dev_multiplier * StdDev)
//!
//! Common settings: period=20, std_dev_multiplier=2.0

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::SMA,
    types::MultiIndicatorValue,
    volatility::StdDev,
    IndicatorError, IndicatorResult,
};

/// Bollinger Bands indicator.
///
/// Outputs three values: Upper Band, Middle Band (SMA), Lower Band.
/// The bands expand during volatile periods and contract during quiet periods.
///
/// # Examples
///
/// ```
/// use velora_ta::{BollingerBands, MultiIndicator};
/// use chrono::Utc;
///
/// let mut bb = BollingerBands::new(20, 2.0).unwrap();
/// let timestamp = Utc::now();
///
/// for price in vec![100.0, 105.0, 95.0, 110.0, 90.0] {
///     if let Some(values) = bb.update(price, timestamp).unwrap() {
///         println!("Upper: {:.2}, Middle: {:.2}, Lower: {:.2}",
///                  values[0], values[1], values[2]);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct BollingerBands {
    period: usize,
    std_dev_multiplier: f64,
    sma: SMA,
    std_dev: StdDev,
    name: String,
}

impl BollingerBands {
    /// Creates a new Bollinger Bands indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for SMA and StdDev (typically 20, must be > 1)
    /// * `std_dev_multiplier` - Number of standard deviations for bands (typically 2.0)
    ///
    /// # Errors
    ///
    /// Returns an error if period <= 1 or std_dev_multiplier <= 0.
    pub fn new(period: usize, std_dev_multiplier: f64) -> IndicatorResult<Self> {
        if period <= 1 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 1".to_string(),
            ));
        }

        if std_dev_multiplier <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Standard deviation multiplier must be positive".to_string(),
            ));
        }

        Ok(BollingerBands {
            period,
            std_dev_multiplier,
            sma: SMA::new(period)?,
            std_dev: StdDev::new(period)?,
            name: format!("BB({period},{std_dev_multiplier:.1})"),
        })
    }

    /// Calculate current Bollinger Band values.
    fn calculate_bands(&self) -> Option<Vec<f64>> {
        let middle = self.sma.current()?;
        let std_dev = self.std_dev.current()?;

        let offset = std_dev * self.std_dev_multiplier;
        let upper = middle + offset;
        let lower = middle - offset;

        Some(vec![upper, middle, lower])
    }
}

impl Indicator for BollingerBands {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.sma.is_ready() && self.std_dev.is_ready()
    }

    fn reset(&mut self) {
        self.sma.reset();
        self.std_dev.reset();
    }
}

impl MultiIndicator for BollingerBands {
    fn output_count(&self) -> usize {
        3 // Upper, Middle, Lower
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["Upper", "Middle", "Lower"]
    }

    fn update(
        &mut self,
        price: f64,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        self.sma.update(price, timestamp)?;
        self.std_dev.update(price, timestamp)?;

        Ok(self.calculate_bands())
    }

    fn current(&self) -> Option<Vec<f64>> {
        self.calculate_bands()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut bb = Self::new(self.period, self.std_dev_multiplier)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(bb.update(price, timestamp)?.map(MultiIndicatorValue::from));
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bollinger_bands_creation() {
        let bb = BollingerBands::new(20, 2.0).unwrap();
        assert_eq!(bb.output_count(), 3);
        assert_eq!(bb.output_names(), vec!["Upper", "Middle", "Lower"]);
        assert!(!bb.is_ready());
    }

    #[test]
    fn test_bollinger_bands_invalid_params() {
        assert!(BollingerBands::new(0, 2.0).is_err());
        assert!(BollingerBands::new(1, 2.0).is_err());
        assert!(BollingerBands::new(20, 0.0).is_err());
        assert!(BollingerBands::new(20, -1.0).is_err());
    }

    #[test]
    fn test_bollinger_bands_no_volatility() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        let timestamp = Utc::now();

        // All same price = no volatility
        for _ in 0..5 {
            bb.update(100.0, timestamp).unwrap();
        }

        if let Some(values) = bb.current() {
            let upper = values[0];
            let middle = values[1];
            let lower = values[2];

            // With no volatility, all bands should be equal
            assert_eq!(upper, 100.0);
            assert_eq!(middle, 100.0);
            assert_eq!(lower, 100.0);
        }
    }

    #[test]
    fn test_bollinger_bands_symmetry() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        let timestamp = Utc::now();

        let prices = vec![95.0, 100.0, 105.0, 100.0, 95.0];

        for &price in &prices {
            bb.update(price, timestamp).unwrap();
        }

        if let Some(values) = bb.current() {
            let upper = values[0];
            let middle = values[1];
            let lower = values[2];

            // Bands should be symmetric around middle
            let upper_distance = upper - middle;
            let lower_distance = middle - lower;

            assert!((upper_distance - lower_distance).abs() < 0.0001);
        }
    }

    #[test]
    fn test_bollinger_bands_width_increases_with_volatility() {
        let mut bb_low_vol = BollingerBands::new(5, 2.0).unwrap();
        let mut bb_high_vol = BollingerBands::new(5, 2.0).unwrap();
        let timestamp = Utc::now();

        // Low volatility prices
        for _ in 0..5 {
            bb_low_vol.update(100.0, timestamp).unwrap();
        }

        // High volatility prices
        let volatile_prices = vec![90.0, 110.0, 85.0, 115.0, 95.0];
        for &price in &volatile_prices {
            bb_high_vol.update(price, timestamp).unwrap();
        }

        let low_vol_bands = bb_low_vol.current().unwrap();
        let high_vol_bands = bb_high_vol.current().unwrap();

        let low_vol_width = low_vol_bands[0] - low_vol_bands[2];
        let high_vol_width = high_vol_bands[0] - high_vol_bands[2];

        // High volatility should have wider bands
        assert!(high_vol_width > low_vol_width);
    }

    #[test]
    fn test_bollinger_bands_batch_calculation() {
        let bb = BollingerBands::new(5, 2.0).unwrap();
        let prices = vec![100.0, 102.0, 104.0, 106.0, 108.0, 110.0];
        let values = bb.calculate(&prices).unwrap();

        assert_eq!(values.len(), 6);

        // Last value should have all three bands
        if let Some(ref bands) = values[5] {
            assert_eq!(bands.values.len(), 3);
            let upper = bands.values[0];
            let middle = bands.values[1];
            let lower = bands.values[2];

            // Upper > Middle > Lower
            assert!(upper > middle);
            assert!(middle > lower);
        }
    }

    #[test]
    fn test_bollinger_bands_reset() {
        let mut bb = BollingerBands::new(5, 2.0).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            bb.update(i as f64, timestamp).unwrap();
        }

        assert!(bb.is_ready());

        bb.reset();
        assert!(!bb.is_ready());
    }
}
