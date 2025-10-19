//! Lighter Exchange implementation (zkRollup DEX)
//!
//! Lighter is a zkRollup-based decentralized exchange supporting:
//! - Perpetual contracts
//! - Spot trading
//! - EVM wallet authentication

mod account;
mod client;
mod market_data;
#[cfg(test)]
mod market_data_test;
mod streaming;
mod trading;
mod types;

pub use client::LighterExchange;

use crate::types::InstrumentType;

/// Lighter API endpoints
pub(crate) mod endpoints {
    // Mainnet configuration
    pub const REST_BASE_URL: &str = "https://mainnet.zklighter.elliot.ai";
    pub const WS_BASE_URL: &str = "wss://mainnet.zklighter.elliot.ai";

    // Market data endpoints (note: camelCase naming)
    pub const ORDERBOOKS: &str = "/api/v1/orderBooks";
    pub const ORDERBOOK_DETAILS: &str = "/api/v1/orderBookDetails";
    pub const ORDERBOOK_ORDERS: &str = "/api/v1/orderBookOrders";
    pub const RECENT_TRADES: &str = "/api/v1/recentTrades";
    pub const TRADES: &str = "/api/v1/trades";
    pub const CANDLESTICKS: &str = "/api/v1/candlesticks";
    pub const FUNDING_RATES: &str = "/api/v1/funding-rates";
    pub const FUNDINGS: &str = "/api/v1/fundings";
    pub const EXCHANGE_STATS: &str = "/api/v1/exchangeStats";

    // Trading endpoints
    pub const SEND_TX: &str = "/api/v1/sendtx";
    pub const SEND_TX_BATCH: &str = "/api/v1/sendtxbatch";
    pub const ACCOUNT_ACTIVE_ORDERS: &str = "/api/v1/accountActiveOrders";
    pub const ACCOUNT_INACTIVE_ORDERS: &str = "/api/v1/accountInactiveOrders";

    // Account endpoints
    pub const ACCOUNT: &str = "/api/v1/account";
    pub const ACCOUNTS_BY_L1: &str = "/api/v1/accountsByL1Address";
    pub const ACCOUNT_LIMITS: &str = "/api/v1/accountLimits";
    pub const ACCOUNT_METADATA: &str = "/api/v1/accountMetadata";
    pub const PNL: &str = "/api/v1/pnl";
    pub const LIQUIDATIONS: &str = "/api/v1/liquidations";
    pub const POSITION_FUNDING: &str = "/api/v1/positionFunding";
    pub const ACCOUNT_TXS: &str = "/api/v1/accounttxs";
    pub const DEPOSIT_HISTORY: &str = "/api/v1/deposit_history";
    pub const WITHDRAW_HISTORY: &str = "/api/v1/withdraw_history";
    pub const TRANSFER_HISTORY: &str = "/api/v1/transfer_history";

    // System endpoints
    pub const STATUS: &str = "/api/v1/status";
    pub const CURRENT_HEIGHT: &str = "/api/v1/currentheight";
    pub const BLOCK: &str = "/api/v1/block";
    pub const BLOCKS: &str = "/api/v1/blocks";
}

/// Supported instruments on Lighter
pub(crate) const SUPPORTED_INSTRUMENTS: &[InstrumentType] =
    &[InstrumentType::Spot, InstrumentType::Perpetual];
