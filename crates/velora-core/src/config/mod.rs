//! Configuration management for the Velora trading platform.
//!
//! This module provides comprehensive configuration support with multiple sources:
//! - TOML configuration files
//! - Environment variables
//! - Default values
//!
//! Configuration is loaded in the following priority (highest to lowest):
//! 1. Environment variables (VELORA_*)
//! 2. Configuration file
//! 3. Default values
//!
//! # Using gonfig
//!
//! The configuration system uses the `gonfig` crate for unified configuration management.
//! Simply call `VeloraConfig::from_gonfig()` to load from all sources automatically.
//!
//! # Module Structure
//!
//! - [`database`] - Database backend configuration (QuestDB, TimescaleDB, InMemory)
//! - [`engine`] - Trading engine configuration (Backtesting, Live Trading)
//! - [`exchange`] - Exchange connection and rate limiting
//! - [`risk`] - Risk management limits and constraints
//! - [`logging`] - Logging levels and output configuration
//!
//! # Examples
//!
//! ```no_run
//! use velora_core::VeloraConfig;
//!
//! // Method 1: Auto-load from all sources (recommended)
//! let config = VeloraConfig::from_gonfig().unwrap();
//!
//! // Method 2: Load from single file
//! let config = VeloraConfig::from_file("config.toml").unwrap();
//!
//! // Method 3: Layer multiple configs
//! let config = VeloraConfig::from_files(&[
//!     "config/base.toml",
//!     "config/testing.toml",
//! ]).unwrap();
//!
//! // Always validate after loading
//! config.validate().unwrap();
//! ```

mod database;
mod engine;
mod exchange;
mod logging;
mod risk;

// Re-export all config types
pub use database::{DatabaseBackend, DatabaseConfig, QuestDbConfig, TimescaleDbConfig};
pub use engine::{BacktestConfig, EngineConfig, LiveTradingConfig};
pub use exchange::{ExchangeConfig, RateLimitConfig};
pub use logging::LoggingConfig;
pub use risk::RiskConfig;

use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::errors::{Result, VeloraError};

/// Main configuration for the Velora platform.
///
/// Environment variables can override any setting using the pattern:
/// `VELORA_<SECTION>_<KEY>` (e.g., `VELORA_ENGINE_LIVE_DRY_RUN=false`)
///
/// # Examples
///
/// ```no_run
/// use velora_core::VeloraConfig;
///
/// // Load from all sources (files + env vars)
/// let config = VeloraConfig::from_gonfig().unwrap();
/// ```
#[allow(missing_docs)]
#[derive(Debug, Clone, Serialize, Deserialize, Default, Gonfig)]
#[Gonfig(env_prefix = "VELORA")]
pub struct VeloraConfig {
    /// Exchange configurations
    /// Env: VELORA_EXCHANGES_* (complex, prefer config file)
    #[serde(default)]
    pub exchanges: HashMap<String, ExchangeConfig>,

    /// Engine configuration
    /// Env: VELORA_ENGINE_*
    #[serde(default)]
    pub engine: EngineConfig,

    /// Risk management configuration
    /// Env: VELORA_RISK_*
    #[serde(default)]
    pub risk: RiskConfig,

    /// Logging configuration
    /// Env: VELORA_LOGGING_*
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Database configuration
    /// Env: VELORA_DATABASE_*
    #[serde(default)]
    pub database: DatabaseConfig,
}

impl VeloraConfig {
    /// Load configuration from a single TOML file with environment variable overrides.
    ///
    /// This is a convenience wrapper around gonfig's ConfigBuilder.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use velora_core::VeloraConfig;
    ///
    /// let config = VeloraConfig::from_file("config.toml").unwrap();
    /// ```
    pub fn from_file<P: AsRef<std::path::Path>>(path: P) -> Result<Self> {
        use gonfig::ConfigBuilder;

        let config = ConfigBuilder::new()
            .with_file(
                path.as_ref()
                    .to_str()
                    .ok_or_else(|| VeloraError::ConfigError("Invalid file path".to_string()))?,
            )?
            .with_env("VELORA")
            .build::<Self>()?;

        Ok(config)
    }

    /// Load configuration from multiple layered TOML files.
    ///
    /// Files are loaded in order, with later files overriding earlier ones.
    /// Environment variables override all file-based configuration.
    ///
    /// # Priority (highest to lowest)
    /// 1. Environment variables (VELORA_*)
    /// 2. Last config file in the list
    /// 3. ...
    /// 4. First config file in the list
    /// 5. Default values
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use velora_core::VeloraConfig;
    ///
    /// // Layer environment-specific config over base config
    /// let config = VeloraConfig::from_files(&[
    ///     "config/base.toml",
    ///     "config/paper-trading.toml",
    /// ]).unwrap();
    /// ```
    pub fn from_files<P: AsRef<std::path::Path>>(paths: &[P]) -> Result<Self> {
        use gonfig::{ConfigBuilder, MergeStrategy};

        let mut builder = ConfigBuilder::new().with_merge_strategy(MergeStrategy::Deep);

        // Add each file in order
        for path in paths {
            builder = builder.with_file(
                path.as_ref()
                    .to_str()
                    .ok_or_else(|| VeloraError::ConfigError("Invalid file path".to_string()))?,
            )?;
        }

        // Environment variables have highest priority
        let config = builder.with_env("VELORA").build::<Self>()?;

        Ok(config)
    }

