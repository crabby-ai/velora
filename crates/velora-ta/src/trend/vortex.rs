//! Vortex Indicator
//!
//! Identifies the start of trends and their direction using two oscillating lines.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator},
    types::{MultiIndicatorValue, OhlcBar},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone, Copy, Default)]
struct VortexPoint {
    high: f64,
    low: f64,
    close: f64,
}

/// Vortex indicator for trend identification.
#[derive(Debug, Clone)]
pub struct Vortex {
    period: usize,
    buffer: CircularBuffer<VortexPoint>,
    name: String,
}

impl Vortex {
    /// Creates a new Vortex indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be > 0".to_string(),
            ));
        }

        Ok(Vortex {
            period,
            buffer: CircularBuffer::new(period + 1),
            name: format!("Vortex({period})"),
        })
    }

    /// Update Vortex with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        self.buffer.push(VortexPoint {
            high: bar.high,
            low: bar.low,
            close: bar.close,
        });

        if !self.is_ready() {
            return Ok(None);
        }

        let mut vm_plus = 0.0;
        let mut vm_minus = 0.0;
        let mut tr_sum = 0.0;

        for i in 1..self.buffer.len() {
            let curr = self.buffer.get(i).unwrap();
            let prev = self.buffer.get(i - 1).unwrap();

            vm_plus += (curr.high - prev.low).abs();
            vm_minus += (curr.low - prev.high).abs();
            tr_sum += (curr.high - curr.low)
                .max((curr.high - prev.close).abs())
                .max((curr.low - prev.close).abs());
        }

        if tr_sum == 0.0 {
            return Ok(Some(vec![0.0, 0.0]));
        }

        let vi_plus = vm_plus / tr_sum;
        let vi_minus = vm_minus / tr_sum;

        Ok(Some(vec![vi_plus, vi_minus]))
    }
}

impl Indicator for Vortex {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full()
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl MultiIndicator for Vortex {
    fn output_count(&self) -> usize {
        2
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["VI+", "VI-"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Vortex requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        None
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "Vortex requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vortex_creation() {
        let vortex = Vortex::new(14).unwrap();
        assert_eq!(vortex.output_count(), 2);
    }
}
