//! Order lifecycle management

use crate::errors::{EngineError, EngineResult};
use crate::events::{OrderId, OrderStatus, OrderUpdate};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Instant;
use velora_core::{OrderType, Side};

/// Manages order lifecycle: submission, tracking, cancellation
pub struct OrderManager {
    /// Orders waiting to be submitted
    pending_orders: HashMap<OrderId, Order>,

    /// Orders submitted to exchange
    active_orders: HashMap<OrderId, Order>,

    /// Completed orders (filled, cancelled, rejected)
    completed_orders: Vec<Order>,

    /// Order event history
    order_history: Vec<OrderEvent>,

    /// Rate limiter
    rate_limiter: RateLimiter,
}

/// Order representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Internal order ID
    pub id: OrderId,

    /// Client order ID (for exchange)
    pub client_order_id: String,

    /// Symbol to trade
    pub symbol: String,

    /// Buy or Sell
    pub side: Side,

    /// Order type
    pub order_type: OrderType,

    /// Quantity to trade
    pub quantity: f64,

    /// Limit price (for limit orders)
    pub price: Option<f64>,

    /// Current status
    pub status: OrderStatus,

    /// When order was created
    pub created_at: DateTime<Utc>,

    /// Last update time
    pub updated_at: DateTime<Utc>,

    /// Quantity filled so far
    pub filled_quantity: f64,

    /// Average fill price
    pub average_fill_price: f64,

    /// Optional error message
    pub error_message: Option<String>,
}

/// Order event for audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderEvent {
    pub order_id: OrderId,
    pub event_type: OrderEventType,
    pub timestamp: DateTime<Utc>,
    pub details: String,
}

/// Types of order events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrderEventType {
    Created,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
    Failed,
}

/// Rate limiter to prevent exceeding exchange limits
pub struct RateLimiter {
    max_orders_per_second: u32,
    window_start: Instant,
    order_count: u32,
}

impl OrderManager {
    /// Create a new order manager
    pub fn new(max_orders_per_second: u32) -> Self {
        Self {
            pending_orders: HashMap::new(),
            active_orders: HashMap::new(),
            completed_orders: Vec::new(),
            order_history: Vec::new(),
            rate_limiter: RateLimiter::new(max_orders_per_second),
        }
    }

    /// Submit a new order
    pub fn submit_order(&mut self, mut order: Order) -> EngineResult<OrderId> {
        // Check rate limit
        self.rate_limiter.check_and_increment()?;

        // Validate order
        self.validate_order(&order)?;

        // Set status to pending
        order.status = OrderStatus::Pending;
        order.created_at = Utc::now();
        order.updated_at = Utc::now();

        let order_id = order.id;

        // Record event
        self.record_event(OrderEvent {
            order_id,
            event_type: OrderEventType::Created,
            timestamp: Utc::now(),
            details: format!(
                "{:?} {} {} @ {:?}",
                order.side, order.quantity, order.symbol, order.price
            ),
        });

        // Add to pending orders
        self.pending_orders.insert(order_id, order);

        Ok(order_id)
    }

    /// Mark order as submitted to exchange
    pub fn mark_submitted(&mut self, order_id: OrderId) -> EngineResult<()> {
        let mut order = self
            .pending_orders
            .remove(&order_id)
            .ok_or_else(|| EngineError::OrderNotFound(order_id.to_string()))?;

        order.status = OrderStatus::Submitted;
        order.updated_at = Utc::now();

        self.record_event(OrderEvent {
            order_id,
            event_type: OrderEventType::Submitted,
            timestamp: Utc::now(),
            details: "Order submitted to exchange".to_string(),
        });

        self.active_orders.insert(order_id, order);
        Ok(())
    }

