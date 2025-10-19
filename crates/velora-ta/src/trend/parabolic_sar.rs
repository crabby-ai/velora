//! Parabolic SAR (Stop and Reverse)
//!
//! Parabolic SAR provides potential entry and exit points based on price momentum.
//! The indicator appears as dots above or below price, flipping when trend reverses.
//!
//! When SAR is below price = uptrend, when above = downtrend.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    types::OhlcBar,
    IndicatorError, IndicatorResult,
};

/// Parabolic SAR (Stop and Reverse) indicator.
#[derive(Debug, Clone)]
pub struct ParabolicSAR {
    af_start: f64,
    af_increment: f64,
    af_max: f64,
    is_long: bool,
    sar: Option<f64>,
    ep: f64, // Extreme Point
    af: f64, // Acceleration Factor
    name: String,
}

impl ParabolicSAR {
    /// Creates a new Parabolic SAR indicator.
    pub fn new(af_start: f64, af_increment: f64, af_max: f64) -> IndicatorResult<Self> {
        if af_start <= 0.0 || af_increment <= 0.0 || af_max <= af_start {
            return Err(IndicatorError::InvalidParameter(
                "Invalid acceleration factor parameters".to_string(),
            ));
        }

        Ok(ParabolicSAR {
            af_start,
            af_increment,
            af_max,
            is_long: true,
            sar: None,
            ep: 0.0,
            af: af_start,
            name: "ParabolicSAR".to_string(),
        })
    }

    /// Update Parabolic SAR with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if self.sar.is_none() {
            // Initialize
            self.sar = Some(bar.low);
            self.ep = bar.high;
            self.is_long = true;
            return Ok(Some(bar.low));
        }

        let mut sar = self.sar.unwrap();
        sar += self.af * (self.ep - sar);

        if self.is_long {
            if bar.low < sar {
                // Reverse to short
                self.is_long = false;
                sar = self.ep;
                self.ep = bar.low;
                self.af = self.af_start;
            } else if bar.high > self.ep {
                self.ep = bar.high;
                self.af = (self.af + self.af_increment).min(self.af_max);
            }
        } else if bar.high > sar {
            // Reverse to long
            self.is_long = true;
            sar = self.ep;
            self.ep = bar.high;
            self.af = self.af_start;
        } else if bar.low < self.ep {
            self.ep = bar.low;
            self.af = (self.af + self.af_increment).min(self.af_max);
        }

        self.sar = Some(sar);
        Ok(Some(sar))
    }
}

impl Indicator for ParabolicSAR {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        1
    }

    fn is_ready(&self) -> bool {
        self.sar.is_some()
    }

    fn reset(&mut self) {
        self.sar = None;
        self.af = self.af_start;
    }
}

impl SingleIndicator for ParabolicSAR {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        Err(IndicatorError::NotInitialized(
            "Parabolic SAR requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        self.sar
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Parabolic SAR requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_psar_creation() {
        let psar = ParabolicSAR::new(0.02, 0.02, 0.2).unwrap();
        assert_eq!(psar.name(), "ParabolicSAR");
    }
}
