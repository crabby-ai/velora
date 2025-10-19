//! Event types for the trading engine

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use velora_core::{Candle, Side};

/// Order ID type alias
pub type OrderId = Uuid;

/// Market events from data streams
#[derive(Debug, Clone)]
pub enum MarketEvent {
    /// New candle received
    Candle(Candle),

    /// Order status update from exchange
    OrderUpdate(OrderUpdate),

    /// Market data error
    Error(String),

    /// Connection lost to market data
    Disconnected,

    /// Reconnected to market data
    Reconnected,
}

/// Order status update from exchange
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    /// Order ID
    pub order_id: OrderId,

    /// Current order status
    pub status: OrderStatus,

    /// Quantity filled so far
    pub filled_quantity: f64,

    /// Average fill price
    pub average_price: f64,

    /// Timestamp of update
    pub timestamp: DateTime<Utc>,

    /// Optional error message (for rejections/failures)
    pub error_message: Option<String>,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order created but not yet submitted
    Pending,

    /// Order submitted to exchange
    Submitted,

    /// Order partially filled
    PartiallyFilled,

    /// Order completely filled
    Filled,

    /// Order cancelled
    Cancelled,

    /// Order rejected by exchange
    Rejected,

    /// Order failed to submit
    Failed,
}

/// Fill event when an order is (partially) executed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    /// Associated order ID
    pub order_id: OrderId,

    /// Symbol
    pub symbol: String,

    /// Side (Buy/Sell)
    pub side: Side,

    /// Quantity filled
    pub quantity: f64,

    /// Fill price
    pub price: f64,

    /// Commission paid
    pub commission: f64,

    /// Timestamp of fill
    pub timestamp: DateTime<Utc>,
}

impl Fill {
    /// Calculate the total cost (or proceeds) including commission
    pub fn total_cost(&self) -> f64 {
        let base = self.quantity * self.price;
        match self.side {
            Side::Buy => base + self.commission,
            Side::Sell => base - self.commission,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fill_total_cost_buy() {
        let fill = Fill {
            order_id: OrderId::new(),
            symbol: "BTC-USD-PERP".to_string(),
            side: Side::Buy,
            quantity: 0.1,
            price: 50_000.0,
            commission: 5.0,
            timestamp: Utc::now(),
        };

        // 0.1 * 50,000 + 5 = 5,005
        assert_eq!(fill.total_cost(), 5_005.0);
    }

    #[test]
    fn test_fill_total_cost_sell() {
        let fill = Fill {
            order_id: OrderId::new(),
            symbol: "BTC-USD-PERP".to_string(),
            side: Side::Sell,
            quantity: 0.1,
            price: 50_000.0,
            commission: 5.0,
            timestamp: Utc::now(),
        };

        // 0.1 * 50,000 - 5 = 4,995
        assert_eq!(fill.total_cost(), 4_995.0);
    }

    #[test]
    fn test_order_status_equality() {
        assert_eq!(OrderStatus::Pending, OrderStatus::Pending);
        assert_ne!(OrderStatus::Pending, OrderStatus::Filled);
    }
}
