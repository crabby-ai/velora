//! Alligator Indicator

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::SMMA,
    types::{MultiIndicatorValue, OhlcBar},
    IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone)]
pub struct Alligator {
    jaw: SMMA,   // 13-period SMMA
    teeth: SMMA, // 8-period SMMA
    lips: SMMA,  // 5-period SMMA
    name: String,
}

impl Alligator {
    pub fn new() -> IndicatorResult<Self> {
        Ok(Alligator {
            jaw: SMMA::new(13)?,
            teeth: SMMA::new(8)?,
            lips: SMMA::new(5)?,
            name: "Alligator".to_string(),
        })
    }

    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        let median = (bar.high + bar.low) / 2.0;

        let jaw_val = self.jaw.update(median, timestamp)?;
        let teeth_val = self.teeth.update(median, timestamp)?;
        let lips_val = self.lips.update(median, timestamp)?;

        match (jaw_val, teeth_val, lips_val) {
            (Some(j), Some(t), Some(l)) => Ok(Some(vec![j, t, l])),
            _ => Ok(None),
        }
    }
}

impl Default for Alligator {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Indicator for Alligator {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        13
    }

    fn is_ready(&self) -> bool {
        self.jaw.is_ready()
    }

    fn reset(&mut self) {
        self.jaw.reset();
        self.teeth.reset();
        self.lips.reset();
    }
}

impl MultiIndicator for Alligator {
    fn output_count(&self) -> usize {
        3
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["Jaw", "Teeth", "Lips"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Alligator requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        match (
            self.jaw.current(),
            self.teeth.current(),
            self.lips.current(),
        ) {
            (Some(j), Some(t), Some(l)) => Some(vec![j, t, l]),
            _ => None,
        }
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "Alligator requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alligator_creation() {
        let alligator = Alligator::new().unwrap();
        assert_eq!(alligator.output_count(), 3);
    }
}
