//! Order types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// Re-export OrderType from velora-core
pub use velora_core::OrderType;

/// New order request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrder {
    /// Symbol
    pub symbol: Symbol,

    /// Order side
    pub side: Side,

    /// Order type
    pub order_type: OrderType,

    /// Time in force
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_in_force: Option<TimeInForce>,

    /// Order quantity
    pub quantity: Decimal,

    /// Limit price (required for LIMIT orders)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    /// Stop price (for STOP_LOSS, STOP_LOSS_LIMIT)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Price>,

    /// Client order ID (optional, for tracking)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Reduce-only flag (for perpetuals/futures)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,

    /// Position side (for hedge mode)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_side: Option<PositionSide>,
}

impl NewOrder {
    /// Create a new market order
    pub fn market(symbol: Symbol, side: Side, quantity: Decimal) -> Self {
        Self {
            symbol,
            side,
            order_type: OrderType::Market,
            time_in_force: None,
            quantity,
            price: None,
            stop_price: None,
            client_order_id: None,
            reduce_only: None,
            position_side: None,
        }
    }

    /// Create a new limit order
    pub fn limit(symbol: Symbol, side: Side, price: Price, quantity: Decimal) -> Self {
        Self {
            symbol,
            side,
            order_type: OrderType::Limit,
            time_in_force: Some(TimeInForce::GoodTilCancel),
            quantity,
            price: Some(price),
            stop_price: None,
            client_order_id: None,
            reduce_only: None,
            position_side: None,
        }
    }
}

/// Order from exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Exchange order ID
    pub order_id: String,

    /// Client order ID (if provided)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_order_id: Option<String>,

    /// Symbol
    pub symbol: Symbol,

    /// Order side
    pub side: Side,

    /// Order type
    pub order_type: OrderType,

    /// Time in force
    pub time_in_force: TimeInForce,

    /// Order quantity
    pub quantity: Decimal,

    /// Limit price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    /// Stop price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Price>,

    /// Order status
    pub status: OrderStatus,

    /// Filled quantity
    pub filled_quantity: Decimal,

    /// Average fill price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub average_price: Option<Price>,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Last update timestamp
    pub updated_at: DateTime<Utc>,

    /// Reduce-only flag
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reduce_only: Option<bool>,

    /// Position side
    #[serde(skip_serializing_if = "Option::is_none")]
    pub position_side: Option<PositionSide>,
}

impl Order {
    /// Check if order is active (can still be filled)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Open | OrderStatus::PartiallyFilled | OrderStatus::Pending
        )
    }

    /// Check if order is closed (no longer active)
    pub fn is_closed(&self) -> bool {
        !self.is_active()
    }

    /// Get remaining quantity to be filled
    pub fn remaining_quantity(&self) -> Decimal {
        self.quantity - self.filled_quantity
    }

    /// Get fill percentage
    pub fn fill_percentage(&self) -> f64 {
        if self.quantity.is_zero() {
            0.0
        } else {
            ((self.filled_quantity / self.quantity) * Decimal::new(100, 0))
                .try_into()
                .unwrap_or(0.0)
        }
    }
}

/// Order modification request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderModification {
    /// New quantity (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Decimal>,

    /// New price (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<Price>,

    /// New stop price (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_price: Option<Price>,
}
