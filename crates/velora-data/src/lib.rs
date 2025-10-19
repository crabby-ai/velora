//! # velora-data
//!
//! Market data ingestion, storage, aggregation, and streaming for the Velora HFT platform.
//!
//! This crate handles all aspects of market data management.
//!
//! ## Storage Backends
//!
//! The storage module provides a trait-based abstraction for storing and retrieving
//! market data with multiple backend implementations:
//!
//! - **QuestDB**: High-performance time series database optimized for tick data
//! - **TimescaleDB**: PostgreSQL-based ACID-compliant database (planned)
//! - **InMemory**: Fast in-memory storage for testing and backtesting (planned)
//!
//! ## Example
//!
//! ```no_run
//! use velora_data::storage::{DataStorage, questdb::QuestDbStorage};
//! use velora_core::{QuestDbConfig, Symbol, Tick};
//! use chrono::Utc;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create storage backend
//! let config = QuestDbConfig::default();
//! let mut storage = QuestDbStorage::new(config).await?;
//! storage.initialize().await?;
//!
//! // Store a tick
//! let tick = Tick {
//!     symbol: Symbol("BTCUSD".to_string()),
//!     price: 50000.0.into(),
//!     volume: 1.5.into(),
//!     timestamp: Utc::now(),
//! };
//!
//! storage.store_tick(&tick).await?;
//! # Ok(())
//! # }
//! ```

#![warn(missing_docs)]

pub mod storage;
