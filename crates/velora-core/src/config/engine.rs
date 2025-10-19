//! Trading engine configuration types for the Velora platform.
//!
//! This module contains configuration structures for different trading modes:
//! - Backtesting (historical simulation)
//! - Live trading (real-time execution)

use serde::{Deserialize, Serialize};

/// Trading engine configuration.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EngineConfig {
    /// Backtesting configuration
    pub backtest: BacktestConfig,

    /// Live trading configuration
    pub live: LiveTradingConfig,
}

/// Backtesting engine configuration.
///
/// Controls simulation parameters for historical testing of trading strategies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestConfig {
    /// Initial capital for backtesting
    /// Env: VELORA_ENGINE_BACKTEST_INITIAL_CAPITAL
    pub initial_capital: f64,

    /// Commission rate (e.g., 0.001 for 0.1%)
    /// Env: VELORA_ENGINE_BACKTEST_COMMISSION_RATE
    pub commission_rate: f64,

    /// Slippage rate (e.g., 0.0001 for 0.01%)
    /// Env: VELORA_ENGINE_BACKTEST_SLIPPAGE_RATE
    pub slippage_rate: f64,
}

impl Default for BacktestConfig {
    fn default() -> Self {
        BacktestConfig {
            initial_capital: 10000.0,
            commission_rate: 0.001,
            slippage_rate: 0.0001,
        }
    }
}

/// Live trading engine configuration.
///
/// Controls real-time trading behavior and safety parameters.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveTradingConfig {
    /// Enable dry-run mode (no real orders)
    /// Env: VELORA_ENGINE_LIVE_DRY_RUN
    ///
    /// **CRITICAL**: Set to `false` only for real money trading.
    /// Default is `true` for safety.
    #[serde(default = "default_true")]
    pub dry_run: bool,

    /// Order timeout in seconds
    /// Env: VELORA_ENGINE_LIVE_ORDER_TIMEOUT_SECONDS
    pub order_timeout_seconds: u64,

    /// Position check interval in milliseconds
    /// Env: VELORA_ENGINE_LIVE_POSITION_CHECK_INTERVAL_MS
    pub position_check_interval_ms: u64,
}

fn default_true() -> bool {
    true
}

impl Default for LiveTradingConfig {
    fn default() -> Self {
        LiveTradingConfig {
            dry_run: true,
            order_timeout_seconds: 30,
            position_check_interval_ms: 1000,
        }
    }
}
