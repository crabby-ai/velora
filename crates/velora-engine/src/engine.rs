//! Main trading engine orchestration

use crate::config::EngineConfig;
use crate::errors::{EngineError, EngineResult};
use crate::events::{Fill, MarketEvent, OrderStatus};
use crate::execution::ExecutionHandler;
use crate::order_manager::{Order, OrderManager};
use crate::position_tracker::{EquitySnapshot, PositionTracker};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::broadcast;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio::time::interval;
use tracing::{debug, info, warn};
use velora_core::{Candle, OrderType};
use velora_strategy::{Signal, Strategy, StrategyContext};

/// Main trading engine
pub struct TradingEngine {
    /// Engine configuration
    config: EngineConfig,

    /// Trading strategy
    strategy: Option<Box<dyn Strategy>>,

    /// Order manager
    order_manager: OrderManager,

    /// Position tracker
    position_tracker: PositionTracker,

    /// Execution handler
    execution_handler: ExecutionHandler,

    /// Strategy context
    context: StrategyContext,

    /// Market event channel sender (for injecting events in examples)
    market_tx: Option<UnboundedSender<MarketEvent>>,

    /// Shutdown channel
    shutdown_tx: Option<broadcast::Sender<()>>,

    /// Engine state
    state: EngineState,

    /// Engine start time
    start_time: Option<DateTime<Utc>>,
}

/// Engine state
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EngineState {
    /// Not started yet
    Idle,

    /// Currently running
    Running,

    /// Paused (can be resumed)
    Paused,

    /// Shutting down
    ShuttingDown,

    /// Stopped
    Stopped,
}

/// Engine status report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    /// Current state
    pub state: EngineState,

    /// Uptime duration
    pub uptime_secs: i64,

    /// Total number of orders
    pub total_orders: usize,

    /// Active orders count
    pub active_orders: usize,

    /// Total fills
    pub total_fills: usize,

    /// Open positions count
    pub open_positions: usize,

    /// Current equity
    pub current_equity: f64,

    /// Unrealized P&L
    pub unrealized_pnl: f64,

    /// Realized P&L
    pub realized_pnl: f64,

    /// Last update time
    pub last_update: DateTime<Utc>,
}

impl TradingEngine {
    /// Create a new trading engine
    pub fn new(config: EngineConfig) -> Self {
        let order_manager = OrderManager::new(config.max_orders_per_second);
        let position_tracker = PositionTracker::new(config.initial_capital);
        let execution_handler = ExecutionHandler::new(config.mode);
        let context = StrategyContext::new(config.initial_capital);

        Self {
            config,
            strategy: None,
            order_manager,
            position_tracker,
            execution_handler,
            context,
            market_tx: None,
            shutdown_tx: None,
            state: EngineState::Idle,
            start_time: None,
        }
    }

    /// Attach a trading strategy
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Start the trading engine with an external market event receiver
    /// This is useful for examples and testing where you want to control the event flow
    pub async fn start_with_receiver(
        &mut self,
        market_rx: UnboundedReceiver<MarketEvent>,
    ) -> EngineResult<()> {
        if self.state != EngineState::Idle {
            return Err(EngineError::AlreadyRunning);
        }

        if self.strategy.is_none() {
            return Err(EngineError::InvalidConfig(
                "No strategy provided".to_string(),
            ));
        }

        info!("Starting trading engine in {:?} mode", self.config.mode);

        self.state = EngineState::Running;
        self.start_time = Some(Utc::now());

        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        // Run event loop with provided receiver
        self.run_event_loop(market_rx, shutdown_rx).await?;

        Ok(())
    }

