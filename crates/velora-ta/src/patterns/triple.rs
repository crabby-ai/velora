//! Three-candle patterns

use crate::{
    patterns::detector::{PatternDetector, PatternSignal},
    types::OhlcBar,
};

#[derive(Debug, Clone)]
pub struct ThreeWhiteSoldiers;

impl ThreeWhiteSoldiers {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThreeWhiteSoldiers {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for ThreeWhiteSoldiers {
    fn name(&self) -> &str {
        "Three White Soldiers"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.len() < 3 {
            return None;
        }

        let c1 = &candles[candles.len() - 3];
        let c2 = &candles[candles.len() - 2];
        let c3 = &candles[candles.len() - 1];

        // All three are bullish
        if c1.close > c1.open && c2.close > c2.open && c3.close > c3.open {
            // Each closes higher than previous
            if c2.close > c1.close && c3.close > c2.close {
                return Some(PatternSignal::Bullish);
            }
        }

        None
    }
}

#[derive(Debug, Clone)]
pub struct ThreeBlackCrows;

impl ThreeBlackCrows {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ThreeBlackCrows {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for ThreeBlackCrows {
    fn name(&self) -> &str {
        "Three Black Crows"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.len() < 3 {
            return None;
        }

        let c1 = &candles[candles.len() - 3];
        let c2 = &candles[candles.len() - 2];
        let c3 = &candles[candles.len() - 1];

        // All three are bearish
        if c1.close < c1.open && c2.close < c2.open && c3.close < c3.open {
            // Each closes lower than previous
            if c2.close < c1.close && c3.close < c2.close {
                return Some(PatternSignal::Bearish);
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_three_white_soldiers() {
        let pattern = ThreeWhiteSoldiers::new();
        assert_eq!(pattern.name(), "Three White Soldiers");
    }
}