    /// Update order with new status/fill information
    pub fn update_order(&mut self, order_id: OrderId, update: OrderUpdate) -> EngineResult<()> {
        // Try to find order in active orders first
        if let Some(order) = self.active_orders.get_mut(&order_id) {
            order.status = update.status;
            order.filled_quantity = update.filled_quantity;
            order.average_fill_price = update.average_price;
            order.updated_at = update.timestamp;
            order.error_message = update.error_message.clone();

            // Capture order quantity before recording event
            let order_quantity = order.quantity;

            // Record event
            let event_type = match update.status {
                OrderStatus::PartiallyFilled => OrderEventType::PartiallyFilled,
                OrderStatus::Filled => OrderEventType::Filled,
                OrderStatus::Cancelled => OrderEventType::Cancelled,
                OrderStatus::Rejected => OrderEventType::Rejected,
                OrderStatus::Failed => OrderEventType::Failed,
                _ => return Ok(()),
            };

            // End the mutable borrow before calling record_event

            self.record_event(OrderEvent {
                order_id,
                event_type: event_type.clone(),
                timestamp: update.timestamp,
                details: format!(
                    "Filled: {}/{} @ {:.2}",
                    update.filled_quantity, order_quantity, update.average_price
                ),
            });

            // Move to completed if terminal state
            if matches!(
                update.status,
                OrderStatus::Filled
                    | OrderStatus::Cancelled
                    | OrderStatus::Rejected
                    | OrderStatus::Failed
            ) {
                if let Some(completed_order) = self.active_orders.remove(&order_id) {
                    self.completed_orders.push(completed_order);
                }
            }

            Ok(())
        } else if let Some(order) = self.pending_orders.get_mut(&order_id) {
            // Update pending order
            order.status = update.status;
            order.updated_at = update.timestamp;
            order.error_message = update.error_message;
            Ok(())
        } else {
            Err(EngineError::OrderNotFound(order_id.to_string()))
        }
    }

    /// Cancel an order
    pub fn cancel_order(&mut self, order_id: OrderId) -> EngineResult<()> {
        if !self.active_orders.contains_key(&order_id) {
            return Err(EngineError::OrderError(format!(
                "Cannot cancel order {order_id}: not active"
            )));
        }

        self.record_event(OrderEvent {
            order_id,
            event_type: OrderEventType::Cancelled,
            timestamp: Utc::now(),
            details: "Cancel request initiated".to_string(),
        });

        Ok(())
    }

    /// Get an order by ID
    pub fn get_order(&self, order_id: OrderId) -> Option<&Order> {
        self.pending_orders
            .get(&order_id)
            .or_else(|| self.active_orders.get(&order_id))
            .or_else(|| self.completed_orders.iter().find(|o| o.id == order_id))
    }

    /// Get all active orders
    pub fn get_active_orders(&self) -> Vec<&Order> {
        self.active_orders.values().collect()
    }

    /// Get all pending orders
    pub fn get_pending_orders(&self) -> Vec<&Order> {
        self.pending_orders.values().collect()
    }

    /// Get orders for a specific symbol
    pub fn get_orders_for_symbol(&self, symbol: &str) -> Vec<&Order> {
        self.active_orders
            .values()
            .filter(|o| o.symbol == symbol)
            .collect()
    }

    /// Get total number of orders
    pub fn total_orders(&self) -> usize {
        self.pending_orders.len() + self.active_orders.len() + self.completed_orders.len()
    }

    /// Get completed orders
    pub fn get_completed_orders(&self) -> &[Order] {
        &self.completed_orders
    }

    /// Get order history
    pub fn get_order_history(&self) -> &[OrderEvent] {
        &self.order_history
    }

    /// Validate order before submission
    fn validate_order(&self, order: &Order) -> EngineResult<()> {
        if order.quantity <= 0.0 {
            return Err(EngineError::OrderError(
                "Quantity must be positive".to_string(),
            ));
        }

        if order.symbol.is_empty() {
            return Err(EngineError::OrderError(
                "Symbol cannot be empty".to_string(),
            ));
        }

        if matches!(order.order_type, OrderType::Limit) && order.price.is_none() {
            return Err(EngineError::OrderError(
                "Limit orders must have a price".to_string(),
            ));
        }

        Ok(())
    }

    /// Record an order event
    fn record_event(&mut self, event: OrderEvent) {
        self.order_history.push(event);
    }
}

