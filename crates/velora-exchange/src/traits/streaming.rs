//! Streaming trait for WebSocket data.

use crate::types::*;
use async_trait::async_trait;
use futures::Stream;

/// Streaming interface for real-time WebSocket data.
#[async_trait]
pub trait Streaming: Send + Sync {
    // === Market Data Streams ===

    /// Subscribe to trade stream
    async fn subscribe_trades(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<StreamTrade>> + Send + Unpin>>;

    /// Subscribe to order book updates
    async fn subscribe_orderbook(
        &self,
        symbol: &Symbol,
        depth: Option<usize>,
    ) -> Result<Box<dyn Stream<Item = Result<OrderBookUpdate>> + Send + Unpin>>;

    /// Subscribe to ticker updates
    async fn subscribe_ticker(
        &self,
        symbol: &Symbol,
    ) -> Result<Box<dyn Stream<Item = Result<Ticker>> + Send + Unpin>>;

    /// Subscribe to candlestick/kline updates
    async fn subscribe_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
    ) -> Result<Box<dyn Stream<Item = Result<Candle>> + Send + Unpin>>;

    // === User Data Streams ===

    /// Subscribe to order updates
    async fn subscribe_orders(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<OrderUpdate>> + Send + Unpin>>;

    /// Subscribe to position updates (perpetuals/futures)
    async fn subscribe_positions(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<PositionUpdate>> + Send + Unpin>>;

    /// Subscribe to balance updates
    async fn subscribe_balances(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<BalanceUpdate>> + Send + Unpin>>;

    /// Subscribe to all user data events
    async fn subscribe_user_data(
        &self,
    ) -> Result<Box<dyn Stream<Item = Result<UserDataEvent>> + Send + Unpin>>;
}
