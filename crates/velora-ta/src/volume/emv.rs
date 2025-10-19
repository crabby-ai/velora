//! Ease of Movement (EMV)
//!
//! EMV relates price change to volume, showing how easily price moves.
//!
//! Formula:
//! Distance Moved = ((High + Low)/2 today - (High + Low)/2 yesterday)
//! Box Ratio = (Volume / scale) / (High - Low)
//! EMV = Distance Moved / Box Ratio
//!
//! High EMV = price moves easily with little volume
//! Low EMV = price requires high volume to move

use chrono::{DateTime, Utc};

use crate::{traits::Indicator, types::OhlcBar, IndicatorError, IndicatorResult};

/// Ease of Movement indicator.
#[derive(Debug, Clone)]
pub struct EMV {
    previous_midpoint: Option<f64>,
    scale: f64,
    name: String,
}

impl EMV {
    /// Creates a new EMV indicator.
    pub fn new(scale: f64) -> IndicatorResult<Self> {
        if scale <= 0.0 {
            return Err(IndicatorError::InvalidParameter(
                "Scale must be positive".to_string(),
            ));
        }

        Ok(EMV {
            previous_midpoint: None,
            scale,
            name: format!("EMV({scale:.0})"),
        })
    }

    /// Update EMV with OHLC bar and volume.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        let midpoint = (bar.high + bar.low) / 2.0;
        let range = bar.high - bar.low;

        if let Some(prev_mid) = self.previous_midpoint {
            if range == 0.0 || volume == 0.0 {
                self.previous_midpoint = Some(midpoint);
                return Ok(Some(0.0));
            }

            let distance = midpoint - prev_mid;
            let box_ratio = (volume / self.scale) / range;
            let emv = distance / box_ratio;

            self.previous_midpoint = Some(midpoint);
            Ok(Some(emv))
        } else {
            self.previous_midpoint = Some(midpoint);
            Ok(None)
        }
    }

    /// Get current EMV value.
    pub fn current(&self) -> Option<f64> {
        None // EMV is calculated per bar, no persistent current value
    }
}

impl Indicator for EMV {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        2
    }

    fn is_ready(&self) -> bool {
        self.previous_midpoint.is_some()
    }

    fn reset(&mut self) {
        self.previous_midpoint = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emv_creation() {
        let emv = EMV::new(10000.0).unwrap();
        assert_eq!(emv.name(), "EMV(10000)");
    }
}
