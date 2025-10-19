//! Logging configuration for the Velora platform.
//!
//! This module defines logging levels and output destinations.

use serde::{Deserialize, Serialize};

/// Logging configuration.
///
/// Controls log verbosity and output destinations.
///
/// # Log Levels
///
/// From most to least verbose:
/// - `trace` - Very detailed debug information
/// - `debug` - Debug information useful for development
/// - `info` - General informational messages (default)
/// - `warn` - Warning messages for potentially problematic situations
/// - `error` - Error messages for failures
///
/// # Example
///
/// ```toml
/// [logging]
/// level = "info"
/// file = "logs/velora.log"
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    /// Env: VELORA_LOGGING_LEVEL
    pub level: String,

    /// Optional log file path
    /// Env: VELORA_LOGGING_FILE
    ///
    /// If not specified, logs are written to stdout only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        LoggingConfig {
            level: "info".to_string(),
            file: None,
        }
    }
}
