//! Storage backends for market data persistence.
//!
//! This module provides a trait-based abstraction for storing and retrieving
//! market data (trades, ticks, candles, order books) with multiple backend implementations.
//!
//! # Available Backends
//!
//! - **InMemory**: Fast in-memory storage for testing and backtesting
//! - **QuestDB**: High-performance time series database optimized for tick data
//! - **TimescaleDB**: PostgreSQL-based ACID-compliant database for transactional data

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use velora_core::{Candle, Symbol, Tick, Trade};

pub mod questdb;

/// Result type for storage operations.
pub type StorageResult<T> = Result<T, StorageError>;

/// Errors that can occur during storage operations.
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    /// Connection error
    #[error("Connection error: {0}")]
    Connection(String),

    /// Query error
    #[error("Query error: {0}")]
    Query(String),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Configuration error
    #[error("Configuration error: {0}")]
    Configuration(String),

    /// Data not found
    #[error("Data not found: {0}")]
    NotFound(String),

    /// Generic database error
    #[error("Database error: {0}")]
    Database(String),
}

/// Trait for storing and retrieving market data.
///
/// This trait abstracts the underlying storage mechanism, allowing for
/// different implementations (in-memory, QuestDB, TimescaleDB, etc.).
#[async_trait]
pub trait DataStorage: Send + Sync {
    /// Initialize the storage backend.
    ///
    /// This should create necessary tables, indexes, and perform any
    /// required setup operations.
    async fn initialize(&mut self) -> StorageResult<()>;

    /// Store a single tick.
    async fn store_tick(&self, tick: &Tick) -> StorageResult<()>;

    /// Store multiple ticks in a batch.
    ///
    /// This is more efficient than calling `store_tick` multiple times.
    async fn store_ticks(&self, ticks: &[Tick]) -> StorageResult<()>;

    /// Store a single trade.
    async fn store_trade(&self, trade: &Trade) -> StorageResult<()>;

    /// Store multiple trades in a batch.
    async fn store_trades(&self, trades: &[Trade]) -> StorageResult<()>;

    /// Store a single candle.
    async fn store_candle(&self, candle: &Candle) -> StorageResult<()>;

    /// Store multiple candles in a batch.
    async fn store_candles(&self, candles: &[Candle]) -> StorageResult<()>;

    /// Retrieve ticks for a symbol within a time range.
    ///
    /// # Arguments
    ///
    /// * `symbol` - The trading symbol to query
    /// * `start` - Start of the time range (inclusive)
    /// * `end` - End of the time range (exclusive)
    /// * `limit` - Maximum number of ticks to return (None for no limit)
    async fn get_ticks(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Tick>>;

    /// Retrieve trades for a symbol within a time range.
    async fn get_trades(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Trade>>;

    /// Retrieve candles for a symbol within a time range.
    async fn get_candles(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Candle>>;

    /// Get the latest tick for a symbol.
    async fn get_latest_tick(&self, symbol: &Symbol) -> StorageResult<Option<Tick>>;

    /// Get the latest trade for a symbol.
    async fn get_latest_trade(&self, symbol: &Symbol) -> StorageResult<Option<Trade>>;

    /// Get the latest candle for a symbol.
    async fn get_latest_candle(&self, symbol: &Symbol) -> StorageResult<Option<Candle>>;

    /// Get the earliest timestamp for a symbol.
    ///
    /// This is useful for determining the available data range.
    async fn get_earliest_timestamp(&self, symbol: &Symbol)
        -> StorageResult<Option<DateTime<Utc>>>;

    /// Get the latest timestamp for a symbol.
    async fn get_latest_timestamp(&self, symbol: &Symbol) -> StorageResult<Option<DateTime<Utc>>>;

    /// Delete ticks for a symbol within a time range.
    ///
    /// This is useful for data cleanup or retention policies.
    async fn delete_ticks(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64>;

    /// Delete trades for a symbol within a time range.
    async fn delete_trades(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64>;

    /// Delete candles for a symbol within a time range.
    async fn delete_candles(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64>;

    /// Close the storage connection and cleanup resources.
    async fn close(&mut self) -> StorageResult<()>;
}
