//! Donchian Channels

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, MultiIndicator},
    types::{MultiIndicatorValue, OhlcBar},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Donchian Channels indicator.
#[derive(Debug, Clone)]
pub struct DonchianChannels {
    period: usize,
    high_buffer: CircularBuffer<f64>,
    low_buffer: CircularBuffer<f64>,
    name: String,
}

impl DonchianChannels {
    /// Creates a new Donchian Channels indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be > 0".to_string(),
            ));
        }

        Ok(DonchianChannels {
            period,
            high_buffer: CircularBuffer::new(period),
            low_buffer: CircularBuffer::new(period),
            name: format!("Donchian({period})"),
        })
    }

    /// Update Donchian Channels with OHLC bar.
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

        let upper = self.high_buffer.max().unwrap_or(bar.high);
        let lower = self.low_buffer.min().unwrap_or(bar.low);
        let middle = (upper + lower) / 2.0;

        Ok(Some(vec![upper, middle, lower]))
    }
}

impl Indicator for DonchianChannels {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.high_buffer.is_full()
    }

    fn reset(&mut self) {
        self.high_buffer.clear();
        self.low_buffer.clear();
    }
}

impl MultiIndicator for DonchianChannels {
    fn output_count(&self) -> usize {
        3
    }

    fn output_names(&self) -> Vec<&str> {
        vec!["Upper", "Middle", "Lower"]
    }

    fn update(
        &mut self,
        _price: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<Vec<f64>>> {
        Err(IndicatorError::NotInitialized(
            "Donchian requires OHLC data".to_string(),
        ))
    }

    fn current(&self) -> Option<Vec<f64>> {
        if !self.is_ready() {
            return None;
        }

        let upper = self.high_buffer.max()?;
        let lower = self.low_buffer.min()?;
        let middle = (upper + lower) / 2.0;

        Some(vec![upper, middle, lower])
    }

    fn calculate(&self, _prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>> {
        Err(IndicatorError::NotInitialized(
            "Donchian requires OHLC data".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_donchian_creation() {
        let donchian = DonchianChannels::new(20).unwrap();
        assert_eq!(donchian.output_count(), 3);
    }
}
