//! Lighter-specific API request/response types

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// API response wrapper - all Lighter API responses are wrapped in this structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterApiResponse<T> {
    pub code: i32,
    #[serde(flatten)]
    pub data: T,
}

/// Wrapper for orderBooks endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBooksData {
    pub order_books: Vec<LighterOrderBookInfo>,
}

/// Wrapper for orderBookDetails endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookDetailsData {
    pub order_book_details: Vec<LighterOrderBookDetails>,
}

/// Lighter orderbook info response (from /orderBooks)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterOrderBookInfo {
    pub symbol: String,
    pub market_id: u64,
    pub status: String,
    pub taker_fee: String,
    pub maker_fee: String,
    pub liquidation_fee: String,
    pub min_base_amount: String,
    pub min_quote_amount: String,
    pub order_quote_limit: String,
    pub supported_size_decimals: u32,
    pub supported_price_decimals: u32,
    pub supported_quote_decimals: u32,
}

/// Lighter orderbook details response (from /orderBookDetails)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterOrderBookDetails {
    pub symbol: String,
    pub market_id: u64,
    pub status: String,
    pub taker_fee: String,
    pub maker_fee: String,
    pub liquidation_fee: String,
    pub min_base_amount: String,
    pub min_quote_amount: String,
    pub order_quote_limit: String,
    pub supported_size_decimals: u32,
    pub supported_price_decimals: u32,
    pub supported_quote_decimals: u32,
    pub size_decimals: u32,
    pub price_decimals: u32,
    pub quote_multiplier: u32,
    pub default_initial_margin_fraction: u32,
    pub min_initial_margin_fraction: u32,
    pub maintenance_margin_fraction: u32,
    pub closeout_margin_fraction: u32,
    pub last_trade_price: Option<f64>,
    pub daily_trades_count: Option<u64>,
    pub daily_base_token_volume: Option<f64>,
    pub daily_quote_token_volume: Option<f64>,
    pub daily_price_low: Option<f64>,
    pub daily_price_high: Option<f64>,
    pub daily_price_change: Option<f64>,
    pub open_interest: Option<f64>,
}

/// Wrapper for orderBookOrders endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookOrdersData {
    pub total_asks: u64,
    pub asks: Vec<LighterOrder>,
    pub total_bids: u64,
    pub bids: Vec<LighterOrder>,
}

/// Lighter order in orderbook
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterOrder {
    pub order_id: String,
    pub price: String,
    pub remaining_base_amount: String,
}

/// Wrapper for candlesticks endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandlesticksData {
    pub resolution: String,
    pub candlesticks: Vec<LighterCandlestick>,
}

/// Lighter candlestick response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterCandlestick {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume0: f64, // base volume
    pub volume1: f64, // quote volume
    pub last_trade_id: u64,
}

/// Wrapper for trades endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradesData {
    pub trades: Vec<LighterTrade>,
}

/// Wrapper for fundings endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingsData {
    pub resolution: String,
    pub fundings: Vec<LighterFunding>,
}

/// Lighter funding rate history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterFunding {
    pub timestamp: i64,
    pub value: String,
    pub rate: String,
    pub direction: String,
}

/// Lighter trade response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterTrade {
    pub trade_id: u64,
    pub market_id: u64,
    pub price: String,
    pub size: String,
    pub is_maker_ask: bool,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub timestamp: DateTime<Utc>,
}

/// Lighter account response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterAccount {
    pub account_id: String,
    pub l1_address: String,
    pub balances: Vec<LighterBalance>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterBalance {
    pub token_id: String,
    pub available: String,
    pub locked: String,
}

/// Lighter position response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterPosition {
    pub order_book_id: String,
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub unrealized_pnl: String,
    pub realized_pnl: String,
    pub margin: String,
    pub leverage: i32,
}

/// Lighter active order response (for account orders)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterActiveOrder {
    pub id: String,
    pub order_book_id: String,
    pub client_order_id: Option<String>,
    pub side: String,       // "buy" or "sell"
    pub order_type: String, // "limit" or "market"
    pub price: Option<String>,
    pub size: String,
    pub filled_size: String,
    pub status: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub updated_at: DateTime<Utc>,
}

/// Lighter funding rate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterFundingRate {
    pub order_book_id: String,
    pub funding_rate: String,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub funding_time: DateTime<Utc>,
    #[serde(with = "chrono::serde::ts_milliseconds")]
    pub next_funding_time: DateTime<Utc>,
}

/// Lighter order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LighterOrderRequest {
    pub order_book_id: String,
    pub side: String,
    pub order_type: String,
    pub price: Option<String>,
    pub size: String,
    pub client_order_id: Option<String>,
    pub time_in_force: Option<String>,
}

/// Response indicating feature not available
#[derive(Debug)]
pub struct NotAvailable {
    pub feature: String,
    pub reason: String,
}

impl std::fmt::Display for NotAvailable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Feature '{}' is not available: {}",
            self.feature, self.reason
        )
    }
}

impl std::error::Error for NotAvailable {}
