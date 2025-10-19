//! SuperTrend Indicator
//!
//! SuperTrend is a trend-following indicator based on ATR.
//! It provides buy/sell signals and acts as dynamic support/resistance.
//!
//! Formula:
//! Basic Upper Band = (High + Low) / 2 + (Multiplier × ATR)
//! Basic Lower Band = (High + Low) / 2 - (Multiplier × ATR)
//!
//! Common settings: period=10, multiplier=3.0

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    types::OhlcBar,
    volatility::ATR,
    IndicatorError, IndicatorResult,
};

/// SuperTrend indicator.
#[derive(Debug, Clone)]
pub struct SuperTrend {
    period: usize,
    multiplier: f64,
    atr: ATR,
    trend: Option<i8>, // 1 = uptrend, -1 = downtrend
    supertrend: Option<f64>,
    name: String,
}

impl SuperTrend {
    /// Creates a new SuperTrend indicator.
    pub fn new(period: usize, multiplier: f64) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be > 0".to_string(),
            ));
        }

        if multiplier <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Multiplier must be positive".to_string(),
            ));
        }

        Ok(SuperTrend {
            period,
            multiplier,
            atr: ATR::new(period)?,
            trend: None,
            supertrend: None,
            name: format!("SuperTrend({period},{multiplier:.1})"),
        })
    }

    /// Update SuperTrend with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if let Some(atr_val) = self.atr.update_ohlc(bar, timestamp)? {
            let hl_avg = (bar.high + bar.low) / 2.0;
            let upper_band = hl_avg + (self.multiplier * atr_val);
            let lower_band = hl_avg - (self.multiplier * atr_val);

            let close = bar.close;

            if self.supertrend.is_none() {
                // Initialize
                self.supertrend = Some(lower_band);
                self.trend = Some(1);
            } else {
                let prev_trend = self.trend.unwrap();

                if prev_trend == 1 {
                    // Uptrend
                    if close <= lower_band {
                        self.supertrend = Some(upper_band);
                        self.trend = Some(-1);
                    } else {
                        self.supertrend = Some(lower_band);
                    }
                } else {
                    // Downtrend
                    if close >= upper_band {
                        self.supertrend = Some(lower_band);
                        self.trend = Some(1);
                    } else {
                        self.supertrend = Some(upper_band);
                    }
                }
            }

            return Ok(self.supertrend);
        }

        Ok(None)
    }
}

impl Indicator for SuperTrend {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.atr.is_ready()
    }

    fn reset(&mut self) {
        self.atr.reset();
        self.trend = None;
        self.supertrend = None;
    }
}

impl SingleIndicator for SuperTrend {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        Err(IndicatorError::NotInitialized(
            "SuperTrend requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        self.supertrend
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Err(IndicatorError::NotInitialized(
            "SuperTrend requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supertrend_creation() {
        let st = SuperTrend::new(10, 3.0).unwrap();
        assert_eq!(st.name(), "SuperTrend(10,3.0)");
    }
}
