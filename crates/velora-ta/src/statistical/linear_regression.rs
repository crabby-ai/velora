//! Linear Regression

use chrono::{DateTime, Utc};

use crate::{
    traits::{Indicator, SingleIndicator},
    utils::CircularBuffer,
    IndicatorError, IndicatorResult,
};

#[derive(Debug, Clone)]
pub struct LinearRegression {
    period: usize,
    buffer: CircularBuffer<f64>,
    name: String,
}

impl LinearRegression {
    pub fn new(period: usize) -> IndicatorResult<Self> {
        if period < 2 {
            return Err(IndicatorError::InvalidParameter(
                "Period must be >= 2".to_string(),
            ));
        }

        Ok(LinearRegression {
            period,
            buffer: CircularBuffer::new(period),
            name: format!("LinReg({period})"),
        })
    }

    fn calculate_regression(&self) -> Option<f64> {
        if !self.is_ready() {
            return None;
        }

        let n = self.period as f64;
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;

        for (i, &price) in self.buffer.iter().enumerate() {
            let x = i as f64;
            sum_x += x;
            sum_y += price;
            sum_xy += x * price;
            sum_x2 += x * x;
        }

        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator == 0.0 {
            return self.buffer.last();
        }

        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;

        // Return forecast for next period
        Some(slope * (n - 1.0) + intercept)
    }
}

impl Indicator for LinearRegression {
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

impl SingleIndicator for LinearRegression {
    fn update(&mut self, price: f64, _timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>> {
        if !price.is_finite() {
            return Err(IndicatorError::InvalidPrice(
                "Price must be finite".to_string(),
            ));
        }

        self.buffer.push(price);
        Ok(self.calculate_regression())
    }

    fn current(&self) -> Option<f64> {
        self.calculate_regression()
    }

    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>> {
        if prices.is_empty() {
            return Ok(Vec::new());
        }

        let mut lr = Self::new(self.period)?;
        let mut result = Vec::with_capacity(prices.len());
        let timestamp = Utc::now();

        for &price in prices {
            result.push(lr.update(price, timestamp)?);
        }

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_linear_regression_creation() {
        let lr = LinearRegression::new(10).unwrap();
        assert_eq!(lr.name(), "LinReg(10)");
    }
}
