//! True Strength Index (TSI)
//!
//! TSI is a momentum oscillator that shows both trend direction and momentum.
//! (Placeholder implementation)

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator, SingleIndicator},
    trend::EMA,
    types::MultiIndicatorValue,
    IndicatorError, IndicatorResult,
};

/// True Strength Index indicator (placeholder).
#[derive(Debug, Clone)]
pub struct TSI {
    long_period: usize,
    short_period: usize,
    #[allow(dead_code)]
    signal_period: usize,
    previous_price: Option<f64>,
    ema_long_momentum: EMA,
    ema_short_momentum: EMA,
    ema_long_abs_momentum: EMA,
    ema_short_abs_momentum: EMA,
    signal_ema: EMA,
    name: String,
}

impl TSI {
    /// Creates a new TSI indicator.
    pub fn new(
        long_period: usize,
        short_period: usize,
        signal_period: usize,
    ) -> IndicatorResult<Self> {
        if long_period == 0 || short_period == 0 || signal_period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "All periods must be > 0".to_string(),
            ));
        }

        Ok(TSI {
            long_period,
            short_period,
            signal_period,
            previous_price: None,
            ema_long_momentum: EMA::new(long_period)?,
            ema_short_momentum: EMA::new(short_period)?,
            ema_long_abs_momentum: EMA::new(long_period)?,
            ema_short_abs_momentum: EMA::new(short_period)?,
            signal_ema: EMA::new(signal_period)?,
            name: format!("TSI({long_period},{short_period},{signal_period})"),
        })
    }
}

impl Indicator for TSI {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.long_period + self.short_period
    }

    fn is_ready(&self) -> bool {
        self.signal_ema.is_ready()
    }

    fn reset(&mut self) {
        self.previous_price = None;
        self.ema_long_momentum.reset();
        self.ema_short_momentum.reset();
        self.ema_long_abs_momentum.reset();
        self.ema_short_abs_momentum.reset();
        self.signal_ema.reset();
    }
}

impl SingleIndicator for TSI {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be finite".to_string(),
            ));
        }

        self.previous_price = Some(price);
        Ok(None) // Simplified - would need full implementation
    }

    fn current(&self) -> Option<f64> {
        None
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        Ok(Vec::new())
    }
}

impl MultiIndicator for TSI {
    fn output_count(&self) -> usize {
        2
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["TSI", "Signal"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Ok(None)
    }

    fn current(&self) -> Option<Vec<f64>> {
        None
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Ok(Vec::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tsi_creation() {
        let tsi = TSI::new(25, 13, 13).unwrap();
        assert_eq!(tsi.output_count(), 2);
    }
}
