//! Mathematical utilities for indicator calculations.

/// Calculate the true range for a candle.
///
/// True Range is the greatest of:
/// - Current High - Current Low
/// - abs(Current High - Previous Close)
/// - abs(Current Low - Previous Close)
///
/// # Arguments
///
/// * `high` - Current period's high price
/// * `low` - Current period's low price
/// * `prev_close` - Previous period's close price
pub fn true_range(high: f64, low: f64, prev_close: f64) -> f64 {
    let hl = high - low;
    let hc = (high - prev_close).abs();
    let lc = (low - prev_close).abs();

    hl.max(hc).max(lc)
}

/// Calculate the typical price for a candle.
///
/// Typical Price = (High + Low + Close) / 3
pub fn typical_price(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close) / 3.0
}

/// Calculate the weighted close price.
///
/// Weighted Close = (High + Low + Close + Close) / 4
///
/// This gives more weight to the close price.
pub fn weighted_close(high: f64, low: f64, close: f64) -> f64 {
    (high + low + close + close) / 4.0
}

/// Calculate the median price.
///
/// Median Price = (High + Low) / 2
pub fn median_price(high: f64, low: f64) -> f64 {
    (high + low) / 2.0
}

/// Calculate the money flow multiplier for Money Flow Index.
///
/// Money Flow Multiplier = [(Close - Low) - (High - Close)] / (High - Low)
pub fn money_flow_multiplier(high: f64, low: f64, close: f64) -> f64 {
    if high == low {
        return 0.0; // Avoid division by zero
    }
    ((close - low) - (high - close)) / (high - low)
}

/// Validate that a value is a valid price (positive and finite).
pub fn is_valid_price(price: f64) -> bool {
    price.is_finite() && price > 0.0
}

/// Validate that a period is valid (greater than 0).
pub fn is_valid_period(period: usize) -> bool {
    period > 0
}

/// Calculate EMA multiplier for a given period.
///
/// Multiplier = 2 / (period + 1)
pub fn ema_multiplier(period: usize) -> f64 {
    2.0 / (period as f64 + 1.0)
}

/// Safe division that returns 0.0 instead of infinity or NaN.
pub fn safe_div(numerator: f64, denominator: f64) -> f64 {
    if denominator == 0.0 || !denominator.is_finite() {
        0.0
    } else {
        numerator / denominator
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_true_range() {
        // Case 1: High - Low is the greatest
        assert_eq!(true_range(105.0, 95.0, 100.0), 10.0);

        // Case 2: |High - Prev Close| is the greatest
        assert_eq!(true_range(120.0, 110.0, 100.0), 20.0);

        // Case 3: |Low - Prev Close| is the greatest
        assert_eq!(true_range(105.0, 85.0, 100.0), 20.0);
    }

    #[test]
    fn test_typical_price() {
        let result = typical_price(105.0, 95.0, 102.0);
        assert!((result - 100.666666).abs() < 0.001);
    }

    #[test]
    fn test_weighted_close() {
        let result = weighted_close(105.0, 95.0, 102.0);
        assert_eq!(result, 101.0);
    }

    #[test]
    fn test_median_price() {
        assert_eq!(median_price(105.0, 95.0), 100.0);
    }

    #[test]
    fn test_money_flow_multiplier() {
        // Close at high: positive flow
        let result = money_flow_multiplier(110.0, 100.0, 110.0);
        assert_eq!(result, 1.0);

        // Close at low: negative flow
        let result = money_flow_multiplier(110.0, 100.0, 100.0);
        assert_eq!(result, -1.0);

        // Close at middle: neutral flow
        let result = money_flow_multiplier(110.0, 100.0, 105.0);
        assert_eq!(result, 0.0);

        // High == Low: should return 0.0
        let result = money_flow_multiplier(100.0, 100.0, 100.0);
        assert_eq!(result, 0.0);
    }

    #[test]
    fn test_is_valid_price() {
        assert!(is_valid_price(100.0));
        assert!(is_valid_price(0.001));
        assert!(!is_valid_price(0.0));
        assert!(!is_valid_price(-10.0));
        assert!(!is_valid_price(f64::NAN));
        assert!(!is_valid_price(f64::INFINITY));
    }

    #[test]
    fn test_is_valid_period() {
        assert!(is_valid_period(1));
        assert!(is_valid_period(20));
        assert!(!is_valid_period(0));
    }

    #[test]
    fn test_ema_multiplier() {
        // For period 9: 2/(9+1) = 0.2
        assert_eq!(ema_multiplier(9), 0.2);

        // For period 14: 2/(14+1) = 0.133...
        assert!((ema_multiplier(14) - 0.133333).abs() < 0.001);
    }

    #[test]
    fn test_safe_div() {
        assert_eq!(safe_div(10.0, 2.0), 5.0);
        assert_eq!(safe_div(10.0, 0.0), 0.0);
        assert_eq!(safe_div(10.0, f64::INFINITY), 0.0);
        assert_eq!(safe_div(10.0, f64::NAN), 0.0);
    }
}
