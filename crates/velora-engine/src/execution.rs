//! Order execution handling (dry-run simulation for now)

use crate::config::ExecutionMode;
use crate::errors::{EngineError, EngineResult};
use crate::events::{Fill, OrderId, OrderStatus, OrderUpdate};
use crate::order_manager::Order;
use chrono::Utc;
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Handles order execution and fills
pub struct ExecutionHandler {
    /// Execution mode (Live or DryRun)
    mode: ExecutionMode,

    /// Fills generated
    fills: Vec<Fill>,

    /// Simulated order book for dry-run mode
    simulated_prices: HashMap<String, f64>,

    /// Commission rate (as fraction, e.g., 0.001 = 0.1%)
    commission_rate: f64,
}

impl ExecutionHandler {
    /// Create a new execution handler
    pub fn new(mode: ExecutionMode) -> Self {
        Self {
            mode,
            fills: Vec::new(),
            simulated_prices: HashMap::new(),
            commission_rate: 0.001, // 0.1% default
        }
    }

    /// Set commission rate
    pub fn with_commission_rate(mut self, rate: f64) -> Self {
        self.commission_rate = rate;
        self
    }

    /// Submit an order for execution
    pub async fn submit_order(&mut self, order: &Order) -> EngineResult<OrderId> {
        match self.mode {
            ExecutionMode::Live => {
                // TODO: Integrate with real exchange API
                warn!("Live trading not yet implemented - falling back to dry-run");
                self.submit_order_dry_run(order).await
            }
            ExecutionMode::DryRun => self.submit_order_dry_run(order).await,
        }
    }

    /// Submit order in dry-run mode (simulated execution)
    async fn submit_order_dry_run(&mut self, order: &Order) -> EngineResult<OrderId> {
        info!(
            order_id = %order.id,
            symbol = %order.symbol,
            side = ?order.side,
            quantity = order.quantity,
            price = ?order.price,
            "[DRY-RUN] Simulating order submission"
        );

        // In dry-run, we simulate instant fills for market orders
        // For limit orders, we'd need to wait for price to reach the limit

        match order.order_type {
            velora_core::OrderType::Market => {
                // Get current price (or use order price if available)
                let fill_price = self
                    .simulated_prices
                    .get(&order.symbol)
                    .copied()
                    .or(order.price)
                    .ok_or_else(|| {
                        EngineError::MarketDataError(format!(
                            "No price available for {}",
                            order.symbol
                        ))
                    })?;

                // Create fill immediately
                let commission = order.quantity * fill_price * self.commission_rate;

                let fill = Fill {
                    order_id: order.id,
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    price: fill_price,
                    commission,
                    timestamp: Utc::now(),
                };

                self.fills.push(fill.clone());

                debug!(
                    order_id = %order.id,
                    fill_price = fill_price,
                    commission = commission,
                    "[DRY-RUN] Order filled"
                );

                Ok(order.id)
            }
            velora_core::OrderType::Limit => {
                // For limit orders in dry-run, we just accept them
                // In a real implementation, we'd track them and fill when price is hit
                info!(
                    order_id = %order.id,
                    "[DRY-RUN] Limit order accepted (instant fill simulation)"
                );

                let fill_price = order.price.ok_or_else(|| {
                    EngineError::OrderError("Limit order must have price".to_string())
                })?;

                let commission = order.quantity * fill_price * self.commission_rate;

                let fill = Fill {
                    order_id: order.id,
                    symbol: order.symbol.clone(),
                    side: order.side,
                    quantity: order.quantity,
                    price: fill_price,
                    commission,
                    timestamp: Utc::now(),
                };

                self.fills.push(fill);
                Ok(order.id)
            }
            velora_core::OrderType::StopLimit | velora_core::OrderType::StopMarket => {
                // Stop orders not yet implemented in dry-run
                Err(EngineError::OrderError(
                    "Stop orders not yet implemented in dry-run mode".to_string(),
                ))
            }
        }
    }

