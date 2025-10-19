//! Configuration types for the trading engine

use serde::{Deserialize, Serialize};

/// Execution mode for the trading engine
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionMode {
    /// Live trading with real money
    Live,
    /// Paper trading (simulated orders without real money)
    DryRun,
}

/// Configuration for the trading engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineConfig {
    /// Execution mode (Live or DryRun)
    pub mode: ExecutionMode,

    /// Symbols to trade
    pub symbols: Vec<String>,

    /// Initial capital for trading
    pub initial_capital: f64,

    /// Maximum orders allowed per second (rate limiting)
    pub max_orders_per_second: u32,

    /// Heartbeat interval in milliseconds
    pub heartbeat_interval_ms: u64,

    /// Delay before attempting reconnection (ms)
    pub reconnect_delay_ms: u64,

    /// Maximum number of reconnection attempts
    pub max_reconnect_attempts: u32,

    /// Enable risk checks before order submission
    pub enable_risk_checks: bool,

    /// Metrics configuration
    pub metrics: MetricsConfig,
}

/// Metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable performance tracking (latency, throughput)
    pub enable_performance_tracking: bool,

    /// Enable order metrics tracking
    pub enable_order_metrics: bool,

    /// Interval for taking equity snapshots (seconds)
    pub snapshot_interval_secs: u64,
}

impl Default for EngineConfig {
    fn default() -> Self {
        Self {
            mode: ExecutionMode::DryRun,
            symbols: Vec::new(),
            initial_capital: 10_000.0,
            max_orders_per_second: 5,
            heartbeat_interval_ms: 1000,
            reconnect_delay_ms: 5000,
            max_reconnect_attempts: 10,
            enable_risk_checks: true,
            metrics: MetricsConfig::default(),
        }
    }
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            enable_performance_tracking: true,
            enable_order_metrics: true,
            snapshot_interval_secs: 60,
        }
    }
}

impl EngineConfig {
    /// Create a new engine configuration with builder pattern
    pub fn builder() -> EngineConfigBuilder {
        EngineConfigBuilder::default()
    }
}

/// Builder for EngineConfig
#[derive(Default)]
pub struct EngineConfigBuilder {
    config: EngineConfig,
}

impl EngineConfigBuilder {
    /// Set execution mode
    pub fn mode(mut self, mode: ExecutionMode) -> Self {
        self.config.mode = mode;
        self
    }

    /// Set symbols to trade
    pub fn symbols(mut self, symbols: Vec<String>) -> Self {
        self.config.symbols = symbols;
        self
    }

    /// Add a single symbol
    pub fn add_symbol(mut self, symbol: String) -> Self {
        self.config.symbols.push(symbol);
        self
    }

    /// Set initial capital
    pub fn initial_capital(mut self, capital: f64) -> Self {
        self.config.initial_capital = capital;
        self
    }

    /// Set max orders per second
    pub fn max_orders_per_second(mut self, max: u32) -> Self {
        self.config.max_orders_per_second = max;
        self
    }

    /// Set heartbeat interval
    pub fn heartbeat_interval_ms(mut self, ms: u64) -> Self {
        self.config.heartbeat_interval_ms = ms;
        self
    }

    /// Set reconnect delay
    pub fn reconnect_delay_ms(mut self, ms: u64) -> Self {
        self.config.reconnect_delay_ms = ms;
        self
    }

    /// Set max reconnect attempts
    pub fn max_reconnect_attempts(mut self, attempts: u32) -> Self {
        self.config.max_reconnect_attempts = attempts;
        self
    }

    /// Enable or disable risk checks
    pub fn enable_risk_checks(mut self, enable: bool) -> Self {
        self.config.enable_risk_checks = enable;
        self
    }

    /// Set metrics configuration
    pub fn metrics(mut self, metrics: MetricsConfig) -> Self {
        self.config.metrics = metrics;
        self
    }

    /// Build the configuration
    pub fn build(self) -> EngineConfig {
        self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = EngineConfig::default();

        assert_eq!(config.mode, ExecutionMode::DryRun);
        assert_eq!(config.initial_capital, 10_000.0);
        assert_eq!(config.max_orders_per_second, 5);
        assert!(config.enable_risk_checks);
    }

    #[test]
    fn test_builder() {
        let config = EngineConfig::builder()
            .mode(ExecutionMode::Live)
            .add_symbol("BTC-USD-PERP".to_string())
            .add_symbol("ETH-USD-PERP".to_string())
            .initial_capital(50_000.0)
            .max_orders_per_second(10)
            .enable_risk_checks(false)
            .build();

        assert_eq!(config.mode, ExecutionMode::Live);
        assert_eq!(config.symbols.len(), 2);
        assert_eq!(config.initial_capital, 50_000.0);
        assert_eq!(config.max_orders_per_second, 10);
        assert!(!config.enable_risk_checks);
    }

    #[test]
    fn test_execution_mode_serialization() {
        let mode = ExecutionMode::Live;
        let json = serde_json::to_string(&mode).unwrap();
        let deserialized: ExecutionMode = serde_json::from_str(&json).unwrap();

        assert_eq!(mode, deserialized);
    }
}
