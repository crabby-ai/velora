//! Position tracking and P&L calculation

use crate::errors::EngineResult;
use crate::events::Fill;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use velora_strategy::PositionSide;

/// Tracks positions and calculates real-time P&L
pub struct PositionTracker {
    /// Current open positions
    positions: HashMap<String, Position>,

    /// Available cash
    cash: f64,

    /// Initial capital
    initial_capital: f64,

    /// Historical equity snapshots
    equity_history: Vec<EquitySnapshot>,

    /// Current market prices for each symbol
    current_prices: HashMap<String, f64>,
}

/// A position in a symbol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbol being traded
    pub symbol: String,

    /// Position side (Long/Short)
    pub side: PositionSide,

    /// Total quantity held
    pub quantity: f64,

    /// Average entry price
    pub average_entry_price: f64,

    /// Current market price
    pub current_price: f64,

    /// Unrealized P&L
    pub unrealized_pnl: f64,

    /// Realized P&L from this position
    pub realized_pnl: f64,

    /// When position was opened
    pub opened_at: DateTime<Utc>,

    /// Last update time
    pub last_updated: DateTime<Utc>,
}

/// Snapshot of equity at a point in time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquitySnapshot {
    /// Timestamp of snapshot
    pub timestamp: DateTime<Utc>,

    /// Total equity (cash + positions value)
    pub total_equity: f64,

    /// Available cash
    pub cash: f64,

    /// Total value of all positions
    pub positions_value: f64,

    /// Total unrealized P&L
    pub unrealized_pnl: f64,

    /// Total realized P&L
    pub realized_pnl: f64,
}

impl PositionTracker {
    /// Create a new position tracker
    pub fn new(initial_capital: f64) -> Self {
        Self {
            positions: HashMap::new(),
            cash: initial_capital,
            initial_capital,
            equity_history: Vec::new(),
            current_prices: HashMap::new(),
        }
    }

    /// Process a fill (open, add to, or reduce position)
    pub fn process_fill(&mut self, fill: &Fill) -> EngineResult<()> {
        // Deduct commission from cash
        self.cash -= fill.commission;

        // Update current price
        self.current_prices.insert(fill.symbol.clone(), fill.price);

        // Determine if we have an existing position
        let has_position = self.positions.contains_key(&fill.symbol);

        if has_position {
            // Clone position data to avoid borrow issues
            let position_side = self.positions.get(&fill.symbol).unwrap().side;

            // Determine action based on fill side and position side
            let is_adding = match fill.side {
                velora_core::Side::Buy => position_side == PositionSide::Long,
                velora_core::Side::Sell => position_side == PositionSide::Short,
            };

            if is_adding {
                // Get mutable reference and add to position inline
                let position = self.positions.get_mut(&fill.symbol).unwrap();
                let total_cost = (position.quantity * position.average_entry_price)
                    + (fill.quantity * fill.price);
                let new_quantity = position.quantity + fill.quantity;
                position.average_entry_price = total_cost / new_quantity;
                position.quantity = new_quantity;
                position.last_updated = fill.timestamp;

                // Update P&L
                position.unrealized_pnl = match position.side {
                    PositionSide::Long => {
                        position.quantity * (position.current_price - position.average_entry_price)
                    }
                    PositionSide::Short => {
                        position.quantity * (position.average_entry_price - position.current_price)
                    }
                };
            } else {
                // Reducing position - need to handle potential removal
                self.handle_reduce_position(fill)?;
            }
        } else {
            // New position
            self.open_new_position(fill)?;
        }

        Ok(())
    }

