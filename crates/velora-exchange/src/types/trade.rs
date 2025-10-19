//! Trade types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Public trade from market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Trade ID
    pub trade_id: String,

    /// Symbol
    pub symbol: Symbol,

    /// Price
    pub price: Price,

    /// Quantity
    pub quantity: Decimal,

    /// Side (inferred from buyer_maker)
    pub side: Side,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// True if buyer is maker
    #[serde(skip_serializing_if = "Option::is_none")]
    pub buyer_maker: Option<bool>,
}

/// Trade event from WebSocket stream
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamTrade {
    /// Symbol
    pub symbol: Symbol,

    /// Trade ID
    pub trade_id: String,

    /// Price
    pub price: Price,

    /// Quantity
    pub quantity: Decimal,

    /// Side
    pub side: Side,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// True if buyer is maker
    pub buyer_maker: bool,
}

/// User's trade execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeExecution {
    /// Trade ID
    pub trade_id: String,

    /// Order ID
    pub order_id: String,

    /// Symbol
    pub symbol: Symbol,

    /// Side
    pub side: Side,

    /// Execution price
    pub price: Price,

    /// Executed quantity
    pub quantity: Decimal,

    /// Commission/fee paid
    pub fee: Decimal,

    /// Commission asset
    pub fee_asset: String,

    /// Is maker trade
    pub is_maker: bool,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
