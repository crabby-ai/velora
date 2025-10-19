//! Keltner Channels

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::EMA,
    types::{MultiIndicatorValue, OhlcBar},
    volatility::ATR,
    IndicatorError, IndicatorResult,
};

/// Keltner Channels indicator.
#[derive(Debug, Clone)]
pub struct KeltnerChannels {
    period: usize,
    multiplier: f64,
    ema: EMA,
    atr: ATR,
    name: String,
}

impl KeltnerChannels {
    /// Creates a new Keltner Channels indicator.
    pub fn new(period: usize, multiplier: f64) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be > 0".to_string(),
            ));
        }

        Ok(KeltnerChannels {
            period,
            multiplier,
            ema: EMA::new(period)?,
            atr: ATR::new(period)?,
            name: format!("Keltner({period},{multiplier:.1})"),
        })
    }

    /// Update Keltner Channels with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        let middle = self.ema.update(bar.close, timestamp)?;
        let atr_val = self.atr.update_ohlc(bar, timestamp)?;

        match (middle, atr_val) {
            (Some(mid), Some(atr)) => {
                let upper = mid + (self.multiplier * atr);
                let lower = mid - (self.multiplier * atr);
                Ok(Some(vec![upper, mid, lower]))
            }
            _ => Ok(None),
        }
    }
}

impl Indicator for KeltnerChannels {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready() && self.atr.is_ready()
    }

    fn reset(&mut self) {
        self.ema.reset();
        self.atr.reset();
    }
}

impl MultiIndicator for KeltnerChannels {
    fn output_count(&self) -> usize {
        3
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["Upper", "Middle", "Lower"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Keltner requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        match (self.ema.current(), self.atr.current()) {
            (Some(mid), Some(atr)) => {
                let upper = mid + (self.multiplier * atr);
                let lower = mid - (self.multiplier * atr);
                Some(vec![upper, mid, lower])
            }
            _ => None,
        }
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "Keltner requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keltner_creation() {
        let keltner = KeltnerChannels::new(20, 2.0).unwrap();
        assert_eq!(keltner.output_count(), 3);
    }
}
