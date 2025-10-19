//! Z-Score

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone)]
pub struct ZScore {
    period: usize,
    buffer: CircularBuffer<f64>,
    name: String,
}

impl ZScore {
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period < 2 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be >= 2".to_string(),
            ));
        }

        Ok(ZScore {
            period,
            buffer: CircularBuffer::new(period),
            name: format!("ZScore({period})"),
        })
    }

    fn calculate_zscore(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let mean = self.buffer.mean()?;
        let std_dev = self.buffer.std_dev()?;
        let current = self.buffer.last()?;

        if std_dev == 0.0 {
            return Some(0.0);
        }

        Some((current - mean) / std_dev)
    }
}

impl Indicator for ZScore {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full()
    }

    fn reset(&mut self) {
        self.buffer.clear();
    }
}

impl SingleIndicator for ZScore {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be finite".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_zscore())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_zscore()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut zscore = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(zscore.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_zscore_creation() {
        let zscore = ZScore::new(20).unwrap();
        assert_eq!(zscore.name(), "ZScore(20)");
    }
}
