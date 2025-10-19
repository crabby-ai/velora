//! Money Flow Index (MFI)
//!
//! MFI is like RSI but incorporates volume. It measures buying and selling pressure.
//!
//! Formula:
//! Typical Price = (High + Low + Close) / 3
//! Money Flow = Typical Price * Volume
//! Positive Money Flow = sum of money flow when typical price increases
//! Negative Money Flow = sum of money flow when typical price decreases
//! Money Ratio = Positive Money Flow / Negative Money Flow
//! MFI = 100 - (100 / (1 + Money Ratio))
//!
//! Values above 80 indicate overbought, below 20 indicate oversold.

use chrono::{DateTime, Utc};

use crate::{
    traits::Indicator, types::OhlcBar, utils::CircularBuffer, IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone, Copy, Default)]
struct MfiPoint {
    typical_price: f64,
    money_flow: f64,
}

/// Money Flow Index indicator.
///
/// Volume-weighted momentum indicator similar to RSI.
#[derive(Debug, Clone)]
pub struct MFI {
    period: usize,
    buffer: CircularBuffer<MfiPoint>,
    previous_typical: Option<f64>,
    name: String,
}

impl MFI {
    /// Creates a new MFI indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(MFI {
            period,
            buffer: CircularBuffer::new(period),
            previous_typical: None,
            name: format!("MFI({period})"),
        })
    }

    /// Update MFI with OHLC bar and volume.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        let typical_price = bar.typical_price();
        let money_flow = typical_price * volume;

        // Store for calculation
        self.buffer.push(MfiPoint {
            typical_price,
            money_flow,
        });

        let mfi = self.calculate_mfi();
        self.previous_typical = Some(typical_price);

        Ok(mfi)
    }

    /// Calculate MFI from buffer.
    fn calculate_mfi(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let mut positive_flow = 0.0;
        let mut negative_flow = 0.0;
        let mut prev_typical: Option<f64> = None;

        for point in self.buffer.iter() {
            if let Some(prev) = prev_typical {
                if point.typical_price > prev {
                    positive_flow += point.money_flow;
                } else if point.typical_price < prev {
                    negative_flow += point.money_flow;
                }
            }
            prev_typical = Some(point.typical_price);
        }

        if negative_flow == 0.0 {
            return Some(100.0); // All positive = overbought
        }

        let money_ratio = positive_flow / negative_flow;
        let mfi = 100.0 - (100.0 / (1.0 + money_ratio));

        Some(mfi.clamp(0.0, 100.0))
    }

    /// Get current MFI value.
    pub fn current(&self) -> Option<f64> {
        self.calculate_mfi()
    }
}

impl Indicator for MFI {
    fn name(&self) -> &str {
        &self.name
    }

    fn warmup_period(&self) -> usize {
        self.period + 1 // Need period + 1 to calculate ratios
    }

    fn is_ready(&self) -> bool {
        self.buffer.is_full() && self.previous_typical.is_some()
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.previous_typical = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mfi_creation() {
        let mfi = MFI::new(14).unwrap();
        assert_eq!(mfi.name(), "MFI(14)");
        assert!(!mfi.is_ready());
    }

    #[test]
    fn test_mfi_range() {
        let mut mfi = MFI::new(5).unwrap();
        let timestamp = Utc::now();

        // Feed some data
        for i in 0..10 {
            let price = 100.0 + i as f64;
            let bar = OhlcBar::new(price, price + 2.0, price - 2.0, price + 1.0);
            mfi.update_ohlc(&bar, 1000.0, timestamp).unwrap();
        }

        if let Some(value) = mfi.current() {
            // MFI should be between 0 and 100
            assert!((0.0..=100.0).contains(&value));
        }
    }
}
