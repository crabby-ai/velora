//! Fractals Indicator

use crate::{types::OhlcBar, IndicatorResult};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FractalType {
    Up,
    Down,
    None,
}

#[derive(Debug, Clone)]
pub struct Fractals;

impl Fractals {
    pub fn new() -> Self {
        Self
    }

    pub fn detect(&self, candles: &[OhlcBar]) -> IndicatorResult<FractalType> {
        if candles.len() < 5 {
            return Ok(FractalType::None);
        }

        let c1 = &candles[candles.len() - 5];
        let c2 = &candles[candles.len() - 4];
        let c3 = &candles[candles.len() - 3]; // Middle
        let c4 = &candles[candles.len() - 2];
        let c5 = &candles[candles.len() - 1];

        // Up fractal: middle high is highest
        if c3.high > c1.high && c3.high > c2.high && c3.high > c4.high && c3.high > c5.high {
            return Ok(FractalType::Up);
        }

        // Down fractal: middle low is lowest
        if c3.low < c1.low && c3.low < c2.low && c3.low < c4.low && c3.low < c5.low {
            return Ok(FractalType::Down);
        }

        Ok(FractalType::None)
    }
}

impl Default for Fractals {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fractals_creation() {
        let _fractals = Fractals::new();
    }
}
