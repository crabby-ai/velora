//! Accumulation/Distribution (AD)
//!
//! AD line measures the cumulative flow of money into and out of a security.
//! It uses price location within the bar's range and volume.
//!
//! Formula:
//! Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
//! Money Flow Volume = Money Flow Multiplier * Volume
//! AD = AD_prev + Money Flow Volume
//!
//! Rising AD confirms uptrends, falling AD confirms downtrends.

use chrono::{DateTime, Utc};

use crate::{traits::Indicator, types::OhlcBar, IndicatorError, IndicatorResult};

/// Accumulation/Distribution indicator.
///
/// Cumulative indicator that combines price and volume to show
/// money flow into or out of a security.
#[derive(Debug, Clone)]
pub struct AD {
    ad_value: f64,
    count: usize,
    name: String,
}

impl AD {
    /// Creates a new AD indicator.
    pub fn new() -> Self {
        AD {
            ad_value: 0.0,
            count: 0,
            name: "AD".to_string(),
        }
    }

    /// Update AD with OHLC bar and volume.
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

        let range = bar.high - bar.low;

        let money_flow_multiplier = if range == 0.0 {
            0.0 // No range = no money flow
        } else {
            ((bar.close - bar.low) - (bar.high - bar.close)) / range
        };

        let money_flow_volume = money_flow_multiplier * volume;
        self.ad_value += money_flow_volume;
        self.count += 1;

        Ok(Some(self.ad_value))
    }

    /// Get current AD value.
    pub fn current(&self) -> Option<f64> {
        if self.count > 0 {
            Some(self.ad_value)
        } else {
            None
        }
    }
}

impl Default for AD {
    fn default() -> Self {
        Self::new()
    }
}

impl Indicator for AD {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.count >= 1
    }

    fn reset(&mut self) {
        self.ad_value = 0.0;
        self.count = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ad_creation() {
        let ad = AD::new();
        assert_eq!(ad.name(), "AD");
        assert!(!ad.is_ready());
    }

    #[test]
    fn test_ad_bullish_close() {
        let mut ad = AD::new();
        let timestamp = Utc::now();

        // Close near high -> positive money flow
        let bar = OhlcBar::new(100.0, 110.0, 90.0, 108.0);
        // MFM = ((108-90) - (110-108)) / (110-90) = (18-2)/20 = 0.8
        // MFV = 0.8 * 1000 = 800
        ad.update_ohlc(&bar, 1000.0, timestamp).unwrap();

        let value = ad.current().unwrap();
        assert_eq!(value, 800.0);
    }

    #[test]
    fn test_ad_bearish_close() {
        let mut ad = AD::new();
        let timestamp = Utc::now();

        // Close near low -> negative money flow
        let bar = OhlcBar::new(100.0, 110.0, 90.0, 92.0);
        // MFM = ((92-90) - (110-92)) / (110-90) = (2-18)/20 = -0.8
        // MFV = -0.8 * 1000 = -800
        ad.update_ohlc(&bar, 1000.0, timestamp).unwrap();

        let value = ad.current().unwrap();
        assert_eq!(value, -800.0);
    }

    #[test]
    fn test_ad_cumulative() {
        let mut ad = AD::new();
        let timestamp = Utc::now();

        let bar1 = OhlcBar::new(100.0, 110.0, 90.0, 108.0);
        ad.update_ohlc(&bar1, 1000.0, timestamp).unwrap(); // +800

        let bar2 = OhlcBar::new(108.0, 115.0, 105.0, 112.0);
        ad.update_ohlc(&bar2, 1500.0, timestamp).unwrap();

        // Should be cumulative
        let value = ad.current().unwrap();
        assert!(value > 800.0); // Should have accumulated
    }

    #[test]
    fn test_ad_reset() {
        let mut ad = AD::new();
        let timestamp = Utc::now();

        let bar = OhlcBar::new(100.0, 110.0, 90.0, 108.0);
        ad.update_ohlc(&bar, 1000.0, timestamp).unwrap();

        assert!(ad.is_ready());

        ad.reset();
        assert!(!ad.is_ready());
        assert_eq!(ad.ad_value, 0.0);
    }
}
