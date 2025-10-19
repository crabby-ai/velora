//! Average Directional Index (ADX)
//!
//! ADX measures trend strength without regard to direction.
//! It's derived from the Directional Movement Index (DMI).
//!
//! Components:
//! - +DI (Plus Directional Indicator) - measures upward movement
//! - -DI (Minus Directional Indicator) - measures downward movement
//! - ADX - smoothed average of the difference between +DI and -DI
//!
//! ADX > 25 indicates strong trend, < 20 indicates weak/no trend.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::SMMA,
    types::{MultiIndicatorValue, OhlcBar},
    volatility::TrueRange,
    IndicatorError, IndicatorResult,
};

/// ADX (Average Directional Index) indicator.
///
/// Outputs three values: ADX, +DI, -DI
#[derive(Debug, Clone)]
pub struct ADX {
    period: usize,
    tr: TrueRange,
    previous_high: Option<f64>,
    previous_low: Option<f64>,
    smma_tr: SMMA,
    smma_plus_dm: SMMA,
    smma_minus_dm: SMMA,
    smma_dx: SMMA,
    name: String,
}

impl ADX {
    /// Creates a new ADX indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(ADX {
            period,
            tr: TrueRange::new(),
            previous_high: None,
            previous_low: None,
            smma_tr: SMMA::new(period)?,
            smma_plus_dm: SMMA::new(period)?,
            smma_minus_dm: SMMA::new(period)?,
            smma_dx: SMMA::new(period)?,
            name: format!("ADX({period})"),
        })
    }

    /// Update ADX with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        // Calculate True Range
        if let Some(tr_val) = self.tr.update_ohlc(bar, timestamp)? {
            self.smma_tr.update(tr_val, timestamp)?;
        }

        // Calculate Directional Movement
        if let (Some(prev_high), Some(prev_low)) = (self.previous_high, self.previous_low) {
            let plus_dm = (bar.high - prev_high).max(0.0);
            let minus_dm = (prev_low - bar.low).max(0.0);

            // Only one can be non-zero
            let (plus_dm, minus_dm) = if plus_dm > minus_dm {
                (plus_dm, 0.0)
            } else if minus_dm > plus_dm {
                (0.0, minus_dm)
            } else {
                (0.0, 0.0)
            };

            self.smma_plus_dm.update(plus_dm, timestamp)?;
            self.smma_minus_dm.update(minus_dm, timestamp)?;

            // Calculate DI values
            if let Some(atr) = self.smma_tr.current() {
                if atr > 0.0 {
                    let plus_dm_smooth = self.smma_plus_dm.current().unwrap_or(0.0);
                    let minus_dm_smooth = self.smma_minus_dm.current().unwrap_or(0.0);

                    let plus_di = (plus_dm_smooth / atr) * 100.0;
                    let minus_di = (minus_dm_smooth / atr) * 100.0;

                    // Calculate DX
                    let di_sum = plus_di + minus_di;
                    if di_sum > 0.0 {
                        let dx = ((plus_di - minus_di).abs() / di_sum) * 100.0;
                        self.smma_dx.update(dx, timestamp)?;

                        if let Some(adx) = self.smma_dx.current() {
                            self.previous_high = Some(bar.high);
                            self.previous_low = Some(bar.low);
                            return Ok(Some(vec![adx, plus_di, minus_di]));
                        }
                    }
                }
            }
        }

        self.previous_high = Some(bar.high);
        self.previous_low = Some(bar.low);
        Ok(None)
    }
}

impl Indicator for ADX {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        2 * self.period
    }

    fn is_ready(&self) -> bool {
        self.smma_dx.is_ready()
    }

    fn reset(&mut self) {
        self.tr.reset();
        self.previous_high = None;
        self.previous_low = None;
        self.smma_tr.reset();
        self.smma_plus_dm.reset();
        self.smma_minus_dm.reset();
        self.smma_dx.reset();
    }
}

impl MultiIndicator for ADX {
    fn output_count(&self) -> usize {
        3 // ADX, +DI, -DI
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["ADX", "+DI", "-DI"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "ADX requires OHLC data. Use update_ohlc() instead.".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        None // ADX calculates per bar
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "ADX requires OHLC data. Use calculate_ohlc() instead.".to_string(),
        ))
    }
}

impl ADX {
    /// Calculate ADX for OHLC bars.
    pub fn calculate_ohlc(
        &self,
        bars: &[OhlcBar],
    ) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        if bars.is_empty() {
            return Ok(Vec::new());
        }

        let mut adx = Self::new(self.period)?;
        let mut result = Vec::with_capacity(bars.len());
        let timestamp = Utc::now();

        for bar in bars {
            result.push(
                adx.update_ohlc(bar, timestamp)?
                    .map(MultiIndicatorValue::from),
            );
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adx_creation() {
        let adx = ADX::new(14).unwrap();
        assert_eq!(adx.output_count(), 3);
        assert_eq!(adx.output_names(), vec!["ADX", "+DI", "-DI"]);
        assert!(!adx.is_ready());
    }

    #[test]
    fn test_adx_strong_trend() {
        let mut adx = ADX::new(5).unwrap();
        let timestamp = Utc::now();

        // Strong uptrend
        for i in 0..20 {
            let price = 100.0 + i as f64 * 5.0;
            let bar = OhlcBar::new(price, price + 10.0, price, price + 8.0);
            adx.update_ohlc(&bar, timestamp).unwrap();
        }

        if let Some(values) = adx
            .update_ohlc(&OhlcBar::new(200.0, 210.0, 200.0, 208.0), timestamp)
            .unwrap()
        {
            let adx_val = values[0];
            assert!(adx_val > 0.0);
        }
    }
}
