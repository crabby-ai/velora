//! Strategy context for accessing market data and state.

use crate::errors::{StrategyError, StrategyResult};
use crate::types::Position;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use velora_core::types::{Candle, Trade};

/// Market data snapshot for a symbol
#[derive(Debug, Clone)]
pub struct MarketSnapshot {
    /// Latest price
    pub last_price: f64,
    /// Latest timestamp
    pub timestamp: DateTime<Utc>,
    /// Best bid price
    pub best_bid: Option<f64>,
    /// Best ask price
    pub best_ask: Option<f64>,
    /// 24h volume
    pub volume_24h: Option<f64>,
}

/// Strategy context providing access to market data and positions
#[derive(Clone)]
pub struct StrategyContext {
    /// Current positions held by the strategy
    positions: Arc<RwLock<HashMap<String, Position>>>,

    /// Market data snapshots by symbol
    market_data: Arc<RwLock<HashMap<String, MarketSnapshot>>>,

    /// Historical candles by symbol
    candles: Arc<RwLock<HashMap<String, Vec<Candle>>>>,

    /// Recent trades by symbol
    trades: Arc<RwLock<HashMap<String, Vec<Trade>>>>,

    /// Current capital available
    capital: Arc<RwLock<f64>>,

    /// Total capital (initial + P&L)
    total_capital: Arc<RwLock<f64>>,
}

impl Default for StrategyContext {
    fn default() -> Self {
        Self::new(10_000.0)
    }
}

impl StrategyContext {
    /// Create a new strategy context
    pub fn new(initial_capital: f64) -> Self {
        Self {
            positions: Arc::new(RwLock::new(HashMap::new())),
            market_data: Arc::new(RwLock::new(HashMap::new())),
            candles: Arc::new(RwLock::new(HashMap::new())),
            trades: Arc::new(RwLock::new(HashMap::new())),
            capital: Arc::new(RwLock::new(initial_capital)),
            total_capital: Arc::new(RwLock::new(initial_capital)),
        }
    }

    // === Position Management ===

