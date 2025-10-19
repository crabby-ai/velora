//! Paradex-specific API request/response types

use serde::{Deserialize, Serialize};

/// API response wrapper - all Paradex API responses are wrapped in this structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexApiResponse<T> {
    pub results: T,
}

/// Paginated API response wrapper - for endpoints that support pagination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexPaginatedResponse<T> {
    pub results: T,
    #[serde(default)]
    pub next: Option<String>,
    #[serde(default)]
    pub prev: Option<String>,
}

/// Paradex market response (matches actual API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexMarket {
    pub symbol: String,
    pub base_currency: String,
    pub quote_currency: String,
    #[serde(default)]
    pub settlement_currency: String,
    pub order_size_increment: String,
    pub price_tick_size: String,
    pub min_notional: String,
    pub asset_kind: String, // "PERP", "PERP_OPTION"
    #[serde(default)]
    pub market_kind: String, // "cross"
    #[serde(default)]
    pub max_order_size: String,
}

/// Paradex markets summary response (matches actual API format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexMarketSummary {
    pub symbol: String,
    #[serde(default)]
    pub mark_price: String,
    #[serde(default)]
    pub last_traded_price: String,
    #[serde(default)]
    pub bid: String,
    #[serde(default)]
    pub ask: String,
    #[serde(default)]
    pub volume_24h: String,
    #[serde(default)]
    pub open_interest: String,
    #[serde(default)]
    pub funding_rate: String,
    #[serde(default)]
    pub underlying_price: String,
}

/// Paradex orderbook response (bids/asks are arrays of [price, size])
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrderBook {
    pub market: String,
    pub seq_no: u64,
    pub last_updated_at: i64,
    pub bids: Vec<Vec<String>>, // Each entry is [price, size]
    pub asks: Vec<Vec<String>>, // Each entry is [price, size]
}

/// Paradex BBO (Best Bid Offer) response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexBBO {
    pub market: String,
    pub best_bid_price: String,
    pub best_bid_size: String,
    pub best_ask_price: String,
    pub best_ask_size: String,
    pub last_updated_at: i64,
}

/// Paradex OHLC (candlestick) response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOHLC {
    pub market: String,
    pub resolution: String,
    pub start_time: i64,
    pub end_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub trades: Option<i64>,
}

/// Paradex trade response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexTrade {
    pub id: String,
    pub market: String,
    pub price: String,
    pub size: String,
    pub side: String, // "BUY" or "SELL"
    pub created_at: i64,
    #[serde(default)]
    pub trade_type: String, // "FILL", "RPI" (Retail Price Improvement), etc.
}

/// Paradex account response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexAccount {
    pub account_id: String,
    pub starknet_address: String,
    pub created_at: i64,
}

/// Paradex balance response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexBalance {
    pub asset: String,
    pub available: String,
    pub locked: String,
    pub total: String,
}

/// Paradex position response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexPosition {
    pub market: String,
    pub side: String, // "LONG" or "SHORT"
    pub size: String,
    pub entry_price: String,
    pub mark_price: String,
    pub liquidation_price: Option<String>,
    pub unrealized_pnl: String,
    pub realized_pnl: String,
    pub margin: String,
    pub leverage: String,
    pub last_updated_at: i64,
}

/// Paradex order response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrder {
    pub id: String,
    pub client_id: Option<String>,
    pub market: String,
    pub side: String,   // "BUY" or "SELL"
    pub r#type: String, // "LIMIT" or "MARKET"
    pub price: Option<String>,
    pub size: String,
    pub filled_size: String,
    pub remaining_size: String,
    pub status: String,
    pub time_in_force: String,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Paradex new order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrderRequest {
    pub market: String,
    pub side: String,   // "BUY" or "SELL"
    pub r#type: String, // "LIMIT" or "MARKET"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<String>,
    pub size: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_only: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,
}

/// Paradex funding payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFundingPayment {
    pub market: String,
    pub payment: String,
    pub rate: String,
    pub position_size: String,
    pub timestamp: i64,
}

/// Paradex funding rate response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFunding {
    pub market: String,
    pub funding_rate: String,
    pub funding_rate_8h: String,
    pub next_funding_time: i64,
    pub timestamp: i64,
}

/// Paradex fill (trade execution) response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexFill {
    pub id: String,
    pub order_id: String,
    pub market: String,
    pub side: String,
    pub price: String,
    pub size: String,
    pub fee: String,
    pub liquidity: String, // "MAKER" or "TAKER"
    pub created_at: i64,
}

/// Paradex JWT auth request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexAuthRequest {
    pub starknet_address: String,
    pub starknet_signature: String,
    pub timestamp: i64,
}

/// Paradex JWT auth response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexAuthResponse {
    pub jwt_token: String,
    pub refresh_token: String,
    pub expires_at: i64,
}

// ============================================================================
// WebSocket Message Types
// ============================================================================

/// WebSocket subscription request (JSON-RPC 2.0 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsSubscribe {
    pub jsonrpc: String, // "2.0"
    pub method: String,  // "subscribe"
    pub params: ParadexWsSubscribeParams,
    pub id: u64,
}

/// Subscription parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsSubscribeParams {
    pub channel: String, // "trades@BTC-USD-PERP", "orderbook@BTC-USD-PERP", etc.
}

/// WebSocket message wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsMessage {
    pub channel: String,
    #[serde(flatten)]
    pub data: serde_json::Value,
}

/// Trades channel subscription message (JSON-RPC 2.0 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsTrade {
    pub jsonrpc: String,
    pub method: String,
    pub params: ParadexWsTradeParams,
}

/// Trade subscription parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsTradeParams {
    pub channel: String,
    pub data: ParadexTrade,
}

/// OrderBook snapshot/update message (JSON-RPC 2.0 format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsOrderBook {
    pub jsonrpc: String,
    pub method: String,
    pub params: ParadexWsOrderBookParams,
}

/// OrderBook subscription parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsOrderBookParams {
    pub channel: String,
    pub data: ParadexOrderBookData,
}

/// OrderBook data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexOrderBookData {
    pub market: String,
    pub seq_no: u64,
    pub last_updated_at: i64,
    pub bids: Vec<Vec<String>>, // [["price", "size"], ...]
    pub asks: Vec<Vec<String>>,
}

/// Ticker channel message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParadexWsTicker {
    pub channel: String,
    pub market: String,
    pub last_price: String,
    pub bid: String,
    pub ask: String,
    pub volume_24h: String,
    pub price_change_24h: String,
    pub timestamp: i64,
}
