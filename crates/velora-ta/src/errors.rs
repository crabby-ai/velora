//! Error types for technical indicators.

use std::fmt;

/// Result type for indicator operations.
pub type IndicatorResult<T> = Result<T, IndicatorError>;

/// Errors that can occur during indicator calculations.
#[derive(Debug, Clone, PartialEq)]
pub enum IndicatorError {
    /// Insufficient data to calculate the indicator.
    ///
    /// Most indicators require a minimum number of data points (warmup period)
    /// before they can produce valid output.
    InsufficientData {
        /// Minimum required data points
        required: usize,
        /// Actual number of data points available
        actual: usize,
    },

    /// Invalid parameter provided to indicator constructor.
    ///
    /// For example, a period of 0 or a negative standard deviation.
    InvalidParameter(String),

    /// Error during calculation.
    ///
    /// This can occur due to numerical issues like division by zero,
    /// overflow, or invalid mathematical operations.
    Calculation(String),

    /// Indicator not properly initialized.
    ///
    /// This error occurs when trying to use an indicator before it's ready.
    NotInitialized(String),

    /// Invalid price data.
    ///
    /// Prices cannot be negative or NaN.
    InvalidPrice(String),

    /// Invalid input data.
    ///
    /// General validation error for input data.
    InvalidInput(String),

    /// Division by zero encountered.
    DivisionByZero,
}

impl fmt::Display for IndicatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IndicatorError::InsufficientData { required, actual } => {
                write!(
                    f,
                    "Insufficient data: need at least {required} data points, got {actual}"
                )
            }
            IndicatorError::InvalidParameter(msg) => write!(f, "Invalid parameter: {msg}"),
            IndicatorError::Calculation(msg) => write!(f, "Calculation error: {msg}"),
            IndicatorError::NotInitialized(msg) => write!(f, "Not initialized: {msg}"),
            IndicatorError::InvalidPrice(msg) => write!(f, "Invalid price: {msg}"),
            IndicatorError::InvalidInput(msg) => write!(f, "Invalid input: {msg}"),
            IndicatorError::DivisionByZero => write!(f, "Division by zero"),
        }
    }
}

impl std::error::Error for IndicatorError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IndicatorError::InsufficientData {
            required: 20,
            actual: 10,
        };
        assert_eq!(
            err.to_string(),
            "Insufficient data: need at least 20 data points, got 10"
        );

        let err = IndicatorError::InvalidParameter("period must be > 0".to_string());
        assert_eq!(err.to_string(), "Invalid parameter: period must be > 0");

        let err = IndicatorError::DivisionByZero;
        assert_eq!(err.to_string(), "Division by zero");
    }
}
