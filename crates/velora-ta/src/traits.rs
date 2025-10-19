//! Traits for technical indicators.

use crate::{IndicatorResult, MultiIndicatorValue};
use chrono::{DateTime, Utc};

/// Base trait for all technical indicators.
///
/// This trait defines common functionality that all indicators must provide,
/// regardless of whether they output single or multiple values.
pub trait Indicator: Send + Sync {
    /// Human-readable name of the indicator.
    ///
    /// # Example
    ///
    /// ```ignore
    /// assert_eq!(sma.name(), "SMA(20)");
    /// assert_eq!(rsi.name(), "RSI(14)");
    /// ```
    fn name(&self) -> &str;

    /// Minimum number of data points required before the indicator produces valid output.
    ///
    /// This is also known as the "warmup period" or "lookback period".
    /// For example, a 20-period SMA needs 20 data points before it can output a value.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sma = SMA::new(20)?;
    /// assert_eq!(sma.warmup_period(), 20);
    /// ```
    fn warmup_period(&self) -> usize;

    /// Check if the indicator has received enough data to produce valid output.
    ///
    /// Returns `true` if the indicator has processed at least `warmup_period()` data points.
    fn is_ready(&self) -> bool;

    /// Reset the indicator to its initial state.
    ///
    /// This clears all internal state and makes the indicator behave as if it was
    /// just constructed. Useful for reusing an indicator with new data.
    fn reset(&mut self);
}

/// Trait for indicators that output a single value.
///
/// Examples: SMA, EMA, RSI, ATR
pub trait SingleIndicator: Indicator {
    /// Process a new data point and return the updated indicator value.
    ///
    /// This is the streaming mode - process one data point at a time and get
    /// immediate results. Returns `None` if the indicator is not ready yet
    /// (hasn't received enough data points).
    ///
    /// # Arguments
    ///
    /// * `price` - The price value to process
    /// * `timestamp` - Timestamp of this data point
    ///
    /// # Example
    ///
    /// ```ignore
    /// let mut sma = SMA::new(3)?;
    ///
    /// assert_eq!(sma.update(10.0, timestamp)?, None);  // Not ready yet
    /// assert_eq!(sma.update(20.0, timestamp)?, None);  // Not ready yet
    /// assert_eq!(sma.update(30.0, timestamp)?, Some(20.0));  // Ready! (10+20+30)/3 = 20
    /// ```
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>) -> IndicatorResult<Option<f64>>;

    /// Get the current indicator value without updating.
    ///
    /// Returns `None` if the indicator hasn't received enough data yet.
    fn current(&self) -> Option<f64>;

    /// Calculate indicator values for historical data (batch mode).
    ///
    /// This processes all prices at once and returns a vector of indicator values.
    /// The output vector will have `prices.len() - warmup_period() + 1` elements,
    /// with `None` values for the warmup period.
    ///
    /// # Arguments
    ///
    /// * `prices` - Historical price data
    ///
    /// # Returns
    ///
    /// Vector of indicator values. Early values (during warmup) will be `None`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let sma = SMA::new(3)?;
    /// let prices = vec![10.0, 20.0, 30.0, 40.0, 50.0];
    /// let values = sma.calculate(&prices)?;
    /// // values = [None, None, Some(20.0), Some(30.0), Some(40.0)]
    /// ```
    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<f64>>>;
}

/// Trait for indicators that output multiple values.
///
/// Examples: MACD (outputs macd_line, signal_line, histogram),
///           Bollinger Bands (outputs upper, middle, lower)
pub trait MultiIndicator: Indicator {
    /// Number of output values this indicator produces.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let macd = MACD::new(12, 26, 9)?;
    /// assert_eq!(macd.output_count(), 3);  // macd_line, signal_line, histogram
    ///
    /// let bb = BollingerBands::new(20, 2.0)?;
    /// assert_eq!(bb.output_count(), 3);  // upper, middle, lower
    /// ```
    fn output_count(&self) -> usize;

    /// Names of each output value.
    ///
    /// The order matches the order of values returned by `update()` and `current()`.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let macd = MACD::new(12, 26, 9)?;
    /// assert_eq!(macd.output_names(), vec!["MACD", "Signal", "Histogram"]);
    /// ```
    fn output_names(&self) -> Vec<&str>;

    /// Process a new data point and return updated indicator values.
    ///
    /// Returns `None` if the indicator is not ready yet.
    ///
    /// # Arguments
    ///
    /// * `price` - The price value to process
    /// * `timestamp` - Timestamp of this data point
    ///
    /// # Returns
    ///
    /// Vector of indicator values in the order specified by `output_names()`.
    fn update(&mut self, price: f64, timestamp: DateTime<Utc>)
        -> IndicatorResult<Option<Vec<f64>>>;

    /// Get current values without updating.
    ///
    /// Returns `None` if the indicator hasn't received enough data yet.
    fn current(&self) -> Option<Vec<f64>>;

    /// Calculate indicator values for historical data (batch mode).
    ///
    /// # Arguments
    ///
    /// * `prices` - Historical price data
    ///
    /// # Returns
    ///
    /// Vector of multi-indicator values. Each element contains all output values
    /// for that time point. Early values (during warmup) will be `None`.
    fn calculate(&self, prices: &[f64]) -> IndicatorResult<Vec<Option<MultiIndicatorValue>>>;
}

// NOTE: OhlcvIndicator and VolumeIndicator traits are commented out for now
// to keep velora-ta standalone (they would depend on external Candle types).
// These will be added back when we implement OHLC-based indicators like ATR.

/*
/// Trait for indicators that require full OHLCV candle data.
///
/// Some indicators need access to high, low, close values, not just a single price.
/// Examples: ATR (needs high, low, close), Stochastic (needs high, low, close)
pub trait OhlcvIndicator: Indicator {
    /// Update the indicator with OHLC data.
    fn update_ohlc(
        &mut self,
        open: f64,
        high: f64,
        low: f64,
        close: f64,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>>;
}
*/

/// Trait for indicators that require volume data in addition to price.
///
/// Examples: VWAP (Volume-Weighted Average Price), MFI (Money Flow Index)
pub trait VolumeIndicator: Indicator {
    /// Update the indicator with price and volume.
    ///
    /// # Arguments
    ///
    /// * `price` - The price value
    /// * `volume` - The volume at this price
    /// * `timestamp` - Timestamp of this data point
    fn update_with_volume(
        &mut self,
        price: f64,
        volume: f64,
        timestamp: DateTime<Utc>,
    ) -> IndicatorResult<Option<f64>>;

    /// Calculate indicator values from price and volume arrays.
    ///
    /// # Arguments
    ///
    /// * `prices` - Historical price data
    /// * `volumes` - Historical volume data (must be same length as prices)
    fn calculate_with_volume(
        &self,
        prices: &[f64],
        volumes: &[f64],
    ) -> IndicatorResult<Vec<Option<f64>>>;
}
