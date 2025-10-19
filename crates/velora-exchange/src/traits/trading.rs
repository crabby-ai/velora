//! Trading trait.

use crate::types::*;
use async_trait::async_trait;

/// Trading interface for order execution and management.
#[async_trait]
pub trait Trading: Send + Sync {
    // === Order Placement ===

    /// Place a new order
    async fn place_order(&self, order: NewOrder) -> Result<Order>;

    /// Place a market order (convenience method)
    async fn place_market_order(
        &self,
        symbol: &Symbol,
        side: Side,
        quantity: Decimal,
    ) -> Result<Order> {
        let order = NewOrder::market(symbol.clone(), side, quantity);
        self.place_order(order).await
    }

    /// Place a limit order (convenience method)
    async fn place_limit_order(
        &self,
        symbol: &Symbol,
        side: Side,
        price: Price,
        quantity: Decimal,
    ) -> Result<Order> {
        let order = NewOrder::limit(symbol.clone(), side, price, quantity);
        self.place_order(order).await
    }

    // === Order Management ===

    /// Cancel an order
    async fn cancel_order(&self, symbol: &Symbol, order_id: &str) -> Result<Order>;

    /// Cancel all orders for a symbol (None = all symbols)
    async fn cancel_all_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>>;

    /// Modify an existing order (if supported by exchange)
    async fn modify_order(&self, order_id: &str, modifications: OrderModification)
        -> Result<Order>;

    // === Order Queries ===

    /// Get order by ID
    async fn get_order(&self, symbol: &Symbol, order_id: &str) -> Result<Order>;

    /// Get all open orders (None = all symbols)
    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>>;

    /// Get order history
    async fn get_order_history(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Order>>;
}
