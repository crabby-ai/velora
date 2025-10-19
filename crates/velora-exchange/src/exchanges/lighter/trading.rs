//! Lighter trading implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::{
    auth::EvmWalletAuth,
    common::{RateLimiter, RestClient},
    traits::Trading,
    types::{NewOrder, Order, OrderModification, Price, Result, Side, Symbol},
};

/// Lighter trading component
pub struct LighterTrading {
    rest_client: Arc<RestClient>,
    rate_limiter: Arc<RateLimiter>,
    auth: Option<EvmWalletAuth>,
}

impl LighterTrading {
    pub fn new(
        rest_client: Arc<RestClient>,
        rate_limiter: Arc<RateLimiter>,
        auth: Option<EvmWalletAuth>,
    ) -> Self {
        Self {
            rest_client,
            rate_limiter,
            auth,
        }
    }
}

#[async_trait]
impl Trading for LighterTrading {
    async fn place_order(&self, order: NewOrder) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement actual order placement with EVM wallet signing
        // 1. Create order payload
        // 2. Sign with EVM wallet
        // 3. Submit to Lighter API
        todo!("Implement Lighter place_order")
    }

    async fn place_market_order(
        &self,
        symbol: &Symbol,
        side: Side,
        quantity: Decimal,
    ) -> Result<Order> {
        let order = NewOrder::market(symbol.clone(), side, quantity);
        self.place_order(order).await
    }

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

    async fn cancel_order(&self, symbol: &Symbol, order_id: &str) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement order cancellation
        todo!(
            "Implement Lighter cancel_order for {} order {}",
            symbol,
            order_id
        )
    }

    async fn cancel_all_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement cancel all orders
        todo!("Implement Lighter cancel_all_orders")
    }

    async fn modify_order(
        &self,
        order_id: &str,
        modifications: OrderModification,
    ) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement order modification
        todo!("Implement Lighter modify_order for {}", order_id)
    }

    async fn get_order(&self, symbol: &Symbol, order_id: &str) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement get order
        todo!(
            "Implement Lighter get_order for {} order {}",
            symbol,
            order_id
        )
    }

    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement get open orders
        todo!("Implement Lighter get_open_orders")
    }

    async fn get_order_history(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement order history
        todo!("Implement Lighter get_order_history for {}", symbol)
    }
}
