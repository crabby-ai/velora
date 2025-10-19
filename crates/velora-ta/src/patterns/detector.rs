//! Pattern detector trait and types.

use crate::types::OhlcBar;

/// Signal from a pattern detection.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatternSignal {
    /// Bullish pattern detected
    Bullish,
    /// Bearish pattern detected
    Bearish,
    /// Neutral pattern detected
    Neutral,
}

/// Trait for candlestick pattern detectors.
pub trait PatternDetector: Send + Sync {
    /// Pattern name.
    fn name(&self) -> &str;
    /// Detect pattern in candles.
    fn detect(&self, candles: &[OhlcBar]) -> Option<PatternSignal>;
}
