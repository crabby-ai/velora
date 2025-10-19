//! Paradex trading implementation

use async_trait::async_trait;
use rust_decimal::Decimal;
use std::sync::Arc;

use crate::{
    auth::StarknetWalletAuth,
    common::{RateLimiter, RestClient},
    traits::Trading,
    types::{NewOrder, Order, OrderModification, Price, Result, Side, Symbol},
};

/// Paradex trading component
pub struct ParadexTrading {
    rest_client: Arc<RestClient>,
    rate_limiter: Arc<RateLimiter>,
    auth: Option<StarknetWalletAuth>,
}

impl ParadexTrading {
    pub fn new(
        rest_client: Arc<RestClient>,
        rate_limiter: Arc<RateLimiter>,
        auth: Option<StarknetWalletAuth>,
    ) -> Self {
        Self {
            rest_client,
            rate_limiter,
            auth,
        }
    }
}

#[async_trait]
impl Trading for ParadexTrading {
    async fn place_order(&self, order: NewOrder) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement actual order placement with Starknet wallet signing
        // 1. Create order payload
        // 2. Sign with Starknet wallet
        // 3. Submit to Paradex API
        todo!("Implement Paradex place_order")
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
            "Implement Paradex cancel_order for {} order {}",
            symbol,
            order_id
        )
    }

    async fn cancel_all_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement cancel all orders
        todo!("Implement Paradex cancel_all_orders")
    }

    async fn modify_order(
        &self,
        order_id: &str,
        modifications: OrderModification,
    ) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement order modification
        todo!("Implement Paradex modify_order for {}", order_id)
    }

    async fn get_order(&self, symbol: &Symbol, order_id: &str) -> Result<Order> {
        self.rate_limiter.wait().await;

        // TODO: Implement get order
        todo!(
            "Implement Paradex get_order for {} order {}",
            symbol,
            order_id
        )
    }

    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement get open orders
        todo!("Implement Paradex get_open_orders")
    }

    async fn get_order_history(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Order>> {
        self.rate_limiter.wait().await;

        // TODO: Implement order history
        todo!("Implement Paradex get_order_history for {}", symbol)
    }
}
