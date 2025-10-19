//! Database configuration types for the Velora platform.
//!
//! This module contains configuration structures for different database backends:
//! - QuestDB (optimized for high-frequency tick data)
//! - TimescaleDB (PostgreSQL-based, ACID compliant)
//! - InMemory (for testing and backtesting)

use serde::{Deserialize, Serialize};

/// Database configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database backend type
    /// Env: VELORA_DATABASE_BACKEND (values: inmemory, questdb, timescaledb)
    pub backend: DatabaseBackend,

    /// QuestDB configuration (used when backend is QuestDB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub questdb: Option<QuestDbConfig>,

    /// TimescaleDB configuration (used when backend is TimescaleDB)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timescaledb: Option<TimescaleDbConfig>,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig {
            backend: DatabaseBackend::InMemory,
            questdb: None,
            timescaledb: None,
        }
    }
}

/// Database backend type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum DatabaseBackend {
    /// In-memory storage (for testing and backtesting)
    InMemory,
    /// QuestDB (optimized for high-frequency tick data)
    QuestDB,
    /// TimescaleDB (PostgreSQL-based, ACID compliant)
    TimescaleDB,
}

/// QuestDB connection configuration.
///
/// QuestDB is a high-performance time-series database optimized for
/// high-frequency trading tick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuestDbConfig {
    /// QuestDB host
    /// Env: VELORA_DATABASE_QUESTDB_HOST
    pub host: String,

    /// PostgreSQL wire protocol port (default: 8812)
    /// Env: VELORA_DATABASE_QUESTDB_PG_PORT
    pub pg_port: u16,

    /// HTTP API port (default: 9000)
    /// Env: VELORA_DATABASE_QUESTDB_HTTP_PORT
    pub http_port: u16,

    /// InfluxDB line protocol port (default: 9009)
    /// Env: VELORA_DATABASE_QUESTDB_ILP_PORT
    pub ilp_port: u16,

    /// Database name
    /// Env: VELORA_DATABASE_QUESTDB_DATABASE
    pub database: String,

    /// Username (optional)
    /// Env: VELORA_DATABASE_QUESTDB_USERNAME
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,

    /// Password (optional)
    /// Env: VELORA_DATABASE_QUESTDB_PASSWORD
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    /// Connection pool size
    /// Env: VELORA_DATABASE_QUESTDB_POOL_SIZE
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,
}

impl Default for QuestDbConfig {
    fn default() -> Self {
        QuestDbConfig {
            host: "localhost".to_string(),
            pg_port: 8812,
            http_port: 9000,
            ilp_port: 9009,
            database: "qdb".to_string(),
            username: Some("admin".to_string()),
            password: Some("quest".to_string()),
            pool_size: 10,
        }
    }
}

/// TimescaleDB connection configuration.
///
/// TimescaleDB provides PostgreSQL-based time-series capabilities with
/// ACID guarantees, suitable for production trading systems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimescaleDbConfig {
    /// PostgreSQL host
    /// Env: VELORA_DATABASE_TIMESCALEDB_HOST
    pub host: String,

    /// PostgreSQL port (default: 5432)
    /// Env: VELORA_DATABASE_TIMESCALEDB_PORT
    pub port: u16,

    /// Database name
    /// Env: VELORA_DATABASE_TIMESCALEDB_DATABASE
    pub database: String,

    /// Username
    /// Env: VELORA_DATABASE_TIMESCALEDB_USERNAME
    pub username: String,

    /// Password
    /// Env: VELORA_DATABASE_TIMESCALEDB_PASSWORD
    pub password: String,

    /// Connection pool size
    /// Env: VELORA_DATABASE_TIMESCALEDB_POOL_SIZE
    #[serde(default = "default_pool_size")]
    pub pool_size: u32,

    /// Enable SSL/TLS
    /// Env: VELORA_DATABASE_TIMESCALEDB_SSL
    #[serde(default)]
    pub ssl: bool,
}

impl Default for TimescaleDbConfig {
    fn default() -> Self {
        TimescaleDbConfig {
            host: "localhost".to_string(),
            port: 5432,
            database: "velora".to_string(),
            username: "postgres".to_string(),
            password: "".to_string(),
            pool_size: 10,
            ssl: false,
        }
    }
}

fn default_pool_size() -> u32 {
    10
}