    /// Load configuration with optional files.
    ///
    /// Similar to `from_files`, but allows some files to be optional.
    /// Useful for optional local overrides.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use velora_core::VeloraConfig;
    ///
    /// // base.toml is required, local.toml is optional
    /// let config = VeloraConfig::from_files_optional(&[
    ///     ("config/base.toml", true),           // required
    ///     ("config/paper-trading.toml", true),  // required
    ///     ("config/local.toml", false),         // optional override
    /// ]).unwrap();
    /// ```
    pub fn from_files_optional<P: AsRef<std::path::Path>>(paths: &[(P, bool)]) -> Result<Self> {
        use gonfig::{ConfigBuilder, MergeStrategy};

        let mut builder = ConfigBuilder::new().with_merge_strategy(MergeStrategy::Deep);

        for (path, required) in paths {
            let path_str = path
                .as_ref()
                .to_str()
                .ok_or_else(|| VeloraError::ConfigError("Invalid file path".to_string()))?;

            if *required {
                builder = builder.with_file(path_str)?;
            } else {
                builder = builder.with_file_optional(path_str)?;
            }
        }

        let config = builder.with_env("VELORA").build::<Self>()?;

        Ok(config)
    }

    /// Validate configuration values.
    ///
    /// Checks that all configuration values are within acceptable ranges
    /// and meet business logic requirements.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Initial capital is not positive
    /// - Commission rate is not between 0 and 1
    /// - Max drawdown is not between 0 and 100
    pub fn validate(&self) -> Result<()> {
        // Validate engine config
        if self.engine.backtest.initial_capital <= 0.0 {
            return Err(VeloraError::ConfigError(
                "Initial capital must be positive".to_string(),
            ));
        }

        if self.engine.backtest.commission_rate < 0.0 || self.engine.backtest.commission_rate > 1.0
        {
            return Err(VeloraError::ConfigError(
                "Commission rate must be between 0 and 1".to_string(),
            ));
        }

        // Validate risk config
        if self.risk.max_drawdown_percent <= 0.0 || self.risk.max_drawdown_percent > 100.0 {
            return Err(VeloraError::ConfigError(
                "Max drawdown must be between 0 and 100".to_string(),
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_default_config() {
        let config = VeloraConfig::default();
        assert_eq!(config.engine.backtest.initial_capital, 10000.0);
        assert_eq!(config.risk.max_drawdown_percent, 20.0);
        assert!(config.engine.live.dry_run);
        assert_eq!(config.database.backend, DatabaseBackend::InMemory);
    }

    #[test]
    fn test_config_validation() {
        let mut config = VeloraConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid capital
        config.engine.backtest.initial_capital = -100.0;
        assert!(config.validate().is_err());

        // Reset and test invalid commission
        config.engine.backtest.initial_capital = 10000.0;
        config.engine.backtest.commission_rate = 1.5;
        assert!(config.validate().is_err());

        // Reset and test invalid drawdown
        config.engine.backtest.commission_rate = 0.001;
        config.risk.max_drawdown_percent = 150.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_env_override() {
        // Set environment variables
        env::set_var("VELORA_ENGINE_LIVE_DRY_RUN", "false");
        env::set_var("VELORA_ENGINE_BACKTEST_INITIAL_CAPITAL", "50000");
        env::set_var("VELORA_RISK_MAX_POSITION_SIZE", "5000");

        // Note: gonfig loads from env automatically with from_gonfig()
        // For now, just test that defaults work
        let config = VeloraConfig::default();
        assert!(config.validate().is_ok());

        // Clean up
        env::remove_var("VELORA_ENGINE_LIVE_DRY_RUN");
        env::remove_var("VELORA_ENGINE_BACKTEST_INITIAL_CAPITAL");
        env::remove_var("VELORA_RISK_MAX_POSITION_SIZE");
    }

    #[test]
    fn test_database_backends() {
        let config = VeloraConfig::default();
        assert_eq!(config.database.backend, DatabaseBackend::InMemory);

        let questdb_config = QuestDbConfig::default();
        assert_eq!(questdb_config.pg_port, 8812);
        assert_eq!(questdb_config.database, "qdb");

        let timescale_config = TimescaleDbConfig::default();
        assert_eq!(timescale_config.port, 5432);
        assert_eq!(timescale_config.database, "velora");
    }

    #[test]
    fn test_rate_limit_default() {
        let rate_limit = RateLimitConfig::default();
        assert_eq!(rate_limit.requests_per_second, 10);
        assert_eq!(rate_limit.orders_per_second, 5);
    }

    #[test]
    fn test_config_serialization() {
        let config = VeloraConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("exchanges"));
        assert!(json.contains("engine"));
        assert!(json.contains("database"));
    }
}
