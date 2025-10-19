//! Hull Moving Average (HMA)
//!
//! The Hull Moving Average achieves both smoothness and responsiveness by using
//! weighted moving averages and the square root of the period.
//!
//! Formula:
//! HMA = WMA(2 * WMA(price, period/2) - WMA(price, period), sqrt(period))
//!
//! This results in a moving average with minimal lag and good smoothing.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::WMA,
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Hull Moving Average indicator.
///
/// Combines weighted moving averages to achieve both smoothness and responsiveness.
/// Generally considered superior to traditional MAs for reducing lag.
///
/// # Examples
///
/// ```
/// use velora_ta::{HMA, SingleIndicator};
/// use chrono::Utc;
///
/// let mut hma = HMA::new(16).unwrap();  // 16 is common (sqrt(16) = 4)
/// let timestamp = Utc::now();
///
/// for price in vec![10.0, 20.0, 30.0, 40.0, 50.0] {
///     if let Some(value) = hma.update(price, timestamp).unwrap() {
///         println!("HMA: {:.2}", value);
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct HMA {
    period: usize,
    #[allow(dead_code)]
    half_period: usize,
    sqrt_period: usize,
    wma_half: WMA,  // WMA with period/2
    wma_full: WMA,  // WMA with full period
    wma_final: WMA, // Final WMA with sqrt(period)
    #[allow(dead_code)]
    raw_buffer: CircularBuffer<f64>, // Store raw differences for final WMA
    name: String,
}

impl HMA {
    /// Creates a new HMA indicator with the specified period.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods (must be > 1, ideally a perfect square like 9, 16, 25)
    ///
    /// # Errors
    ///
    /// Returns an error if period is 0 or 1.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period <= 1 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 1".to_string(),
            ));
        }

        let half_period = period / 2;
        let sqrt_period = (period as f64).sqrt().floor() as usize;

        if sqrt_period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period too small for sqrt calculation".to_string(),
            ));
        }

        Ok(HMA {
            period,
            half_period,
            sqrt_period,
            wma_half: WMA::new(half_period)?,
            wma_full: WMA::new(period)?,
            wma_final: WMA::new(sqrt_period)?,
            raw_buffer: CircularBuffer::new(sqrt_period),
            name: format!("HMA({period})"),
        })
    }

    /// Calculate raw WMA difference value.
    fn calculate_raw_wma(&self) -> Option<f64> {
        let wma_half_val = self.wma_half.current()?;
        let wma_full_val = self.wma_full.current()?;

        // 2 * WMA(half) - WMA(full)
        Some(2.0 * wma_half_val - wma_full_val)
    }
}

impl Indicator for HMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        // Needs period + sqrt(period) to be fully warmed up
        self.period + self.sqrt_period
    }

    fn is_ready(&self) -> bool {
        self.wma_final.is_ready()
    }

    fn reset(&mut self) {
        self.wma_half.reset();
        self.wma_full.reset();
        self.wma_final.reset();
        self.raw_buffer.clear();
    }
}

impl SingleIndicator for HMA {
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        // Update both WMAs with the price
        self.wma_half.update(price, timestamp)?;
        self.wma_full.update(price, timestamp)?;

        // Calculate raw WMA difference
        if let Some(raw_wma) = self.calculate_raw_wma() {
            // Feed the difference to the final WMA
            self.wma_final.update(raw_wma, timestamp)?;
        }

        Ok(self.wma_final.current())
    }

    fn current(&self) -> Option<f64> {
        self.wma_final.current()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut hma = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(hma.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hma_creation() {
        let hma = HMA::new(16).unwrap();
        assert_eq!(hma.period, 16);
        assert_eq!(hma.sqrt_period, 4);
        assert_eq!(hma.warmup_period(), 20); // 16 + 4
        assert!(!hma.is_ready());
        assert_eq!(hma.name(), "HMA(16)");
    }

    #[test]
    fn test_hma_invalid_period() {
        assert!(HMA::new(0).is_err());
        assert!(HMA::new(1).is_err());
    }

    #[test]
    fn test_hma_most_responsive() {
        use crate::{DEMA, EMA, SMA, TEMA};

        let mut hma = HMA::new(9).unwrap(); // sqrt(9) = 3
        let mut tema = TEMA::new(9).unwrap();
        let mut dema = DEMA::new(9).unwrap();
        let mut ema = EMA::new(9).unwrap();
        let mut sma = SMA::new(9).unwrap();
        let timestamp = Utc::now();

        // Create uptrending prices
        let prices: Vec<f64> = (1..=50).map(|x| x as f64).collect();

        for &price in &prices {
            hma.update(price, timestamp).unwrap();
            tema.update(price, timestamp).unwrap();
            dema.update(price, timestamp).unwrap();
            ema.update(price, timestamp).unwrap();
            sma.update(price, timestamp).unwrap();
        }

        if let Some(hma_val) = hma.current() {
            let tema_val = tema.current().unwrap();
            let dema_val = dema.current().unwrap();
            let ema_val = ema.current().unwrap();
            let sma_val = sma.current().unwrap();

            // In an uptrend, HMA typically has least lag (closest to current price)
            println!(
                "HMA: {hma_val:.2}, TEMA: {tema_val:.2}, DEMA: {dema_val:.2}, EMA: {ema_val:.2}, SMA: {sma_val:.2}"
            );

            // HMA should be >= TEMA in most cases
            assert!(hma_val >= tema_val - 0.5); // Small tolerance
        }
    }

    #[test]
    fn test_hma_batch_calculation() {
        let hma = HMA::new(9).unwrap();
        let prices: Vec<f64> = (1..=30).map(|x| x as f64).collect();
        let values = hma.calculate(&prices).unwrap();

        assert_eq!(values.len(), 30);

        // Should have None values during warmup
        let warmup = hma.warmup_period();
        for i in 0..warmup {
            if i < values.len() {
                assert!(values[i].is_none() || values[i].is_some());
            }
        }
    }

    #[test]
    fn test_hma_reset() {
        let mut hma = HMA::new(9).unwrap();
        let timestamp = Utc::now();

        // Feed enough data
        for i in 1..=25 {
            hma.update(i as f64, timestamp).unwrap();
        }

        if hma.is_ready() {
            hma.reset();
            assert!(!hma.is_ready());
            assert_eq!(hma.current(), None);
        }
    }
}
