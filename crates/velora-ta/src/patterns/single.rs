//! Single candle patterns

use crate::{
    patterns::detector::{PatternDetector, PatternSignal},
    types::OhlcBar,
};

#[derive(Debug, Clone)]
pub struct Doji {
    body_percent: f64,
}

impl Doji {
    pub fn new() -> Self {
        Self { body_percent: 0.1 }
    }
}

impl Default for Doji {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for Doji {
    fn name(&self) -> &str {
        "Doji"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.is_empty() {
            return None;
        }

        let candle = &candles[candles.len() - 1];
        let body = (candle.close - candle.open).abs();
        let range = candle.range();

        if range == 0.0 {
            return Some(PatternSignal::Neutral);
        }

        if body / range < self.body_percent {
            Some(PatternSignal::Neutral)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Hammer;

impl Hammer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for Hammer {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for Hammer {
    fn name(&self) -> &str {
        "Hammer"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.is_empty() {
            return None;
        }

        let candle = &candles[candles.len() - 1];
        let body = (candle.close - candle.open).abs();
        let upper_shadow = candle.high - candle.close.max(candle.open);
        let lower_shadow = candle.close.min(candle.open) - candle.low;
        let range = candle.range();

        if range == 0.0 {
            return None;
        }

        // Hammer: small body, small upper shadow, long lower shadow
        if lower_shadow > 2.0 * body && upper_shadow < body {
            Some(PatternSignal::Bullish)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct ShootingStar;

impl ShootingStar {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ShootingStar {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for ShootingStar {
    fn name(&self) -> &str {
        "Shooting Star"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.is_empty() {
            return None;
        }

        let candle = &candles[candles.len() - 1];
        let body = (candle.close - candle.open).abs();
        let upper_shadow = candle.high - candle.close.max(candle.open);
        let lower_shadow = candle.close.min(candle.open) - candle.low;

        // Shooting Star: small body, long upper shadow, small lower shadow
        if upper_shadow > 2.0 * body && lower_shadow < body {
            Some(PatternSignal::Bearish)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_doji() {
        let doji = Doji::new();
        let candles = vec![OhlcBar::new(100.0, 101.0, 99.0, 100.1)];
        assert_eq!(doji.detect(&candles), Some(PatternSignal::Neutral));
    }
}