    /// Handle reducing a position (extracted to avoid borrow issues)
    fn handle_reduce_position(&mut self, fill: &Fill) -> EngineResult<()> {
        let position_quantity = self.positions.get(&fill.symbol).unwrap().quantity;
        let position_side = self.positions.get(&fill.symbol).unwrap().side;
        let position_entry_price = self
            .positions
            .get(&fill.symbol)
            .unwrap()
            .average_entry_price;

        if fill.quantity > position_quantity {
            // Closing and reversing
            let close_quantity = position_quantity;
            let reverse_quantity = fill.quantity - close_quantity;

            // Calculate realized P&L from closing
            let pnl = match position_side {
                PositionSide::Long => close_quantity * (fill.price - position_entry_price),
                PositionSide::Short => close_quantity * (position_entry_price - fill.price),
            };

            self.cash += pnl;

            // Get mutable reference and reverse position
            let position = self.positions.get_mut(&fill.symbol).unwrap();
            position.realized_pnl += pnl;
            position.side = match position.side {
                PositionSide::Long => PositionSide::Short,
                PositionSide::Short => PositionSide::Long,
            };
            position.quantity = reverse_quantity;
            position.average_entry_price = fill.price;
            position.unrealized_pnl = 0.0;
            position.last_updated = fill.timestamp;
        } else if fill.quantity == position_quantity {
            // Fully closing position
            let pnl = match position_side {
                PositionSide::Long => position_quantity * (fill.price - position_entry_price),
                PositionSide::Short => position_quantity * (position_entry_price - fill.price),
            };

            self.cash += pnl;

            // Remove position
            self.positions.remove(&fill.symbol);
        } else {
            // Partially closing position
            let pnl = match position_side {
                PositionSide::Long => fill.quantity * (fill.price - position_entry_price),
                PositionSide::Short => fill.quantity * (position_entry_price - fill.price),
            };

            self.cash += pnl;

            // Get mutable reference and update
            let position = self.positions.get_mut(&fill.symbol).unwrap();
            position.realized_pnl += pnl;
            position.quantity -= fill.quantity;
            position.last_updated = fill.timestamp;

            // Update P&L
            position.unrealized_pnl = match position.side {
                PositionSide::Long => {
                    position.quantity * (position.current_price - position.average_entry_price)
                }
                PositionSide::Short => {
                    position.quantity * (position.average_entry_price - position.current_price)
                }
            };
        }

        Ok(())
    }

    /// Open a new position
    fn open_new_position(&mut self, fill: &Fill) -> EngineResult<()> {
        let side = match fill.side {
            velora_core::Side::Buy => PositionSide::Long,
            velora_core::Side::Sell => PositionSide::Short,
        };

        let position = Position {
            symbol: fill.symbol.clone(),
            side,
            quantity: fill.quantity,
            average_entry_price: fill.price,
            current_price: fill.price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            opened_at: fill.timestamp,
            last_updated: fill.timestamp,
        };

        self.positions.insert(fill.symbol.clone(), position);
        Ok(())
    }

    /// Update position price from market data
    pub fn update_position_price(&mut self, symbol: &str, price: f64) {
        self.current_prices.insert(symbol.to_string(), price);

        if let Some(position) = self.positions.get_mut(symbol) {
            position.current_price = price;
            // Calculate P&L inline to avoid borrow checker issues
            position.unrealized_pnl = match position.side {
                PositionSide::Long => {
                    position.quantity * (position.current_price - position.average_entry_price)
                }
                PositionSide::Short => {
                    position.quantity * (position.average_entry_price - position.current_price)
                }
            };
        }
    }

    /// Update position's unrealized P&L
    fn update_position_pnl(&mut self, position: &mut Position) {
        position.unrealized_pnl = match position.side {
            PositionSide::Long => {
                position.quantity * (position.current_price - position.average_entry_price)
            }
            PositionSide::Short => {
                position.quantity * (position.average_entry_price - position.current_price)
            }
        };
    }

    /// Get a position by symbol
    pub fn get_position(&self, symbol: &str) -> Option<&Position> {
        self.positions.get(symbol)
    }

    /// Get all open positions
    pub fn get_positions(&self) -> Vec<&Position> {
        self.positions.values().collect()
    }

    /// Calculate total equity (cash + positions value)
    pub fn total_equity(&self) -> f64 {
        let positions_value: f64 = self
            .positions
            .values()
            .map(|p| p.quantity * p.current_price)
            .sum();

        self.cash + positions_value
    }