    /// Cancel an order
    pub async fn cancel_order(&mut self, order_id: OrderId) -> EngineResult<()> {
        match self.mode {
            ExecutionMode::Live => {
                // TODO: Real exchange cancellation
                warn!("[DRY-RUN] Would cancel order: {}", order_id);
                Ok(())
            }
            ExecutionMode::DryRun => {
                info!("[DRY-RUN] Simulating order cancellation: {}", order_id);
                Ok(())
            }
        }
    }

    /// Update simulated market price (for dry-run mode)
    pub fn update_market_price(&mut self, symbol: String, price: f64) {
        self.simulated_prices.insert(symbol, price);
    }

    /// Get pending fills (and clear them)
    pub fn drain_fills(&mut self) -> Vec<Fill> {
        std::mem::take(&mut self.fills)
    }

    /// Generate order update for a filled order
    pub fn create_fill_update(&self, order: &Order, fill: &Fill) -> OrderUpdate {
        OrderUpdate {
            order_id: order.id,
            status: if fill.quantity == order.quantity {
                OrderStatus::Filled
            } else {
                OrderStatus::PartiallyFilled
            },
            filled_quantity: fill.quantity,
            average_price: fill.price,
            timestamp: fill.timestamp,
            error_message: None,
        }
    }

    /// Sync orders with exchange (for live mode)
    pub async fn sync_orders(&self) -> EngineResult<Vec<OrderUpdate>> {
        match self.mode {
            ExecutionMode::Live => {
                // TODO: Fetch order updates from exchange
                Ok(Vec::new())
            }
            ExecutionMode::DryRun => {
                // In dry-run, no external sync needed
                Ok(Vec::new())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::order_manager::Order;
    use velora_core::{OrderType, Side};

    #[tokio::test]
    async fn test_submit_market_order_dry_run() {
        let mut handler = ExecutionHandler::new(ExecutionMode::DryRun).with_commission_rate(0.001);

        // Set market price
        handler.update_market_price("BTC-USD-PERP".to_string(), 50_000.0);

        // Create market order
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );

        // Submit order
        let result = handler.submit_order(&order).await;
        assert!(result.is_ok());

        // Check fill
        let fills = handler.drain_fills();
        assert_eq!(fills.len(), 1);

        let fill = &fills[0];
        assert_eq!(fill.quantity, 0.1);
        assert_eq!(fill.price, 50_000.0);
        assert_eq!(fill.commission, 5.0); // 0.1 * 50,000 * 0.001
    }

    #[tokio::test]
    async fn test_submit_limit_order_dry_run() {
        let mut handler = ExecutionHandler::new(ExecutionMode::DryRun).with_commission_rate(0.001);

        // Create limit order
        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Limit,
            0.1,
            Some(49_000.0),
        );

        // Submit order
        let result = handler.submit_order(&order).await;
        assert!(result.is_ok());

        // Check fill (instant in dry-run)
        let fills = handler.drain_fills();
        assert_eq!(fills.len(), 1);

        let fill = &fills[0];
        assert_eq!(fill.price, 49_000.0);
        assert_eq!(fill.commission, 4.9); // 0.1 * 49,000 * 0.001
    }

    #[tokio::test]
    async fn test_create_fill_update() {
        let handler = ExecutionHandler::new(ExecutionMode::DryRun);

        let order = Order::new(
            "BTC-USD-PERP".to_string(),
            Side::Buy,
            OrderType::Market,
            0.1,
            None,
        );

        let fill = Fill {
            order_id: order.id,
            symbol: "BTC-USD-PERP".to_string(),
            side: Side::Buy,
            quantity: 0.1,
            price: 50_000.0,
            commission: 5.0,
            timestamp: Utc::now(),
        };

        let update = handler.create_fill_update(&order, &fill);

        assert_eq!(update.status, OrderStatus::Filled);
        assert_eq!(update.filled_quantity, 0.1);
        assert_eq!(update.average_price, 50_000.0);
    }
}
