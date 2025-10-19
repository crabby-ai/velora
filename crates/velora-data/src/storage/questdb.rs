//! QuestDB storage backend implementation.
//!
//! This module provides a high-performance storage backend using QuestDB,
//! optimized for storing high-frequency tick data and trades.
//!
//! # Features
//!
//! - PostgreSQL wire protocol for maximum compatibility
//! - Automatic table creation with proper partitioning
//! - Batch insert support for high throughput
//! - Connection pooling for concurrent access
//!
//! # Example
//!
//! ```no_run
//! use velora_data::storage::questdb::QuestDbStorage;
//! use velora_core::QuestDbConfig;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = QuestDbConfig::default();
//! let mut storage = QuestDbStorage::new(config).await?;
//! storage.initialize().await?;
//! # Ok(())
//! # }
//! ```

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::postgres::{PgPool, PgPoolOptions};
use sqlx::Row;
use velora_core::{Candle, QuestDbConfig, Symbol, Tick, Trade};

use super::{DataStorage, StorageError, StorageResult};

/// QuestDB storage implementation using PostgreSQL wire protocol.
pub struct QuestDbStorage {
    pool: PgPool,
    #[allow(dead_code)]
    config: QuestDbConfig,
}

impl QuestDbStorage {
    /// Create a new QuestDB storage instance.
    ///
    /// # Arguments
    ///
    /// * `config` - QuestDB connection configuration
    ///
    /// # Errors
    ///
    /// Returns `StorageError::Connection` if the connection cannot be established.
    pub async fn new(config: QuestDbConfig) -> StorageResult<Self> {
        let connection_string = Self::build_connection_string(&config);

        let pool = PgPoolOptions::new()
            .max_connections(config.pool_size)
            .connect(&connection_string)
            .await
            .map_err(|e| StorageError::Connection(format!("Failed to connect to QuestDB: {e}")))?;

        Ok(Self { pool, config })
    }

    /// Build PostgreSQL connection string from config.
    fn build_connection_string(config: &QuestDbConfig) -> String {
        let user = config.username.as_deref().unwrap_or("admin");
        let password = config.password.as_deref().unwrap_or("quest");

        format!(
            "postgresql://{}:{}@{}:{}/{}",
            user, password, config.host, config.pg_port, config.database
        )
    }

    /// Create the ticks table with proper schema.
    async fn create_ticks_table(&self) -> StorageResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ticks (
                symbol SYMBOL,
                price DOUBLE,
                volume DOUBLE,
                timestamp TIMESTAMP
            ) timestamp(timestamp) PARTITION BY DAY
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to create ticks table: {e}")))?;

        Ok(())
    }

    /// Create the trades table with proper schema.
    async fn create_trades_table(&self) -> StorageResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS trades (
                id SYMBOL,
                order_id SYMBOL,
                symbol SYMBOL,
                side SYMBOL,
                price DOUBLE,
                quantity DOUBLE,
                fee DOUBLE,
                timestamp TIMESTAMP
            ) timestamp(timestamp) PARTITION BY DAY
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to create trades table: {e}")))?;

        Ok(())
    }

    /// Create the candles table with proper schema.
    async fn create_candles_table(&self) -> StorageResult<()> {
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS candles (
                symbol SYMBOL,
                open DOUBLE,
                high DOUBLE,
                low DOUBLE,
                close DOUBLE,
                volume DOUBLE,
                timestamp TIMESTAMP
            ) timestamp(timestamp) PARTITION BY DAY
            "#,
        )
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to create candles table: {e}")))?;

        Ok(())
    }
}

#[async_trait]
impl DataStorage for QuestDbStorage {
    async fn initialize(&mut self) -> StorageResult<()> {
        self.create_ticks_table().await?;
        self.create_trades_table().await?;
        self.create_candles_table().await?;
        Ok(())
    }

