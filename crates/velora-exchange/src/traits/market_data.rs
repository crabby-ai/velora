//! Market data trait.

use crate::types::*;
use async_trait::async_trait;
use chrono::{DateTime, Utc};

/// Market data interface (instrument agnostic).
///
/// This trait provides the same interface for public market data
/// regardless of instrument type (Spot, Perpetuals, Futures, Options).
///
/// The key insight: Candles are candles, order books are order books,
/// trades are trades - everywhere!
#[async_trait]
pub trait MarketData: Send + Sync {
    // === Market Information ===

    /// Get all available markets
    async fn get_markets(&self) -> Result<Vec<Market>>;

    /// Get information for a specific market
    async fn get_market(&self, symbol: &Symbol) -> Result<Market>;

    // === Ticker ===

    /// Get ticker for a symbol
    async fn get_ticker(&self, symbol: &Symbol) -> Result<Ticker>;

    /// Get tickers for all symbols
    async fn get_tickers(&self) -> Result<Vec<Ticker>>;

    // === Order Book ===

    /// Get order book snapshot
    ///
    /// # Arguments
    /// * `symbol` - Trading symbol
    /// * `depth` - Number of price levels (None = exchange default)
    async fn get_orderbook(&self, symbol: &Symbol, depth: Option<usize>) -> Result<OrderBook>;

    // === Recent Trades ===

    /// Get recent public trades
    ///
    /// # Arguments
    /// * `symbol` - Trading symbol
    /// * `limit` - Number of trades (None = exchange default)
    async fn get_recent_trades(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Trade>>;

    // === Candles/Klines (INSTRUMENT AGNOSTIC!) ===

    /// Get candlestick/kline data
    ///
    /// This method works the same for Spot, Perpetuals, Futures, and Options!
    ///
    /// # Arguments
    /// * `symbol` - Trading symbol
    /// * `interval` - Candle interval
    /// * `start_time` - Start time (None = earliest available)
    /// * `end_time` - End time (None = now)
    /// * `limit` - Maximum number of candles
    async fn get_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<Candle>>;

    // === Funding Rate (Perpetuals only) ===

    /// Get current funding rate (returns None for non-perpetual instruments)
    async fn get_funding_rate(&self, symbol: &Symbol) -> Result<Option<FundingRate>>;

    /// Get funding rate history
    async fn get_funding_rate_history(
        &self,
        symbol: &Symbol,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<FundingRate>>;
}
