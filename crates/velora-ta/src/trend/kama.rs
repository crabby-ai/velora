//! Kaufman's Adaptive Moving Average (KAMA)
//!
//! KAMA adapts to market volatility, moving faster in trending markets
//! and slower in ranging markets.

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

/// Kaufman's Adaptive Moving Average.
#[derive(Debug, Clone)]
pub struct KAMA {
    period: usize,
    fast_period: usize,
    slow_period: usize,
    buffer: CircularBuffer<f64>,
    kama_value: Option<f64>,
    name: String,
}

impl KAMA {
    /// Creates a new KAMA indicator.
    pub fn new(period: usize, fast_period: usize, slow_period: usize) -> IndicatorResult<Self> {
        if period == 0 || fast_period == 0 || slow_period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "All periods must be > 0".to_string(),
            ));
        }

        Ok(KAMA {
            period,
            fast_period,
            slow_period,
            buffer: CircularBuffer::new(period + 1),
            kama_value: None,
            name: format!("KAMA({period},{fast_period},{slow_period})"),
        })
    }
}

impl Indicator for KAMA {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1
    }

    fn is_ready(&self) -> bool {
        self.kama_value.is_some()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.kama_value = None;
    }
}

impl SingleIndicator for KAMA {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be finite".to_string(),
            ));
        }

        self.buffer.push(price);

        if !self.buffer.is_full() {
            return Ok(None);
        }

        if self.kama_value.is_none() {
            self.kama_value = Some(price);
            return Ok(Some(price));
        }

        // Calculate ER (Efficiency Ratio)
        let change = (price - self.buffer.first().unwrap()).abs();
        let mut volatility = 0.0;

        for i in 1..self.buffer.len() {
            volatility += (self.buffer.get(i).unwrap() - self.buffer.get(i - 1).unwrap()).abs();
        }

        let er = if volatility == 0.0 {
            1.0
        } else {
            change / volatility
        };

        // Calculate SC (Smoothing Constant)
        let fast_sc = 2.0 / (self.fast_period + 1) as f64;
        let slow_sc = 2.0 / (self.slow_period + 1) as f64;
        let sc = er * (fast_sc - slow_sc) + slow_sc;
        let sc_squared = sc * sc;

        // Update KAMA
        let prev_kama = self.kama_value.unwrap();
        let new_kama = prev_kama + sc_squared * (price - prev_kama);
        self.kama_value = Some(new_kama);

        Ok(Some(new_kama))
    }

    fn current(&self) -> Option<f64> {
        self.kama_value
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut kama = Self::new(self.period, self.fast_period, self.slow_period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(kama.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kama_creation() {
        let kama = KAMA::new(10, 2, 30).unwrap();
        assert_eq!(kama.name(), "KAMA(10,2,30)");
    }
}