    /// Get current position for a symbol
    pub fn get_position(&self, symbol: &str) -> StrategyResult<Option<Position>> {
        let positions = self
            .positions
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.get(symbol).cloned())
    }

    /// Get all current positions
    pub fn get_all_positions(&self) -> StrategyResult<Vec<Position>> {
        let positions = self
            .positions
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.values().cloned().collect())
    }

    /// Check if position exists for symbol
    pub fn has_position(&self, symbol: &str) -> StrategyResult<bool> {
        let positions = self
            .positions
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.contains_key(symbol))
    }

    /// Add or update a position
    pub fn update_position(&self, position: Position) -> StrategyResult<()> {
        let mut positions = self
            .positions
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        positions.insert(position.symbol.clone(), position);
        Ok(())
    }

    /// Remove a position
    pub fn remove_position(&self, symbol: &str) -> StrategyResult<Option<Position>> {
        let mut positions = self
            .positions
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.remove(symbol))
    }

    /// Get total number of open positions
    pub fn position_count(&self) -> StrategyResult<usize> {
        let positions = self
            .positions
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.len())
    }

    // === Capital Management ===

    /// Get available capital
    pub fn available_capital(&self) -> StrategyResult<f64> {
        let capital = self
            .capital
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(*capital)
    }

    /// Get total capital (initial + realized P&L)
    pub fn total_capital(&self) -> StrategyResult<f64> {
        let total = self
            .total_capital
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(*total)
    }

    /// Update available capital
    pub fn update_capital(&self, amount: f64) -> StrategyResult<()> {
        let mut capital = self
            .capital
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        *capital = amount;
        Ok(())
    }

    /// Get total unrealized P&L across all positions
    pub fn total_unrealized_pnl(&self) -> StrategyResult<f64> {
        let positions = self
            .positions
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(positions.values().map(|p| p.unrealized_pnl).sum())
    }

    /// Get total equity (capital + unrealized P&L)
    pub fn total_equity(&self) -> StrategyResult<f64> {
        let capital = self.available_capital()?;
        let unrealized = self.total_unrealized_pnl()?;
        Ok(capital + unrealized)
    }

    // === Market Data Access ===

    /// Get latest market snapshot for a symbol
    pub fn get_market_snapshot(&self, symbol: &str) -> StrategyResult<Option<MarketSnapshot>> {
        let data = self
            .market_data
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(data.get(symbol).cloned())
    }

    /// Update market snapshot
    pub fn update_market_snapshot(
        &self,
        symbol: impl Into<String>,
        snapshot: MarketSnapshot,
    ) -> StrategyResult<()> {
        let mut data = self
            .market_data
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        data.insert(symbol.into(), snapshot);
        Ok(())
    }

    /// Get latest price for a symbol
    pub fn get_last_price(&self, symbol: &str) -> StrategyResult<Option<f64>> {
        let data = self
            .market_data
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(data.get(symbol).map(|s| s.last_price))
    }

    /// Get historical candles for a symbol
    pub fn get_candles(&self, symbol: &str) -> StrategyResult<Vec<Candle>> {
        let candles = self
            .candles
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        Ok(candles.get(symbol).cloned().unwrap_or_default())
    }

    /// Add a candle to history
    pub fn add_candle(&self, symbol: impl Into<String>, candle: Candle) -> StrategyResult<()> {
        let mut candles = self
            .candles
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        candles
            .entry(symbol.into())
            .or_insert_with(Vec::new)
            .push(candle);
        Ok(())
    }

    /// Get recent trades for a symbol
    pub fn get_recent_trades(&self, symbol: &str, limit: usize) -> StrategyResult<Vec<Trade>> {
        let trades = self
            .trades
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;

        if let Some(symbol_trades) = trades.get(symbol) {
            let start = if symbol_trades.len() > limit {
                symbol_trades.len() - limit
            } else {
                0
            };
            Ok(symbol_trades[start..].to_vec())
        } else {
            Ok(vec![])
        }
    }

    /// Add a trade to history
    pub fn add_trade(&self, symbol: impl Into<String>, trade: Trade) -> StrategyResult<()> {
        let mut trades = self
            .trades
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;
        trades
            .entry(symbol.into())
            .or_insert_with(Vec::new)
            .push(trade);
        Ok(())
    }

    /// Update position prices based on latest market data
    pub fn update_position_prices(&self) -> StrategyResult<()> {
        let mut positions = self
            .positions
            .write()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;

        let market_data = self
            .market_data
            .read()
            .map_err(|e| StrategyError::Internal(format!("Lock error: {e}")))?;

        for position in positions.values_mut() {
            if let Some(snapshot) = market_data.get(&position.symbol) {
                position.update_price(snapshot.last_price);
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::PositionSide;

    #[test]
    fn test_context_capital() {
        let ctx = StrategyContext::new(10_000.0);
        assert_eq!(ctx.available_capital().unwrap(), 10_000.0);
        assert_eq!(ctx.total_capital().unwrap(), 10_000.0);
    }

    #[test]
    fn test_context_positions() {
        let ctx = StrategyContext::new(10_000.0);
        assert_eq!(ctx.position_count().unwrap(), 0);

        let pos = Position::new("BTC-USD-PERP", PositionSide::Long, 1.0, 50000.0);
        ctx.update_position(pos).unwrap();

        assert_eq!(ctx.position_count().unwrap(), 1);
        assert!(ctx.has_position("BTC-USD-PERP").unwrap());

        let retrieved = ctx.get_position("BTC-USD-PERP").unwrap();
        assert!(retrieved.is_some());
    }

    #[test]
    fn test_context_market_data() {
        let ctx = StrategyContext::new(10_000.0);

        let snapshot = MarketSnapshot {
            last_price: 50000.0,
            timestamp: Utc::now(),
            best_bid: Some(49999.0),
            best_ask: Some(50001.0),
            volume_24h: Some(1_000_000.0),
        };

        ctx.update_market_snapshot("BTC-USD-PERP", snapshot)
            .unwrap();

        let price = ctx.get_last_price("BTC-USD-PERP").unwrap();
        assert_eq!(price, Some(50000.0));
    }

    #[test]
    fn test_context_unrealized_pnl() {
        let ctx = StrategyContext::new(10_000.0);

        let mut pos1 = Position::new("BTC-USD-PERP", PositionSide::Long, 1.0, 50000.0);
        pos1.update_price(51000.0);

        let mut pos2 = Position::new("ETH-USD-PERP", PositionSide::Long, 10.0, 3000.0);
        pos2.update_price(3100.0);

        ctx.update_position(pos1).unwrap();
        ctx.update_position(pos2).unwrap();

        let total_pnl = ctx.total_unrealized_pnl().unwrap();
        assert_eq!(total_pnl, 2000.0); // 1000 + 1000

        let equity = ctx.total_equity().unwrap();
        assert_eq!(equity, 12_000.0); // 10000 + 2000
    }
}
