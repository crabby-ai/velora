//! Paradex Exchange implementation (Starknet L2 DEX)
//!
//! Paradex is a Starknet L2-based decentralized exchange supporting:
//! - Perpetual contracts
//! - Starknet wallet authentication

mod account;
mod client;
mod market_data;
mod streaming;
mod trading;
pub(crate) mod types;

pub use client::ParadexExchange;

use crate::types::InstrumentType;

/// Paradex API endpoints
pub(crate) mod endpoints {
    // Based on official Paradex documentation at docs.paradex.trade
    pub const REST_BASE_URL: &str = "https://api.prod.paradex.trade";
    pub const WS_BASE_URL: &str = "wss://ws.api.prod.paradex.trade/v1";

    // Authentication
    pub const AUTH_JWT: &str = "/v1/auth";

    // Market data endpoints
    pub const MARKETS: &str = "/v1/markets";
    pub const MARKETS_SUMMARY: &str = "/v1/markets/summary";
    pub const ORDERBOOK: &str = "/v1/orderbook"; // /:market
    pub const BBO: &str = "/v1/bbo"; // /:market - Best bid/offer
    pub const TRADES: &str = "/v1/trades"; // /:market
    pub const OHLC: &str = "/v1/ohlc"; // /:market - Candlesticks
    pub const FUNDING: &str = "/v1/funding"; // /:market
    pub const IMPACT_PRICE: &str = "/v1/impact-price"; // /:market

    // Trading endpoints
    pub const ORDERS: &str = "/v1/orders";
    pub const CANCEL_ORDER: &str = "/v1/orders"; // DELETE /:id
    pub const CANCEL_ALL: &str = "/v1/orders";
    pub const MODIFY_ORDER: &str = "/v1/orders"; // PATCH /:id
    pub const ALGO_ORDERS: &str = "/v1/algo-orders";
    pub const BLOCK_TRADE: &str = "/v1/block-trade";

    // Account endpoints
    pub const ACCOUNT: &str = "/v1/account";
    pub const ACCOUNT_PROFILE: &str = "/v1/account/profile";
    pub const MARGIN: &str = "/v1/account/margin";
    pub const BALANCES: &str = "/v1/account/balance";
    pub const POSITIONS: &str = "/v1/account/positions";
    pub const FILLS: &str = "/v1/fills";
    pub const FUNDING_PAYMENTS: &str = "/v1/funding";
    pub const PNL: &str = "/v1/pnl";
    pub const TRANSFERS: &str = "/v1/transfers";
    pub const TRANSACTIONS: &str = "/v1/transactions";

    // System endpoints
    pub const SYSTEM_STATE: &str = "/v1/system/state";
    pub const SYSTEM_TIME: &str = "/v1/system/time";
    pub const SYSTEM_CONFIG: &str = "/v1/system/config";
}

/// Supported instruments on Paradex
pub(crate) const SUPPORTED_INSTRUMENTS: &[InstrumentType] = &[InstrumentType::Perpetual];
