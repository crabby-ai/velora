//! Two-candle patterns

use crate::{
    patterns::detector::{PatternDetector, PatternSignal},
    types::OhlcBar,
};

#[derive(Debug, Clone)]
pub struct BullishEngulfing;

impl BullishEngulfing {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BullishEngulfing {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for BullishEngulfing {
    fn name(&self) -> &str {
        "Bullish Engulfing"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.len() < 2 {
            return None;
        }

        let prev = &candles[candles.len() - 2];
        let curr = &candles[candles.len() - 1];

        // Previous candle is bearish
        if prev.close >= prev.open {
            return None;
        }

        // Current candle is bullish and engulfs previous
        if curr.close > curr.open && curr.open < prev.close && curr.close > prev.open {
            Some(PatternSignal::Bullish)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct BearishEngulfing;

impl BearishEngulfing {
    pub fn new() -> Self {
        Self
    }
}

impl Default for BearishEngulfing {
    fn default() -> Self {
        Self::new()
    }
}

impl PatternDetector for BearishEngulfing {
    fn name(&self) -> &str {
        "Bearish Engulfing"
    }

    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal> {
        if candles.len() < 2 {
            return None;
        }

        let prev = &candles[candles.len() - 2];
        let curr = &candles[candles.len() - 1];

        // Previous candle is bullish
        if prev.close <= prev.open {
            return None;
        }

        // Current candle is bearish and engulfs previous
        if curr.close < curr.open && curr.open > prev.close && curr.close < prev.open {
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
    fn test_bullish_engulfing() {
        let pattern = BullishEngulfing::new();
        assert_eq!(pattern.name(), "Bullish Engulfing");
    }
}
