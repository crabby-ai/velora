//! Configuration types for backtesting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Main configuration for a backtest run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial capital for the backtest
    pub initial_capital: f64,

    /// Start date of the backtest
    pub start_date: DateTime<Utc>,

    /// End date of the backtest
    pub end_date: DateTime<Utc>,

    /// Symbols to include in the backtest
    pub symbols: Vec<String>,

    /// Execution configuration
    pub execution: ExecutionConfig,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        Self {
            initial_capital: 10_000.0,
            start_date: Utc::now() - chrono::Duration::days(365),
            end_date: Utc::now(),
            symbols: vec![],
            execution: ExecutionConfig::default(),
        }
    }
}

impl BacktestConfig {
    /// Create a new backtest configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set initial capital
    pub fn with_capital(mut self, capital: f64) -> Self {
        self.initial_capital = capital;
        self
    }

    /// Set date range
    pub fn with_date_range(mut self, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        self.start_date = start;
        self.end_date = end;
        self
    }

    /// Set symbols
    pub fn with_symbols(mut self, symbols: Vec<String>) -> Self {
        self.symbols = symbols;
        self
    }

    /// Set execution config
    pub fn with_execution(mut self, execution: ExecutionConfig) -> Self {
        self.execution = execution;
        self
    }
}

/// Configuration for order execution simulation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Commission rate (e.g., 0.001 = 0.1%)
    pub commission_rate: f64,

    /// Slippage in basis points
    pub slippage_bps: f64,

    /// Simulated fill delay in milliseconds
    pub fill_delay_ms: u64,

    /// Fill model to use
    pub fill_model: FillModel,
}

impl Default for ExecutionConfig {
    fn default() -> Self {
        Self {
            commission_rate: 0.001, // 0.1%
            slippage_bps: 5.0,      // 5 basis points
            fill_delay_ms: 0,
            fill_model: FillModel::Market,
        }
    }
}

impl ExecutionConfig {
    /// Create realistic execution config (moderate fees and slippage)
    pub fn realistic() -> Self {
        Self {
            commission_rate: 0.001,
            slippage_bps: 5.0,
            fill_delay_ms: 100,
            fill_model: FillModel::Realistic,
        }
    }

    /// Create pessimistic execution config (high fees and slippage)
    pub fn pessimistic() -> Self {
        Self {
            commission_rate: 0.002,
            slippage_bps: 10.0,
            fill_delay_ms: 500,
            fill_model: FillModel::Pessimistic,
        }
    }

    /// Create optimistic execution config (no fees or slippage)
    pub fn optimistic() -> Self {
        Self {
            commission_rate: 0.0,
            slippage_bps: 0.0,
            fill_delay_ms: 0,
            fill_model: FillModel::Market,
        }
    }
}

/// Model for simulating order fills
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FillModel {
    /// Always fill at current market price (close)
    Market,

    /// Realistic fills with slippage based on order size
    Realistic,

    /// Pessimistic fills (worst case pricing)
    Pessimistic,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = BacktestConfig::new()
            .with_capital(50_000.0)
            .with_symbols(vec!["BTC-USD-PERP".to_string()]);

        assert_eq!(config.initial_capital, 50_000.0);
        assert_eq!(config.symbols.len(), 1);
    }

    #[test]
    fn test_execution_presets() {
        let realistic = ExecutionConfig::realistic();
        assert_eq!(realistic.commission_rate, 0.001);
        assert_eq!(realistic.fill_model, FillModel::Realistic);

        let pessimistic = ExecutionConfig::pessimistic();
        assert_eq!(pessimistic.commission_rate, 0.002);
        assert_eq!(pessimistic.fill_model, FillModel::Pessimistic);

        let optimistic = ExecutionConfig::optimistic();
        assert_eq!(optimistic.commission_rate, 0.0);
        assert_eq!(optimistic.slippage_bps, 0.0);
    }
}
