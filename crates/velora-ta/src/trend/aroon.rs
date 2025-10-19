//! Aroon Indicator
//!
//! Aroon identifies when trends are likely to change direction.
//! Outputs Aroon Up and Aroon Down.
//!
//! Formula:
//! Aroon Up = ((period - periods since period high) / period) × 100
//! Aroon Down = ((period - periods since period low) / period) × 100

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator},
    types::{MultiIndicatorValue, OhlcBar},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Aroon indicator for identifying trend changes.
#[derive(Debug, Clone)]
pub struct Aroon {
    period: usize,
    high_buffer: CircularBuffer<f64>,
    low_buffer: CircularBuffer<f64>,
    name: String,
}

impl Aroon {
    /// Creates a new Aroon indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be > 0".to_string(),
            ));
        }

        Ok(Aroon {
            period,
            high_buffer: CircularBuffer::new(period + 1),
            low_buffer: CircularBuffer::new(period + 1),
            name: format!("Aroon({period})"),
        })
    }

    /// Update Aroon with OHLC bar.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        self.high_buffer.push(bar.high);
        self.low_buffer.push(bar.low);

        if !self.is_ready() {
            return Ok(None);
        }

        // Find periods since high/low
        let mut periods_since_high = 0;
        let mut periods_since_low = 0;
        let mut max_high = f64::NEG_INFINITY;
        let mut min_low = f64::INFINITY;

        for (i, (&high, &low)) in self
            .high_buffer
            .iter()
            .zip(self.low_buffer.iter())
            .enumerate()
        {
            if high >= max_high {
                max_high = high;
                periods_since_high = self.high_buffer.len() - 1 - i;
            }
            if low <= min_low {
                min_low = low;
                periods_since_low = self.low_buffer.len() - 1 - i;
            }
        }

        let aroon_up = ((self.period - periods_since_high) as f64 / self.period as f64) * 100.0;
        let aroon_down = ((self.period - periods_since_low) as f64 / self.period as f64) * 100.0;

        Ok(Some(vec![aroon_up, aroon_down]))
    }
}

impl Indicator for Aroon {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.high_buffer.is_full()
    }

    fn reset(&mut self) {
        self.high_buffer.clear();
        self.low_buffer.clear();
    }
}

impl MultiIndicator for Aroon {
    fn output_count(&self) -> usize {
        2
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["Aroon Up", "Aroon Down"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Aroon requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        None
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "Aroon requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aroon_creation() {
        let aroon = Aroon::new(25).unwrap();
        assert_eq!(aroon.output_names(), vec!["Aroon Up", "Aroon Down"]);
    }
}
