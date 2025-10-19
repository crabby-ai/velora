//! Exchange connection configuration for the Velora platform.
//!
//! This module contains configuration for connecting to cryptocurrency
//! exchanges and managing API rate limits.

use serde::{Deserialize, Serialize};

/// Exchange connection configuration.
///
/// Configures connection parameters for a specific cryptocurrency exchange.
///
/// # Security Note
///
/// API secrets should **never** be committed to configuration files.
/// Use environment variables instead:
/// - `VELORA_EXCHANGES_<NAME>_API_KEY`
/// - `VELORA_EXCHANGES_<NAME>_API_SECRET`
///
/// # Example
///
/// ```toml
/// [exchanges.binance]
/// name = "binance"
/// testnet = true
/// rate_limit.requests_per_second = 10
/// rate_limit.orders_per_second = 5
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExchangeConfig {
    /// Exchange name (e.g., "binance", "coinbase")
    pub name: String,

    /// API key (optional for public data)
    /// Env: VELORA_EXCHANGES_<NAME>_API_KEY
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key: Option<String>,

    /// API secret (optional for public data)
    /// Env: VELORA_EXCHANGES_<NAME>_API_SECRET
    /// Note: For security, prefer env vars over config files
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_secret: Option<String>,

    /// Use testnet/sandbox environment
    /// Env: VELORA_EXCHANGES_<NAME>_TESTNET
    #[serde(default)]
    pub testnet: bool,

    /// Custom REST API URL (overrides default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rest_url: Option<String>,

    /// Custom WebSocket URL (overrides default)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ws_url: Option<String>,

    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
}

/// Rate limiting configuration for exchange API calls.
///
/// Prevents exceeding exchange API rate limits and potential bans.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    /// Env: VELORA_RATE_LIMIT_REQUESTS_PER_SECOND
    pub requests_per_second: u32,

    /// Maximum orders per second
    /// Env: VELORA_RATE_LIMIT_ORDERS_PER_SECOND
    pub orders_per_second: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        RateLimitConfig {
            requests_per_second: 10,
            orders_per_second: 5,
        }
    }
}
