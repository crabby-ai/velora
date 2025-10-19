//! Chaikin Money Flow (CMF)
//!
//! CMF measures the amount of Money Flow Volume over a period.
//!
//! Formula:
//! Money Flow Multiplier = ((Close - Low) - (High - Close)) / (High - Low)
//! Money Flow Volume = Money Flow Multiplier * Volume
//! CMF = Sum(Money Flow Volume, period) / Sum(Volume, period)
//!
//! Positive values indicate buying pressure, negative indicate selling pressure.

use chrono::{DateTime, Utc};

use crate::{
    traits::Indicator, types::OhlcBar, utils::CircularBuffer, IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone, Copy, Default)]
struct CmfPoint {
    money_flow_volume: f64,
    volume: f64,
}

/// Chaikin Money Flow indicator.
#[derive(Debug, Clone)]
pub struct CMF {
    period: usize,
    buffer: CircularBuffer<CmfPoint>,
    name: String,
}

impl CMF {
    /// Creates a new CMF indicator.
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period == 0 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be greater than 0".to_string(),
            ));
        }

        Ok(CMF {
            period,
            buffer: CircularBuffer::new(period),
            name: format!("CMF({period})"),
        })
    }

    /// Update CMF with OHLC bar and volume.
    pub fn update_ohlc(
        &mut self,
        bar: &OhlcBar,
        volume: f64,
        _timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>> {
        let range = bar.high - bar.low;

        let mf_multiplier = if range == 0.0 {
            0.0
        } else {
            ((bar.close - bar.low) - (bar.high - bar.close)) / range
        };

        let mf_volume = mf_multiplier * volume;

        self.buffer.push(CmfPoint {
            money_flow_volume: mf_volume,
            volume,
        });

        Ok(self.calculate_cmf())
    }

    fn calculate_cmf(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let mut mf_sum = 0.0;
        let mut vol_sum = 0.0;

        for point in self.buffer.iter() {
            mf_sum += point.money_flow_volume;
            vol_sum += point.volume;
        }

        if vol_sum == 0.0 {
            return Some(0.0);
        }

        Some(mf_sum / vol_sum)
    }

    /// Get current CMF value.
    pub fn current(&self) -> Option<f64> {
        self.calculate_cmf()
    }
}

impl Indicator for CMF {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cmf_creation() {
        let cmf = CMF::new(20).unwrap();
        assert_eq!(cmf.name(), "CMF(20)");
        assert!(!cmf.is_ready());
    }

    #[test]
    fn test_cmf_positive() {
        let mut cmf = CMF::new(3).unwrap();
        let timestamp = Utc::now();

        // Closes near high = buying pressure
        for i in 0..5 {
            let price = 100.0 + i as f64;
            let bar = OhlcBar::new(price, price + 5.0, price - 2.0, price + 4.0);
            cmf.update_ohlc(&bar, 1000.0, timestamp).unwrap();
        }

        if let Some(value) = cmf.current() {
            assert!(value > 0.0); // Should show buying pressure
        }
    }
}
