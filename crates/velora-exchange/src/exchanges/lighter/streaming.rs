//! Lighter streaming implementation

use async_trait::async_trait;
use futures::stream::Stream;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    auth::EvmWalletAuth,
    common::WebSocketClient,
    traits::Streaming,
    types::{
        BalanceUpdate, Candle, Interval, OrderBookUpdate, OrderUpdate, PositionUpdate, Result,
        StreamTrade, Symbol, Ticker, UserDataEvent,
    },
};

/// Lighter streaming component
pub struct LighterStreaming {
    ws_client: Arc<RwLock<WebSocketClient>>,
    auth: Option<EvmWalletAuth>,
}

impl LighterStreaming {
    pub fn new(ws_client: Arc<RwLock<WebSocketClient>>, auth: Option<EvmWalletAuth>) -> Self {
        Self { ws_client, auth }
    }
}

#[async_trait]
impl Streaming for LighterStreaming {
    async fn subscribe_trades(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<StreamTrade>> + Send + Unpin>> {
        // TODO: Implement trades WebSocket subscription
        todo!("Implement Lighter subscribe_trades for {}", symbol)
    }

    async fn subscribe_orderbook(
        &self,
        symbol: &Symbol,
        depth: Option<usize>,
    ) -> Result<Box<dyn Stream<Item = Result<OrderBookUpdate>> + Send + Unpin>> {
        // TODO: Implement orderbook WebSocket subscription
        todo!("Implement Lighter subscribe_orderbook for {}", symbol)
    }

    async fn subscribe_ticker(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<Ticker>> + Send + Unpin>> {
        // TODO: Implement ticker WebSocket subscription
        todo!("Implement Lighter subscribe_ticker for {}", symbol)
    }

    async fn subscribe_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
    ) -> Result<Box<dyn Stream<Item = Result<Candle>> + Send + Unpin>> {
        // TODO: Implement candles WebSocket subscription
        todo!(
            "Implement Lighter subscribe_candles for {} interval {:?}",
            symbol,
            interval
        )
    }

    async fn subscribe_orders(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<OrderUpdate>> + Send + Unpin>> {
        // TODO: Implement orders WebSocket subscription
        todo!("Implement Lighter subscribe_orders")
    }

    async fn subscribe_positions(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<PositionUpdate>> + Send + Unpin>> {
        // TODO: Implement positions WebSocket subscription
        todo!("Implement Lighter subscribe_positions")
    }

    async fn subscribe_balances(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<BalanceUpdate>> + Send + Unpin>> {
        // TODO: Implement balances WebSocket subscription
        todo!("Implement Lighter subscribe_balances")
    }

    async fn subscribe_user_data(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<UserDataEvent>> + Send + Unpin>> {
        // TODO: Implement user data WebSocket subscription
        todo!("Implement Lighter subscribe_user_data")
    }
}
