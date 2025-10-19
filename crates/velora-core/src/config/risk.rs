//! Risk management configuration for the Velora platform.
//!
//! This module defines risk limits and position constraints to protect
//! trading capital and prevent catastrophic losses.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Risk management configuration.
///
/// Defines limits and constraints to protect trading capital.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskConfig {
    /// Maximum position size per trade
    /// Env: VELORA_RISK_MAX_POSITION_SIZE
    pub max_position_size: f64,

    /// Maximum total exposure across all positions
    /// Env: VELORA_RISK_MAX_TOTAL_EXPOSURE
    pub max_total_exposure: f64,

    /// Maximum drawdown percentage before stopping
    /// Env: VELORA_RISK_MAX_DRAWDOWN_PERCENT
    pub max_drawdown_percent: f64,

    /// Maximum daily loss before stopping
    /// Env: VELORA_RISK_MAX_DAILY_LOSS
    pub max_daily_loss: f64,

    /// Per-symbol position limits (prefer config file)
    ///
    /// Example:
    /// ```toml
    /// [risk.position_limits]
    /// "BTC/USDT" = 5000.0
    /// "ETH/USDT" = 2000.0
    /// ```
    #[serde(default)]
    pub position_limits: HashMap<String, f64>,
}

impl Default for RiskConfig {
    fn default() -> Self {
        RiskConfig {
            max_position_size: 1000.0,
            max_total_exposure: 10000.0,
            max_drawdown_percent: 20.0,
            max_daily_loss: 500.0,
            position_limits: HashMap::new(),
        }
    }
}
