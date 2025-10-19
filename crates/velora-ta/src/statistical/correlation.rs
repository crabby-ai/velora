//! Correlation Coefficient

use crate::{utils::CircularBuffer, IndicatorError, IndicatorResult};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Correlation {
    period: usize,
    buffer_x: CircularBuffer<f64>,
    buffer_y: CircularBuffer<f64>,
    name: String,
}

impl Correlation {
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period < 2 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be >= 2".to_string(),
            ));
        }

        Ok(Correlation {
            period,
            buffer_x: CircularBuffer::new(period),
            buffer_y: CircularBuffer::new(period),
            name: format!("Corr({period})"),
        })
    }

    pub fn update(&mut self, x: f64, y: f64) -> IndicatorResult<Option<f64>> {
        self.buffer_x.push(x);
        self.buffer_y.push(y);

        if !self.buffer_x.is_full() {
            return Ok(None);
        }

        let mean_x = self.buffer_x.mean().unwrap_or(0.0);
        let mean_y = self.buffer_y.mean().unwrap_or(0.0);

        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        let mut sum_y2 = 0.0;

        for (&x_val, &y_val) in self.buffer_x.iter().zip(self.buffer_y.iter()) {
            let dx = x_val - mean_x;
            let dy = y_val - mean_y;
            sum_xy += dx * dy;
            sum_x2 += dx * dx;
            sum_y2 += dy * dy;
        }

        let denominator = (sum_x2 * sum_y2).sqrt();
        if denominator == 0.0 {
            return Ok(Some(0.0));
        }

        Ok(Some(sum_xy / denominator))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correlation_creation() {
        let corr = Correlation::new(20).unwrap();
        assert_eq!(corr.name, "Corr(20)");
    }
}
