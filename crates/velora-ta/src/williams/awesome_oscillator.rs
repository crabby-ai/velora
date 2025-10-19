//! Awesome Oscillator

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    trend::SMA,
    types::OhlcBar,
    IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone)]
pub struct AwesomeOscillator {
    sma_fast: SMA,
    sma_slow: SMA,
    name: String,
}

impl AwesomeOscillator {
    pub fn new() -> IndicatorResult<Self> {
        Ok(AwesomeOscillator {
            sma_fast: SMA::new(5)?,
            sma_slow: SMA::new(34)?,
            name: "AO".to_string(),
        })
    }

    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        let midpoint = (bar.high + bar.low) / 2.0;
        let fast = self.sma_fast.update(midpoint, timestamp)?;
        let slow = self.sma_slow.update(midpoint, timestamp)?;

        match (fast, slow) {
            (Some(f), Some(s)) => Ok(Some(f - s)),
            _ => Ok(None),
        }
    }
}

impl Default for AwesomeOscillator {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

impl Indicator for AwesomeOscillator {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        34
    }

    fn is_ready(&self) -> bool {
        self.sma_fast.is_ready() && self.sma_slow.is_ready()
    }

    fn reset(&mut self) {
        self.sma_fast.reset();
        self.sma_slow.reset();
    }
}

impl SingleIndicator for AwesomeOscillator {
    fn update(&mut self, _price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        Err(IndicatorError::NotInitialized(
            "AO requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<f64> {
        match (self.sma_fast.current(), self.sma_slow.current()) {
            (Some(f), Some(s)) => Some(f - s),
            _ => None,
        }
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Err(IndicatorError::NotInitialized(
            "AO requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ao_creation() {
        let ao = AwesomeOscillator::new().unwrap();
        assert_eq!(ao.name(), "AO");
    }
}
