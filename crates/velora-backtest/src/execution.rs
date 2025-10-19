//! Order execution simulation.

use crate::config::{ExecutionConfig, FillModel};
use crate::errors::{BacktestError, BacktestResult};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use velora_core::types::{Candle, Side};
use velora_strategy::Signal;

/// Unique order identifier
pub type OrderId = u64;

/// Simulated order
#[derive(Debug, Clone)]
pub struct Order {
    /// Unique order ID
    pub id: OrderId,

    /// Symbol to trade
    pub symbol: String,

    /// Order side
    pub side: Side,

    /// Quantity
    pub quantity: f64,

    /// Limit price (if applicable)
    pub limit_price: Option<f64>,

    /// When the order was created
    pub created_at: DateTime<Utc>,

    /// Order status
    pub status: OrderStatus,
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OrderStatus {
    /// Order is pending execution
    Pending,

    /// Order has been filled
    Filled,

    /// Order was cancelled
    Cancelled,
}

/// Fill event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fill {
    /// Order ID that was filled
    pub order_id: OrderId,

    /// Symbol
    pub symbol: String,

    /// Side
    pub side: Side,

    /// Quantity filled
    pub quantity: f64,

    /// Fill price
    pub price: f64,

    /// Commission paid
    pub commission: f64,

    /// When the fill occurred
    pub timestamp: DateTime<Utc>,
}

/// Execution simulator
pub struct ExecutionSimulator {
    config: ExecutionConfig,
    next_order_id: OrderId,
    pending_orders: HashMap<OrderId, Order>,
    fills: Vec<Fill>,
}

impl ExecutionSimulator {
    /// Create a new execution simulator
    pub fn new(config: ExecutionConfig) -> Self {
        Self {
            config,
            next_order_id: 1,
            pending_orders: HashMap::new(),
            fills: Vec::new(),
        }
    }

    /// Submit a new order from a strategy signal
    pub fn submit_order(
        &mut self,
        signal: Signal,
        timestamp: DateTime<Utc>,
    ) -> BacktestResult<OrderId> {
        let (symbol, quantity, side, limit_price) = match signal {
            Signal::Buy {
                symbol,
                quantity,
                limit_price,
                ..
            } => (symbol, quantity, Side::Buy, limit_price),
            Signal::Sell {
                symbol,
                quantity,
                limit_price,
                ..
            } => (symbol, quantity, Side::Sell, limit_price),
            Signal::Close { .. } => {
                // Close signals are handled separately
                return Err(BacktestError::InvalidOrder(
                    "Close signals should be converted to sell orders".to_string(),
                ));
            }
            Signal::Hold | Signal::Modify { .. } => {
                return Err(BacktestError::InvalidOrder(
                    "Cannot submit Hold or Modify signals".to_string(),
                ));
            }
        };

        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let order = Order {
            id: order_id,
            symbol,
            side,
            quantity,
            limit_price,
            created_at: timestamp,
            status: OrderStatus::Pending,
        };

        self.pending_orders.insert(order_id, order);

        Ok(order_id)
    }

    /// Submit a close order for an existing position
    pub fn submit_close_order(
        &mut self,
        symbol: String,
        quantity: f64,
        side: Side, // Opposite of position side
        timestamp: DateTime<Utc>,
    ) -> BacktestResult<OrderId> {
        let order_id = self.next_order_id;
        self.next_order_id += 1;

        let order = Order {
            id: order_id,
            symbol,
            side,
            quantity,
            limit_price: None, // Market order
            created_at: timestamp,
            status: OrderStatus::Pending,
        };

        self.pending_orders.insert(order_id, order);

        Ok(order_id)
    }

    /// Process a candle and generate fills for pending orders
    pub fn process_candle(&mut self, candle: &Candle) -> Vec<Fill> {
        let mut fills = Vec::new();
        let mut filled_orders = Vec::new();

        for (order_id, order) in &self.pending_orders {
            // Only process orders for this symbol
            if order.symbol != candle.symbol.as_str() {
                continue;
            }

            // Check if order can be filled
            if let Some(fill_price) = self.can_fill_order(order, candle) {
                let commission = self.calculate_commission(order.quantity, fill_price);

                let fill = Fill {
                    order_id: *order_id,
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    price: fill_price,
                    commission,
                    timestamp: candle.timestamp,
                };

                fills.push(fill.clone());
                self.fills.push(fill);
                filled_orders.push(*order_id);
            }
        }

        // Remove filled orders
        for order_id in filled_orders {
            if let Some(mut order) = self.pending_orders.remove(&order_id) {
                order.status = OrderStatus::Filled;
            }
        }

        fills
    }

