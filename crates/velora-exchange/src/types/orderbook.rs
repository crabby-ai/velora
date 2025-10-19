//! Order book types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Level 2 order book (aggregated by price level)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Symbol
    pub symbol: Symbol,

    /// Bid levels (sorted descending by price)
    pub bids: Vec<PriceLevel>,

    /// Ask levels (sorted ascending by price)
    pub asks: Vec<PriceLevel>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,

    /// Last update ID (for synchronization)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_update_id: Option<u64>,
}

impl OrderBook {
    /// Get the best bid (highest buy price)
    pub fn best_bid(&self) -> Option<&PriceLevel> {
        self.bids.first()
    }

    /// Get the best ask (lowest sell price)
    pub fn best_ask(&self) -> Option<&PriceLevel> {
        self.asks.first()
    }

    /// Get the mid price (average of best bid and ask)
    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => {
                let mid = (bid.price.0 + ask.price.0) / 2.0;
                Some(Price::from(mid))
            }
            _ => None,
        }
    }

    /// Get the spread (difference between best ask and bid)
    pub fn spread(&self) -> Option<Decimal> {
        match (self.best_bid(), self.best_ask()) {
            (Some(bid), Some(ask)) => Decimal::try_from(ask.price.0 - bid.price.0).ok(),
            _ => None,
        }
    }
}

/// Price level in order book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceLevel {
    /// Price
    pub price: Price,

    /// Quantity at this price level
    pub quantity: Decimal,
}

/// Order book update event (for streaming)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookUpdate {
    /// Symbol
    pub symbol: Symbol,

    /// Updated bid levels
    pub bids: Vec<PriceLevel>,

    /// Updated ask levels
    pub asks: Vec<PriceLevel>,

    /// First update ID in this event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_update_id: Option<u64>,

    /// Final update ID in this event
    #[serde(skip_serializing_if = "Option::is_none")]
    pub final_update_id: Option<u64>,

    /// Timestamp
    pub timestamp: DateTime<Utc>,
}