impl RateLimiter {
    pub fn new(max_orders_per_second: u32) -> Self {
        Self {
            max_orders_per_second,
            window_start: Instant::now(),
            order_count: 0,
        }
    }

    pub fn check_and_increment(&mut self) -> EngineResult<()> {
        let elapsed = self.window_start.elapsed();

        // Reset window every second
        if elapsed >= std::time::Duration::from_secs(1) {
            self.window_start = Instant::now();
            self.order_count = 0;
        }

        // Check limit
        if self.order_count >= self.max_orders_per_second {
            return Err(EngineError::RateLimitExceeded {
                max: self.max_orders_per_second,
            });
        }

        self.order_count += 1;
        Ok(())
    }
}

impl Order {
    /// Create a new order
    pub fn new(
        symbol: String,
        side: Side,
        order_type: OrderType,
        quantity: f64,
        price: Option<f64>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: uuid::Uuid::new_v4(),
            client_order_id: format!("CLT-{}", uuid::Uuid::new_v4()),
            symbol,
            side,
            order_type,
            quantity,
            price,
            status: OrderStatus::Pending,
            created_at: now,
            updated_at: now,
            filled_quantity: 0.0,
            average_fill_price: 0.0,
            error_message: None,
        }
    }

    /// Check if order is terminal (won't change anymore)
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Failed
        )
    }

    /// Check if order is active (can still receive updates)
    pub fn is_active(&self) -> bool {
        matches!(
            self.status,
            OrderStatus::Submitted | OrderStatus::PartiallyFilled
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_submit_order() {
        let mut manager = OrderManager::new(10);
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );

        let order_id = manager.submit_order(order).unwrap();

        assert_eq!(manager.get_pending_orders().len(), 1);
        assert_eq!(manager.get_active_orders().len(), 0);

        let retrieved = manager.get_order(order_id).unwrap();
        assert_eq!(retrieved.status, OrderStatus::Pending);
    }

    #[test]
    fn test_mark_submitted() {
        let mut manager = OrderManager::new(10);
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );

        let order_id = manager.submit_order(order).unwrap();
        manager.mark_submitted(order_id).unwrap();

        assert_eq!(manager.get_pending_orders().len(), 0);
        assert_eq!(manager.get_active_orders().len(), 1);

        let retrieved = manager.get_order(order_id).unwrap();
        assert_eq!(retrieved.status, OrderStatus::Submitted);
    }

    #[test]
    fn test_update_order_filled() {
        let mut manager = OrderManager::new(10);
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );

        let order_id = manager.submit_order(order).unwrap();
        manager.mark_submitted(order_id).unwrap();

        let update = OrderUpdate {
            order_id,
            status: OrderStatus::Filled,
            filled_quantity: 0.1,
            average_price: 50_000.0,
            timestamp: Utc::now(),
            error_message: None,
        };

        manager.update_order(order_id, update).unwrap();

        assert_eq!(manager.get_active_orders().len(), 0);
        assert_eq!(manager.get_completed_orders().len(), 1);

        let completed = &manager.get_completed_orders()[0];
        assert_eq!(completed.status, OrderStatus::Filled);
        assert_eq!(completed.filled_quantity, 0.1);
        assert_eq!(completed.average_fill_price, 50_000.0);
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2);

        // First two should succeed
        assert!(limiter.check_and_increment().is_ok());
        assert!(limiter.check_and_increment().is_ok());

        // Third should fail
        assert!(limiter.check_and_increment().is_err());
    }

    #[test]
    fn test_validate_order() {
        let manager = OrderManager::new(10);

        // Invalid: zero quantity
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.0,
            None,
        );
        assert!(manager.validate_order(&order).is_err());

        // Invalid: limit order without price
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Limit,
            0.1,
            None,
        );
        assert!(manager.validate_order(&order).is_err());

        // Valid: market order
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );
        assert!(manager.validate_order(&order).is_ok());

        // Valid: limit order with price
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Limit,
            0.1,
            Some(50_000.0),
        );
        assert!(manager.validate_order(&order).is_ok());
    }
}
