//! Portfolio tracking for backtesting.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use velora_strategy::{Position, PositionSide};

/// Portfolio state during backtest
#[derive(Debug, Clone)]
pub struct Portfolio {
    /// Available cash
    cash: f64,

    /// Initial capital at start of backtest
    initial_capital: f64,

    /// Current positions
    positions: HashMap<String, Position>,

    /// Completed trades
    trades: Vec<CompletedTrade>,

    /// Equity curve (snapshots over time)
    equity_curve: Vec<EquityPoint>,

    /// Current prices for each symbol
    current_prices: HashMap<String, f64>,
}

/// Point on the equity curve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquityPoint {
    /// Timestamp of this snapshot
    pub timestamp: DateTime<Utc>,

    /// Total equity (cash + positions value)
    pub equity: f64,

    /// Available cash
    pub cash: f64,

    /// Total value of positions
    pub positions_value: f64,
}

/// A completed trade (entry + exit)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTrade {
    /// Symbol traded
    pub symbol: String,

    /// Side of the trade
    pub side: PositionSide,

    /// Entry timestamp
    pub entry_time: DateTime<Utc>,

    /// Exit timestamp
    pub exit_time: DateTime<Utc>,

    /// Entry price
    pub entry_price: f64,

    /// Exit price
    pub exit_price: f64,

    /// Quantity traded
    pub quantity: f64,

    /// Realized P&L
    pub pnl: f64,

    /// P&L percentage
    pub pnl_pct: f64,

    /// Commission paid
    pub commission: f64,
}

impl Portfolio {
    /// Create a new portfolio
    pub fn new(initial_capital: f64) -> Self {
        Self {
            cash: initial_capital,
            initial_capital,
            positions: HashMap::new(),
            trades: Vec::new(),
            equity_curve: Vec::new(),
            current_prices: HashMap::new(),
        }
    }

    /// Get available cash
    pub fn cash(&self) -> f64 {
        self.cash
    }

    /// Get initial capital
    pub fn initial_capital(&self) -> f64 {
        self.initial_capital
    }

    /// Get current positions
    pub fn positions(&self) -> &HashMap<String, Position> {
        &self.positions
    }

    /// Get completed trades
    pub fn trades(&self) -> &[CompletedTrade] {
        &self.trades
    }

    /// Get equity curve
    pub fn equity_curve(&self) -> &[EquityPoint] {
        &self.equity_curve
    }

    /// Get position for a symbol
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Check if we have a position for a symbol
    pub fn has_position(&self, symbol: &str) -> bool {
        self.positions.contains_key(symbol)
    }

    /// Update current price for a symbol
    pub fn update_price(&mut self, symbol: String, price: f64) {
        self.current_prices.insert(symbol.clone(), price);

        // Update position price if we have one
        if let Some(position) = self.positions.get_mut(&symbol) {
            position.update_price(price);
        }
    }

    /// Calculate total equity (cash + positions value)
    pub fn total_equity(&self) -> f64 {
        let positions_value: f64 = self.positions.values().map(|p| p.value()).sum();
        self.cash + positions_value
    }

    /// Calculate total unrealized P&L
    pub fn unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    /// Calculate total realized P&L
    pub fn realized_pnl(&self) -> f64 {
        self.trades.iter().map(|t| t.pnl).sum()
    }

    /// Open a new position (buy)
    pub fn open_position(
        &mut self,
        symbol: String,
        side: PositionSide,
        quantity: f64,
        price: f64,
        commission: f64,
        timestamp: DateTime<Utc>,
    ) {
        // In backtesting, we only deduct commission from cash
        // The position value is tracked separately in total_equity()
        self.cash -= commission;

        // Create position
        let mut position = Position::new(&symbol, side, quantity, price);
        position.opened_at = timestamp;

        self.positions.insert(symbol.clone(), position);
        self.current_prices.insert(symbol, price);
    }

    /// Close an existing position (sell)
    pub fn close_position(
        &mut self,
        symbol: &str,
        price: f64,
        commission: f64,
        timestamp: DateTime<Utc>,
    ) {
        if let Some(position) = self.positions.remove(symbol) {
            // In backtesting, we realize the P&L to cash
            // Cash gets: position value at exit - commission
            let position_value = position.quantity * price;
            self.cash += position_value - commission;

            // Calculate realized P&L
            let pnl = match position.side {
                PositionSide::Long => {
                    (price - position.entry_price) * position.quantity - commission - commission
                }
                PositionSide::Short => {
                    (position.entry_price - price) * position.quantity - commission - commission
                }
            };

            let pnl_pct = (pnl / (position.entry_price * position.quantity)) * 100.0;

            // Record completed trade
            let trade = CompletedTrade {
                symbol: symbol.to_string(),
                side: position.side,
                entry_time: position.opened_at,
                exit_time: timestamp,
                entry_price: position.entry_price,
                exit_price: price,
                quantity: position.quantity,
                pnl,
                pnl_pct,
                commission: commission * 2.0, // entry + exit
            };

            self.trades.push(trade);
        }
    }

    /// Record an equity snapshot
    pub fn record_snapshot(&mut self, timestamp: DateTime<Utc>) {
        let positions_value: f64 = self.positions.values().map(|p| p.value()).sum();

        let snapshot = EquityPoint {
            timestamp,
            equity: self.cash + positions_value,
            cash: self.cash,
            positions_value,
        };

        self.equity_curve.push(snapshot);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_portfolio_creation() {
        let portfolio = Portfolio::new(10_000.0);
        assert_eq!(portfolio.cash(), 10_000.0);
        assert_eq!(portfolio.total_equity(), 10_000.0);
        assert_eq!(portfolio.positions().len(), 0);
    }

    #[test]
    fn test_open_close_position() {
        let mut portfolio = Portfolio::new(10_000.0);
        let timestamp = Utc::now();

        // Open position
        portfolio.open_position(
            "BTC-USD-PERP".to_string(),
            PositionSide::Long,
            1.0,
            50_000.0,
            50.0, // commission
            timestamp,
        );

        assert!(portfolio.has_position("BTC-USD-PERP"));
        assert_eq!(portfolio.cash(), 9_950.0); // 10000 - 50000 - 50

        // Update price
        portfolio.update_price("BTC-USD-PERP".to_string(), 51_000.0);

        let equity = portfolio.total_equity();
        assert!(equity > 10_000.0); // Should have profit

        // Close position
        portfolio.close_position("BTC-USD-PERP", 51_000.0, 51.0, timestamp);

        assert!(!portfolio.has_position("BTC-USD-PERP"));
        assert_eq!(portfolio.trades().len(), 1);

        let trade = &portfolio.trades()[0];
        assert_eq!(trade.entry_price, 50_000.0);
        assert_eq!(trade.exit_price, 51_000.0);
        assert!(trade.pnl > 0.0);
    }

    #[test]
    fn test_equity_snapshots() {
        let mut portfolio = Portfolio::new(10_000.0);
        let timestamp = Utc::now();

        portfolio.record_snapshot(timestamp);
        assert_eq!(portfolio.equity_curve().len(), 1);
        assert_eq!(portfolio.equity_curve()[0].equity, 10_000.0);
    }
}
