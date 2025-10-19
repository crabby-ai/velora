//! # velora-exchange
//!
//! Exchange integrations and connectivity for the Velora HFT platform.
//!
//! This crate provides a unified interface for connecting to multiple cryptocurrency exchanges
//! and decentralized trading protocols, supporting:
//! - **Centralized Exchanges (CEX)**: Binance (Spot, Futures, Options)
//! - **zkRollup DEX**: Lighter (Perpetuals, Spot)
//! - **Starknet L2 DEX**: Paradex (Perpetuals)
//!
//! ## Features
//!
//! - **Multi-Exchange Support**: Seamless switching between different trading platforms
//! - **Multi-Instrument Support**: Spot, Perpetuals, Futures, Options with unified interface
//! - **Instrument-Agnostic API**: Same methods for candles, orderbooks, trades across all instruments
//! - **Real-time Streaming**: WebSocket support for live market data and account updates
//! - **Flexible Authentication**: API keys for CEX, wallet signing for DEX (EVM, Starknet)
//!
//! ## Example
//!
//! ```rust,no_run
//! use velora_exchange::{ExchangeFactory, ExchangeType, InstrumentType};
//! use velora_exchange::types::Symbol;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a Lighter exchange client
//!     let mut exchange = ExchangeFactory::create(
//!         ExchangeType::DexZk,
//!         "lighter",
//!         /* auth config */
//!     ).await?;
//!
//!     // Connect
//!     exchange.connect().await?;
//!
//!     // Get candles - same API for all instruments!
//!     let symbol = Symbol::new("BTC-USD-PERP");
//!     let candles = exchange.market_data()
//!         .get_candles(&symbol, Interval::Minutes(5), None, None, Some(100))
//!         .await?;
//!
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

// Core modules
pub mod auth;
pub mod common;
pub mod traits;
pub mod types;

// Exchange implementations
pub mod exchanges;

// Re-export commonly used types
pub use types::{
    AccountInfo, AccountType, Balance, Candle, ExchangeError, ExchangeType, FundingRate,
    InstrumentInfo, InstrumentType, Interval, MarginType, Market, MarketStatus, NewOrder,
    OptionType, Order, OrderBook, OrderStatus, OrderType, Position, PositionSide, Price, Result,
    Side, Symbol, Ticker, TimeInForce, Trade,
};

// Re-export traits
pub use traits::{Account, Exchange, MarketData, Streaming, Trading};

// Re-export auth types
pub use auth::{ApiKeyAuth, AuthConfig, EvmWalletAuth, StarknetWalletAuth};

// Re-export common utilities
pub use common::{ExchangeFactory, RateLimiter};

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::auth::*;
    pub use crate::common::*;
    pub use crate::traits::*;
    pub use crate::types::*;
}