    async fn store_tick(&self, tick: &Tick) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO ticks (symbol, price, volume, timestamp)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(tick.symbol.0.as_str())
        .bind(tick.price.0)
        .bind(tick.volume.0)
        .bind(tick.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to store tick: {e}")))?;

        Ok(())
    }

    async fn store_ticks(&self, ticks: &[Tick]) -> StorageResult<()> {
        if ticks.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to begin transaction: {e}")))?;

        for tick in ticks {
            sqlx::query(
                r#"
                INSERT INTO ticks (symbol, price, volume, timestamp)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(tick.symbol.0.as_str())
            .bind(tick.price.0)
            .bind(tick.volume.0)
            .bind(tick.timestamp)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to store tick: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to commit transaction: {e}")))?;

        Ok(())
    }

    async fn store_trade(&self, trade: &Trade) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO trades (id, order_id, symbol, side, price, quantity, fee, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            "#,
        )
        .bind(trade.id.to_string())
        .bind(trade.order_id.to_string())
        .bind(trade.symbol.0.as_str())
        .bind(format!("{:?}", trade.side))
        .bind(trade.price.0)
        .bind(trade.quantity.0)
        .bind(trade.fee.0)
        .bind(trade.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to store trade: {e}")))?;

        Ok(())
    }

    async fn store_trades(&self, trades: &[Trade]) -> StorageResult<()> {
        if trades.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to begin transaction: {e}")))?;

        for trade in trades {
            sqlx::query(
                r#"
                INSERT INTO trades (id, order_id, symbol, side, price, quantity, fee, timestamp)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(trade.id.to_string())
            .bind(trade.order_id.to_string())
            .bind(trade.symbol.0.as_str())
            .bind(format!("{:?}", trade.side))
            .bind(trade.price.0)
            .bind(trade.quantity.0)
            .bind(trade.fee.0)
            .bind(trade.timestamp)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to store trade: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to commit transaction: {e}")))?;

        Ok(())
    }

    async fn store_candle(&self, candle: &Candle) -> StorageResult<()> {
        sqlx::query(
            r#"
            INSERT INTO candles (symbol, open, high, low, close, volume, timestamp)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(candle.symbol.0.as_str())
        .bind(candle.open.0)
        .bind(candle.high.0)
        .bind(candle.low.0)
        .bind(candle.close.0)
        .bind(candle.volume.0)
        .bind(candle.timestamp)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to store candle: {e}")))?;

        Ok(())
    }

    async fn store_candles(&self, candles: &[Candle]) -> StorageResult<()> {
        if candles.is_empty() {
            return Ok(());
        }

        let mut tx = self
            .pool
            .begin()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to begin transaction: {e}")))?;

        for candle in candles {
            sqlx::query(
                r#"
                INSERT INTO candles (symbol, open, high, low, close, volume, timestamp)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(candle.symbol.0.as_str())
            .bind(candle.open.0)
            .bind(candle.high.0)
            .bind(candle.low.0)
            .bind(candle.close.0)
            .bind(candle.volume.0)
            .bind(candle.timestamp)
            .execute(&mut *tx)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to store candle: {e}")))?;
        }

        tx.commit()
            .await
            .map_err(|e| StorageError::Database(format!("Failed to commit transaction: {e}")))?;

        Ok(())
    }

    async fn get_ticks(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Tick>> {
        let limit_clause = limit.map(|l| format!("LIMIT {l}")).unwrap_or_default();

        let query = format!(
            r#"
            SELECT symbol, price, volume, timestamp
            FROM ticks
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            ORDER BY timestamp ASC
            {limit_clause}
            "#
        );

        let rows = sqlx::query(&query)
            .bind(symbol.0.as_str())
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to fetch ticks: {e}")))?;

        let ticks = rows
            .into_iter()
            .map(|row| {
                Ok(Tick {
                    symbol: Symbol(row.get::<String, _>("symbol")),
                    price: row.get::<f64, _>("price").into(),
                    volume: row.get::<f64, _>("volume").into(),
                    timestamp: row.get("timestamp"),
                })
            })
            .collect::<StorageResult<Vec<Tick>>>()?;

        Ok(ticks)
    }

    async fn get_trades(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Trade>> {
        let limit_clause = limit.map(|l| format!("LIMIT {l}")).unwrap_or_default();

        let query = format!(
            r#"
            SELECT id, order_id, symbol, side, price, quantity, fee, timestamp
            FROM trades
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            ORDER BY timestamp ASC
            {limit_clause}
            "#
        );

        let rows = sqlx::query(&query)
            .bind(symbol.0.as_str())
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to fetch trades: {e}")))?;

        let trades = rows
            .into_iter()
            .map(|row| {
                let side_str: String = row.get("side");
                let side = match side_str.as_str() {
                    "Buy" => velora_core::Side::Buy,
                    "Sell" => velora_core::Side::Sell,
                    _ => {
                        return Err(StorageError::Serialization(format!(
                            "Invalid side: {side_str}"
                        )))
                    }
                };

                Ok(Trade {
                    id: row
                        .get::<String, _>("id")
                        .parse()
                        .map_err(|e| StorageError::Serialization(format!("Invalid id: {e}")))?,
                    order_id: row.get::<String, _>("order_id").parse().map_err(|e| {
                        StorageError::Serialization(format!("Invalid order_id: {e}"))
                    })?,
                    symbol: Symbol(row.get::<String, _>("symbol")),
                    side,
                    price: row.get::<f64, _>("price").into(),
                    quantity: row.get::<f64, _>("quantity").into(),
                    fee: row.get::<f64, _>("fee").into(),
                    timestamp: row.get("timestamp"),
                })
            })
            .collect::<StorageResult<Vec<Trade>>>()?;

        Ok(trades)
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
        limit: Option<usize>,
    ) -> StorageResult<Vec<Candle>> {
        let limit_clause = limit.map(|l| format!("LIMIT {l}")).unwrap_or_default();

        let query = format!(
            r#"
            SELECT symbol, open, high, low, close, volume, timestamp
            FROM candles
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            ORDER BY timestamp ASC
            {limit_clause}
            "#
        );

        let rows = sqlx::query(&query)
            .bind(symbol.0.as_str())
            .bind(start)
            .bind(end)
            .fetch_all(&self.pool)
            .await
            .map_err(|e| StorageError::Query(format!("Failed to fetch candles: {e}")))?;

        let candles = rows
            .into_iter()
            .map(|row| {
                Ok(Candle {
                    symbol: Symbol(row.get::<String, _>("symbol")),
                    open: row.get::<f64, _>("open").into(),
                    high: row.get::<f64, _>("high").into(),
                    low: row.get::<f64, _>("low").into(),
                    close: row.get::<f64, _>("close").into(),
                    volume: row.get::<f64, _>("volume").into(),
                    timestamp: row.get("timestamp"),
                })
            })
            .collect::<StorageResult<Vec<Candle>>>()?;

        Ok(candles)
    }

    async fn get_latest_tick(&self, symbol: &Symbol) -> StorageResult<Option<Tick>> {
        let row = sqlx::query(
            r#"
            SELECT symbol, price, volume, timestamp
            FROM ticks
            WHERE symbol = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(symbol.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to fetch latest tick: {e}")))?;

        match row {
            Some(row) => Ok(Some(Tick {
                symbol: Symbol(row.get::<String, _>("symbol")),
                price: row.get::<f64, _>("price").into(),
                volume: row.get::<f64, _>("volume").into(),
                timestamp: row.get("timestamp"),
            })),
            None => Ok(None),
        }
    }

    async fn get_latest_trade(&self, symbol: &Symbol) -> StorageResult<Option<Trade>> {
        let row = sqlx::query(
            r#"
            SELECT id, order_id, symbol, side, price, quantity, fee, timestamp
            FROM trades
            WHERE symbol = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(symbol.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to fetch latest trade: {e}")))?;

        match row {
            Some(row) => {
                let side_str: String = row.get("side");
                let side = match side_str.as_str() {
                    "Buy" => velora_core::Side::Buy,
                    "Sell" => velora_core::Side::Sell,
                    _ => {
                        return Err(StorageError::Serialization(format!(
                            "Invalid side: {side_str}"
                        )))
                    }
                };

                Ok(Some(Trade {
                    id: row
                        .get::<String, _>("id")
                        .parse()
                        .map_err(|e| StorageError::Serialization(format!("Invalid id: {e}")))?,
                    order_id: row.get::<String, _>("order_id").parse().map_err(|e| {
                        StorageError::Serialization(format!("Invalid order_id: {e}"))
                    })?,
                    symbol: Symbol(row.get::<String, _>("symbol")),
                    side,
                    price: row.get::<f64, _>("price").into(),
                    quantity: row.get::<f64, _>("quantity").into(),
                    fee: row.get::<f64, _>("fee").into(),
                    timestamp: row.get("timestamp"),
                }))
            }
            None => Ok(None),
        }
    }

    async fn get_latest_candle(&self, symbol: &Symbol) -> StorageResult<Option<Candle>> {
        let row = sqlx::query(
            r#"
            SELECT symbol, open, high, low, close, volume, timestamp
            FROM candles
            WHERE symbol = $1
            ORDER BY timestamp DESC
            LIMIT 1
            "#,
        )
        .bind(symbol.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to fetch latest candle: {e}")))?;

        match row {
            Some(row) => Ok(Some(Candle {
                symbol: Symbol(row.get::<String, _>("symbol")),
                open: row.get::<f64, _>("open").into(),
                high: row.get::<f64, _>("high").into(),
                low: row.get::<f64, _>("low").into(),
                close: row.get::<f64, _>("close").into(),
                volume: row.get::<f64, _>("volume").into(),
                timestamp: row.get("timestamp"),
            })),
            None => Ok(None),
        }
    }

    async fn get_earliest_timestamp(
        &self,
        symbol: &Symbol,
    ) -> StorageResult<Option<DateTime<Utc>>> {
        let row = sqlx::query(
            r#"
            SELECT MIN(timestamp) as earliest
            FROM ticks
            WHERE symbol = $1
            "#,
        )
        .bind(symbol.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to fetch earliest timestamp: {e}")))?;

        Ok(row.and_then(|r| r.get("earliest")))
    }

    async fn get_latest_timestamp(&self, symbol: &Symbol) -> StorageResult<Option<DateTime<Utc>>> {
        let row = sqlx::query(
            r#"
            SELECT MAX(timestamp) as latest
            FROM ticks
            WHERE symbol = $1
            "#,
        )
        .bind(symbol.0.as_str())
        .fetch_optional(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to fetch latest timestamp: {e}")))?;

        Ok(row.and_then(|r| r.get("latest")))
    }

    async fn delete_ticks(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM ticks
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            "#,
        )
        .bind(symbol.0.as_str())
        .bind(start)
        .bind(end)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to delete ticks: {e}")))?;

        Ok(result.rows_affected())
    }

    async fn delete_trades(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM trades
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            "#,
        )
        .bind(symbol.0.as_str())
        .bind(start)
        .bind(end)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to delete trades: {e}")))?;

        Ok(result.rows_affected())
    }

    async fn delete_candles(
        &self,
        symbol: &Symbol,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> StorageResult<u64> {
        let result = sqlx::query(
            r#"
            DELETE FROM candles
            WHERE symbol = $1 AND timestamp >= $2 AND timestamp < $3
            "#,
        )
        .bind(symbol.0.as_str())
        .bind(start)
        .bind(end)
        .execute(&self.pool)
        .await
        .map_err(|e| StorageError::Query(format!("Failed to delete candles: {e}")))?;

        Ok(result.rows_affected())
    }

    async fn close(&mut self) -> StorageResult<()> {
        self.pool.close().await;
        Ok(())
    }
}

// Note: Interval parsing removed as Candle struct doesn't include interval field