    /// Start the trading engine
    /// In production, you would connect to exchange websocket feeds here
    pub async fn start(&mut self) -> EngineResult<()> {
        if self.state != EngineState::Idle {
            return Err(EngineError::AlreadyRunning);
        }

        if self.strategy.is_none() {
            return Err(EngineError::InvalidConfig(
                "No strategy provided".to_string(),
            ));
        }

        info!("Starting trading engine in {:?} mode", self.config.mode);

        self.state = EngineState::Running;
        self.start_time = Some(Utc::now());

        // Create channels
        let (market_tx, market_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = broadcast::channel(1);

        self.market_tx = Some(market_tx);
        self.shutdown_tx = Some(shutdown_tx);

        // TODO: In production, spawn tasks to connect to exchange feeds
        // and send events to market_tx

        // Run event loop
        self.run_event_loop(market_rx, shutdown_rx).await?;

        Ok(())
    }

    /// Stop the trading engine
    pub async fn stop(&mut self) -> EngineResult<()> {
        if self.state == EngineState::Stopped {
            return Ok(());
        }

        info!("Stopping trading engine");
        self.state = EngineState::ShuttingDown;

        // Cancel all pending orders
        let active_orders = self.order_manager.get_active_orders();
        for order in active_orders {
            info!("Cancelling order: {}", order.id);
            self.execution_handler.cancel_order(order.id).await?;
        }

        // Send shutdown signal
        if let Some(ref shutdown_tx) = self.shutdown_tx {
            let _ = shutdown_tx.send(());
        }

        self.state = EngineState::Stopped;
        info!("Trading engine stopped");

        Ok(())
    }

    /// Get current engine status
    pub fn status(&self) -> EngineStatus {
        let uptime_secs = self
            .start_time
            .map(|start| (Utc::now() - start).num_seconds())
            .unwrap_or(0);

        EngineStatus {
            state: self.state,
            uptime_secs,
            total_orders: self.order_manager.total_orders(),
            active_orders: self.order_manager.get_active_orders().len(),
            total_fills: 0, // TODO: Track fills
            open_positions: self.position_tracker.position_count(),
            current_equity: self.position_tracker.total_equity(),
            unrealized_pnl: self.position_tracker.total_unrealized_pnl(),
            realized_pnl: self.position_tracker.total_realized_pnl(),
            last_update: Utc::now(),
        }
    }

    /// Get equity snapshots
    pub fn get_equity_history(&self) -> &[EquitySnapshot] {
        self.position_tracker.get_equity_history()
    }

    /// Main event loop
    async fn run_event_loop(
        &mut self,
        mut market_rx: UnboundedReceiver<MarketEvent>,
        mut shutdown_rx: broadcast::Receiver<()>,
    ) -> EngineResult<()> {
        let mut heartbeat = interval(Duration::from_millis(self.config.heartbeat_interval_ms));

        loop {
            tokio::select! {
                // Process market events
                event = market_rx.recv() => {
                    match event {
                        Some(MarketEvent::Candle(candle)) => {
                            if let Err(e) = self.process_candle(candle).await {
                                warn!("Error processing candle: {}", e);
                            }
                        }
                        Some(MarketEvent::OrderUpdate(update)) => {
                            if let Err(e) = self.order_manager.update_order(update.order_id, update) {
                                warn!("Error updating order: {}", e);
                            }
                        }
                        Some(MarketEvent::Error(msg)) => {
                            warn!("Market error: {}", msg);
                        }
                        Some(MarketEvent::Disconnected) => {
                            warn!("Market disconnected");
                        }
                        Some(MarketEvent::Reconnected) => {
                            info!("Market reconnected");
                        }
                        None => {
                            info!("Market channel closed, shutting down");
                            break;
                        }
                    }
                }

                // Heartbeat tick
                _ = heartbeat.tick() => {
                    debug!("Heartbeat");
                    self.on_heartbeat().await?;
                }

                // Shutdown signal
                _ = shutdown_rx.recv() => {
                    info!("Received shutdown signal");
                    break;
                }
            }
        }

        self.state = EngineState::Stopped;
        info!("Event loop stopped");

        Ok(())
    }

    /// Process a new candle
    async fn process_candle(&mut self, candle: Candle) -> EngineResult<()> {
        debug!("Processing candle for {}", candle.symbol);

        // Update position prices with current market data
        self.position_tracker
            .update_position_price(candle.symbol.as_str(), candle.close.into_inner());

        // Update strategy context with market snapshot
        let snapshot = velora_strategy::MarketSnapshot {
            last_price: candle.close.into_inner(),
            timestamp: candle.timestamp,
            best_bid: None,
            best_ask: None,
            volume_24h: Some(candle.volume.into_inner()),
        };
        self.context
            .update_market_snapshot(candle.symbol.as_str(), snapshot)?;

        // Process any fills from execution handler
        let fills = self.execution_handler.drain_fills();
        for fill in fills {
            self.process_fill(fill).await?;
        }

        // Call strategy
        let strategy = self
            .strategy
            .as_mut()
            .ok_or_else(|| EngineError::InvalidConfig("No strategy".to_string()))?;

        let signal = strategy.on_candle(&candle, &self.context).await?;

        // Execute signal if actionable
        if signal.is_actionable() {
            self.execute_signal(signal, &candle).await?;
        }

        Ok(())
    }

    /// Execute a trading signal
    async fn execute_signal(&mut self, signal: Signal, candle: &Candle) -> EngineResult<()> {
        match signal {
            Signal::Buy {
                symbol,
                quantity,
                limit_price,
                stop_price,
                ..
            } => {
                self.place_order(
                    symbol,
                    velora_core::Side::Buy,
                    quantity,
                    limit_price.or(Some(candle.close.into_inner())),
                    stop_price,
                )
                .await?;
            }
            Signal::Sell {
                symbol,
                quantity,
                limit_price,
                stop_price,
                ..
            } => {
                self.place_order(
                    symbol,
                    velora_core::Side::Sell,
                    quantity,
                    limit_price.or(Some(candle.close.into_inner())),
                    stop_price,
                )
                .await?;
            }
            Signal::Close { .. } | Signal::Modify { .. } => {
                return Err(EngineError::OrderError(
                    "Close and Modify signals not yet implemented in live engine".to_string(),
                ));
            }
            Signal::Hold => {}
        }

        Ok(())
    }

    /// Place an order
    async fn place_order(
        &mut self,
        symbol: String,
        side: velora_core::Side,
        quantity: f64,
        price: Option<f64>,
        stop_price: Option<f64>,
    ) -> EngineResult<()> {
        // Determine order type
        let order_type = match (price, stop_price) {
            (Some(_), None) => OrderType::Limit,
            (None, None) => OrderType::Market,
            (Some(_), Some(_)) => OrderType::StopLimit,
            (None, Some(_)) => OrderType::StopMarket,
        };

        // Create order
        let order = Order::new(symbol, side, order_type, quantity, price);

        info!(
            "Placing {:?} {} order for {} @ {:?}",
            side, order.symbol, order.quantity, price
        );

        // Submit to order manager (rate limit check)
        self.order_manager.submit_order(order.clone())?;

        // Execute via execution handler
        let order_id = self.execution_handler.submit_order(&order).await?;

        info!("Order submitted: {}", order_id);

        Ok(())
    }

    /// Process a fill
    async fn process_fill(&mut self, fill: Fill) -> EngineResult<()> {
        info!(
            "Processing fill: {:?} {} {} @ {}",
            fill.side, fill.quantity, fill.symbol, fill.price
        );

        // Update order manager
        let update = crate::events::OrderUpdate {
            order_id: fill.order_id,
            status: OrderStatus::Filled,
            filled_quantity: fill.quantity,
            average_price: fill.price,
            timestamp: fill.timestamp,
            error_message: None,
        };

        self.order_manager.update_order(fill.order_id, update)?;

        // Update positions
        self.position_tracker.process_fill(&fill)?;

        // Update strategy context with new position
        if let Some(position) = self.position_tracker.get_position(&fill.symbol) {
            let strategy_position = velora_strategy::Position {
                symbol: position.symbol.clone(),
                side: position.side,
                quantity: position.quantity,
                entry_price: position.average_entry_price,
                current_price: position.current_price,
                opened_at: position.opened_at,
                updated_at: position.last_updated,
                stop_loss: None,
                take_profit: None,
                unrealized_pnl: position.unrealized_pnl,
                metadata: std::collections::HashMap::new(),
            };

            self.context.update_position(strategy_position)?;
        } else {
            // Position was closed
            self.context.remove_position(&fill.symbol)?;
        }

        Ok(())
    }

    /// Heartbeat callback
    async fn on_heartbeat(&mut self) -> EngineResult<()> {
        // Record equity snapshot
        self.position_tracker.record_snapshot();

        // Clean up old completed orders (keep last 1000)
        // TODO: Implement in order_manager

        Ok(())
    }
}
