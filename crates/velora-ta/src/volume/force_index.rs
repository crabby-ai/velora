//! Force Index
//!
//! Force Index combines price change and volume to measure the force behind price movements.
//!
//! Formula:
//! Force Index = (Close - Previous Close) * Volume
//! Often smoothed with EMA(Force Index, period)
//!
//! Positive values indicate buying force, negative indicate selling force.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator, VolumeIndicator},
    trend::EMA,
    IndicatorError, IndicatorResult,
};

/// Force Index indicator.
#[derive(Debug, Clone)]
pub struct ForceIndex {
    period: usize,
    previous_close: Option<f64>,
    ema: EMA,
    name: String,
}

impl ForceIndex {
    /// Creates a new Force Index indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(ForceIndex {
            period,
            previous_close: None,
            ema: EMA::new(period)?,
            name: format!("ForceIndex({period})"),
        })
    }

    /// Get current Force Index value.
    pub fn current(&self) -> Option<f64> {
        self.ema.current()
    }
}

impl Indicator for ForceIndex {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.ema.is_ready()
    }

    fn reset(&mut self) {
        self.previous_close = None;
        self.ema.reset();
    }
}

impl VolumeIndicator for ForceIndex {
    fn update_with_volume(
        &mut self,
        price: f64,
        volume: f64,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be a finite number".to_string(),
            ));
        }

        if !volume.is_finite() || volume < 0.0 {
            return Err(IndicatorError::InvalidInput(
                "Volume must be a finite non-negative number".to_string(),
            ));
        }

        if let Some(prev_close) = self.previous_close {
            let raw_force = (price - prev_close) * volume;
            self.ema.update(raw_force.abs(), timestamp)?;
        }

        self.previous_close = Some(price);
        Ok(self.ema.current())
    }

    fn calculate_with_volume(
        &self,
        prices: &[f64],
        volumes: &[f64],
    ) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.len() != volumes.len() {
            return Err(IndicatorError::InvalidParameter(
                "Prices and volumes must have the same length".to_string(),
            ));
        }

        let mut fi = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for i in 0..prices.len() {
            result.push(fi.update_with_volume(prices[i], volumes[i], timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_force_index_creation() {
        let fi = ForceIndex::new(13).unwrap();
        assert_eq!(fi.name(), "ForceIndex(13)");
        assert!(!fi.is_ready());
    }
}