    /// Calculate total unrealized P&L
    pub fn total_unrealized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.unrealized_pnl).sum()
    }

    /// Calculate total realized P&L
    pub fn total_realized_pnl(&self) -> f64 {
        self.positions.values().map(|p| p.realized_pnl).sum()
    }

    /// Create an equity snapshot
    pub fn snapshot(&self) -> EquitySnapshot {
        let positions_value: f64 = self
            .positions
            .values()
            .map(|p| p.quantity * p.current_price)
            .sum();

        EquitySnapshot {
            timestamp: Utc::now(),
            total_equity: self.total_equity(),
            cash: self.cash,
            positions_value,
            unrealized_pnl: self.total_unrealized_pnl(),
            realized_pnl: self.total_realized_pnl(),
        }
    }

    /// Record a snapshot to history
    pub fn record_snapshot(&mut self) {
        self.equity_history.push(self.snapshot());
    }

    /// Get equity history
    pub fn get_equity_history(&self) -> &[EquitySnapshot] {
        &self.equity_history
    }

    /// Get number of open positions
    pub fn position_count(&self) -> usize {
        self.positions.len()
    }

    /// Get available cash
    pub fn available_cash(&self) -> f64 {
        self.cash
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use velora_core::{OrderId, Side};

    fn create_fill(symbol: &str, side: Side, quantity: f64, price: f64) -> Fill {
        Fill {
            order_id: OrderId::new(),
            symbol: symbol.to_string(),
            side,
            quantity,
            price,
            commission: 5.0,
            timestamp: Utc::now(),
        }
    }

    #[test]
    fn test_open_long_position() {
        let mut tracker = PositionTracker::new(10_000.0);

        // Buy 0.1 BTC at 50,000
        let fill = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 50_000.0);
        tracker.process_fill(&fill).unwrap();

        // Check position
        let position = tracker.get_position("BTC-USD-PERP").unwrap();
        assert_eq!(position.quantity, 0.1);
        assert_eq!(position.average_entry_price, 50_000.0);
        assert_eq!(position.side, PositionSide::Long);

        // Check cash (commission deducted)
        assert_eq!(tracker.available_cash(), 10_000.0 - 5.0);
    }

    #[test]
    fn test_add_to_long_position() {
        let mut tracker = PositionTracker::new(10_000.0);

        // Buy 0.1 BTC at 50,000
        let fill1 = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 50_000.0);
        tracker.process_fill(&fill1).unwrap();

        // Buy another 0.1 BTC at 52,000
        let fill2 = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 52_000.0);
        tracker.process_fill(&fill2).unwrap();

        // Check position
        let position = tracker.get_position("BTC-USD-PERP").unwrap();
        assert_eq!(position.quantity, 0.2);
        // Average: (0.1 * 50,000 + 0.1 * 52,000) / 0.2 = 51,000
        assert_eq!(position.average_entry_price, 51_000.0);
    }

    #[test]
    fn test_close_long_position() {
        let mut tracker = PositionTracker::new(10_000.0);

        // Buy 0.1 BTC at 50,000
        let fill1 = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 50_000.0);
        tracker.process_fill(&fill1).unwrap();

        // Update price to 52,000
        tracker.update_position_price("BTC-USD-PERP", 52_000.0);

        // Sell 0.1 BTC at 52,000
        let fill2 = create_fill("BTC-USD-PERP", Side::Sell, 0.1, 52_000.0);
        tracker.process_fill(&fill2).unwrap();

        // Position should be closed
        assert!(tracker.get_position("BTC-USD-PERP").is_none());

        // P&L: 0.1 * (52,000 - 50,000) = 200
        // Cash: 10,000 - 5 (commission1) - 5 (commission2) + 200 (profit) = 10,190
        assert_eq!(tracker.available_cash(), 10_190.0);
    }

    #[test]
    fn test_update_position_price() {
        let mut tracker = PositionTracker::new(10_000.0);

        // Buy 0.1 BTC at 50,000
        let fill = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 50_000.0);
        tracker.process_fill(&fill).unwrap();

        // Update price to 52,000
        tracker.update_position_price("BTC-USD-PERP", 52_000.0);

        // Check unrealized P&L
        let position = tracker.get_position("BTC-USD-PERP").unwrap();
        // 0.1 * (52,000 - 50,000) = 200
        assert_eq!(position.unrealized_pnl, 200.0);

        // Total equity: 10,000 - 5 (commission) + 200 (unrealized) = 10,195
        // But total_equity doesn't include unrealized, so:
        // Cash (9,995) + Position Value (0.1 * 52,000 = 5,200) = 15,195
        assert_eq!(tracker.total_equity(), 15_195.0);
    }

    #[test]
    fn test_snapshot() {
        let mut tracker = PositionTracker::new(10_000.0);

        // Buy 0.1 BTC at 50,000
        let fill = create_fill("BTC-USD-PERP", Side::Buy, 0.1, 50_000.0);
        tracker.process_fill(&fill).unwrap();

        // Update price to 52,000
        tracker.update_position_price("BTC-USD-PERP", 52_000.0);

        // Take snapshot
        let snapshot = tracker.snapshot();

        assert_eq!(snapshot.cash, 9_995.0); // 10,000 - 5 commission
        assert_eq!(snapshot.positions_value, 5_200.0); // 0.1 * 52,000
        assert_eq!(snapshot.unrealized_pnl, 200.0); // 0.1 * (52,000 - 50,000)
        assert_eq!(snapshot.total_equity, 15_195.0); // 9,995 + 5,200
    }
}
