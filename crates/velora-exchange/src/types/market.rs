//! Market and ticker information types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Trading market/pair information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Market {
    /// Symbol (e.g., "BTC/USDT", "ETH-PERP")
    pub symbol: Symbol,

    /// Base asset (e.g., "BTC")
    pub base_asset: String,

    /// Quote asset (e.g., "USDT")
    pub quote_asset: String,

    /// Instrument type
    pub instrument_type: InstrumentType,

    /// Market status
    pub status: MarketStatus,

    // Trading rules
    /// Minimum order quantity
    pub min_quantity: Decimal,

    /// Maximum order quantity
    pub max_quantity: Decimal,

    /// Quantity step size (lot size)
    pub step_size: Decimal,

    /// Price tick size
    pub tick_size: Decimal,

    /// Minimum notional value
    pub min_notional: Decimal,

    /// Instrument-specific information
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instrument_info: Option<InstrumentInfo>,
}

/// Real-time ticker information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ticker {
    /// Symbol
    pub symbol: Symbol,

    /// Last traded price
    pub last_price: Price,

    /// Best bid price
    pub bid: Price,

    /// Best bid size
    pub bid_size: Decimal,

    /// Best ask price
    pub ask: Price,

    /// Best ask size
    pub ask_size: Decimal,

    /// 24h trading volume
    pub volume_24h: Decimal,

    /// 24h high price
    pub high_24h: Price,

    /// 24h low price
    pub low_24h: Price,

    /// 24h price change (absolute)
    pub price_change_24h: Decimal,

    /// 24h price change (percentage)
    pub price_change_percent_24h: Decimal,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