    /// Check if an order can be filled against a candle
    fn can_fill_order(&self, order: &Order, candle: &Candle) -> Option<f64> {
        match order.limit_price {
            Some(limit_price) => {
                // Limit order - check if price was reached
                match order.side {
                    Side::Buy => {
                        // Buy limit: fill if low <= limit_price
                        if candle.low.into_inner() <= limit_price {
                            Some(self.calculate_fill_price(order, candle, Some(limit_price)))
                        } else {
                            None
                        }
                    }
                    Side::Sell => {
                        // Sell limit: fill if high >= limit_price
                        if candle.high.into_inner() >= limit_price {
                            Some(self.calculate_fill_price(order, candle, Some(limit_price)))
                        } else {
                            None
                        }
                    }
                }
            }
            None => {
                // Market order - always fills
                Some(self.calculate_fill_price(order, candle, None))
            }
        }
    }

    /// Calculate the actual fill price based on fill model
    fn calculate_fill_price(
        &self,
        order: &Order,
        candle: &Candle,
        limit_price: Option<f64>,
    ) -> f64 {
        let base_price = limit_price.unwrap_or_else(|| candle.close.into_inner());

        match self.config.fill_model {
            FillModel::Market => {
                // Use close price
                base_price
            }
            FillModel::Realistic => {
                // Apply slippage
                let slippage = base_price * (self.config.slippage_bps / 10_000.0);
                match order.side {
                    Side::Buy => base_price + slippage,
                    Side::Sell => base_price - slippage,
                }
            }
            FillModel::Pessimistic => {
                // Use worst price
                match order.side {
                    Side::Buy => candle.high.into_inner(),
                    Side::Sell => candle.low.into_inner(),
                }
            }
        }
    }

    /// Calculate commission for an order
    fn calculate_commission(&self, quantity: f64, price: f64) -> f64 {
        let notional = quantity * price;
        notional * self.config.commission_rate
    }

    /// Get all fills
    pub fn fills(&self) -> &[Fill] {
        &self.fills
    }

    /// Get pending order count
    pub fn pending_order_count(&self) -> usize {
        self.pending_orders.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use velora_core::types::Symbol;

    fn create_test_candle(close: f64, low: f64, high: f64) -> Candle {
        Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp: Utc::now(),
            open: close.into(),
            high: high.into(),
            low: low.into(),
            close: close.into(),
            volume: 100.0.into(),
        }
    }

    #[test]
    fn test_market_order_execution() {
        let config = ExecutionConfig::default();
        let mut simulator = ExecutionSimulator::new(config);

        let signal = Signal::buy("BTC-USD-PERP", 1.0);
        let order_id = simulator.submit_order(signal, Utc::now()).unwrap();

        let candle = create_test_candle(50_000.0, 49_900.0, 50_100.0);
        let fills = simulator.process_candle(&candle);

        assert_eq!(fills.len(), 1);
        assert_eq!(fills[0].order_id, order_id);
        assert_eq!(fills[0].price, 50_000.0);
    }

    #[test]
    fn test_limit_order_execution() {
        let config = ExecutionConfig::default();
        let mut simulator = ExecutionSimulator::new(config);

        // Buy limit at 49,500
        let mut signal = Signal::buy("BTC-USD-PERP", 1.0);
        if let Signal::Buy {
            ref mut limit_price,
            ..
        } = signal
        {
            *limit_price = Some(49_500.0);
        }

        simulator.submit_order(signal, Utc::now()).unwrap();

        // Candle doesn't reach limit price
        let candle1 = create_test_candle(50_000.0, 49_600.0, 50_100.0);
        let fills1 = simulator.process_candle(&candle1);
        assert_eq!(fills1.len(), 0);

        // Candle reaches limit price
        let candle2 = create_test_candle(49_400.0, 49_300.0, 49_700.0);
        let fills2 = simulator.process_candle(&candle2);
        assert_eq!(fills2.len(), 1);
    }

    #[test]
    fn test_commission_calculation() {
        let config = ExecutionConfig {
            commission_rate: 0.001,
            ..Default::default()
        };
        let simulator = ExecutionSimulator::new(config);

        let commission = simulator.calculate_commission(1.0, 50_000.0);
        assert_eq!(commission, 50.0); // 0.1% of 50,000
    }
}
