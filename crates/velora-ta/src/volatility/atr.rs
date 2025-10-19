//! Average True Range (ATR)
//!
//! ATR is a volatility indicator that measures the average of true ranges over a period.
//! It's commonly used for setting stop-losses and position sizing.
//!
//! Formula:
//! ATR = Moving Average of True Range (typically using Wilder's smoothing / SMMA)
//!
//! Higher ATR values indicate higher volatility, lower values indicate lower volatility.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::SMMA,
    types::OhlcBar,
    volatility::TrueRange,
    IndicatorError, IndicatorResult,
};

/// Average True Range indicator.
///
/// Measures average volatility over a period using True Range.
/// Commonly used with a 14-period setting.
///
/// # Examples
///
/// ```
/// use velora_ta::{ATR, SingleIndicator};
/// use velora_ta::types::OhlcBar;
/// use chrono::Utc;
///
/// let mut atr = ATR::new(14).unwrap();
/// let timestamp = Utc::now();
///
/// let bar = OhlcBar::new(100.0, 105.0, 95.0, 102.0);
/// if let Some(value) = atr.update_ohlc(&bar, timestamp).unwrap() {
///     println!("ATR(14): {:.2}", value);
///     println!("Volatility is {} points", value);
/// }
/// ```
#[derive(Debug, Clone)]
pub struct ATR {
    period: usize,
    tr: TrueRange,
    smma: SMMA,
    name: String,
}

impl ATR {
    /// Creates a new ATR indicator.
    ///
    /// # Arguments
    ///
    /// * `period` - Number of periods for smoothing (typically 14, must be > 0)
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

        Ok(ATR {
            period,
            tr: TrueRange::new(),
            smma: SMMA::new(period)?,
            name: format!("ATR({period})"),
        })
    }

    /// Update the indicator with OHLC data.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        // Calculate true range for this bar
        if let Some(tr_value) = self.tr.update_ohlc(bar, timestamp)? {
            // Feed TR to SMMA
            return self.smma.update(tr_value, timestamp);
        }

        Ok(None)
    }
}

impl Indicator for ATR {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.smma.is_ready()
    }

    fn reset(&mut self) {
        self.tr.reset();
        self.smma.reset();
    }
}

impl SingleIndicator for ATR {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        Err(IndicatorError::NotInitialized(
            "ATR requires OHLC data. Use update_ohlc() instead.".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        self.smma.current()
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Err(IndicatorError::NotInitialized(
            "ATR requires OHLC data. Use calculate_ohlc() instead.".to_string(),
        ))
    }
}

impl ATR {
    /// Calculate ATR values for historical OHLC data (batch mode).
    pub fn calculate_ohlc(&self, bars: &[OhlcBar]) -> IndicatorResult<Vec<Option<f64>>> {
        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut atr = Self::new(self.period)?;
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for bar in bars {
            result.push(atr.update_ohlc(bar, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atr_creation() {
        let atr = ATR::new(14).unwrap();
        assert_eq!(atr.warmup_period(), 14);
        assert!(!atr.is_ready());
        assert_eq!(atr.name(), "ATR(14)");
    }

    #[test]
    fn test_atr_invalid_period() {
        assert!(ATR::new(0).is_err());
    }

    #[test]
    fn test_atr_calculation() {
        let mut atr = ATR::new(5).unwrap();
        let timestamp = Utc::now();

        // Create bars with consistent volatility
        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = OhlcBar::new(price, price + 10.0, price - 10.0, price + 5.0);
            atr.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(value) = atr.current() {
            // ATR should be around 20 (high-low range)
            assert!(value > 15.0 && value < 25.0);
        }
    }

    #[test]
    fn test_atr_increases_with_volatility() {
        let mut atr = ATR::new(5).unwrap();
        let timestamp = Utc::now();

        // Low volatility period
        for i in 0..10 {
            let price = 100.0 + i as f64 * 0.5;
            let bar = OhlcBar::new(price, price + 1.0, price - 1.0, price + 0.5);
            atr.update_ohlc(&bar, timestamp).unwrap();
        }

        let low_vol_atr = atr.current().unwrap();

        // High volatility period
        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = OhlcBar::new(price, price + 10.0, price - 10.0, price + 5.0);
            atr.update_ohlc(&bar, timestamp).unwrap();
        }

        let high_vol_atr = atr.current().unwrap();

        // ATR should be higher with higher volatility
        assert!(high_vol_atr > low_vol_atr);
    }

    #[test]
    fn test_atr_batch_calculation() {
        let atr = ATR::new(5).unwrap();

        let bars: Vec<OhlcBar> = (1..=15)
            .map(|i| {
                let price = i as f64 * 10.0;
                OhlcBar::new(price, price + 5.0, price - 5.0, price + 2.0)
            })
            .collect();

        let values = atr.calculate_ohlc(&bars).unwrap();
        assert_eq!(values.len(), 15);

        // ATR uses SMMA which needs warmup period before outputting
        // After warmup (period=5), should have stable ATR values
        assert!(values[4].is_some() || values[5].is_some());
        assert!(values[14].is_some());
    }

    #[test]
    fn test_atr_reset() {
        let mut atr = ATR::new(5).unwrap();
        let timestamp = Utc::now();

        for i in 1..=10 {
            let bar = OhlcBar::new(i as f64, i as f64 + 1.0, i as f64 - 1.0, i as f64);
            atr.update_ohlc(&bar, timestamp).unwrap();
        }

        assert!(atr.is_ready());

        atr.reset();
        assert!(!atr.is_ready());
    }
}
