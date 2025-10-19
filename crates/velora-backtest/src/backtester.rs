//! Main backtester orchestrator.

use crate::config::BacktestConfig;
use crate::errors::{BacktestError, BacktestResult};
use crate::execution::{ExecutionSimulator, Fill};
use crate::performance::{calculate_metrics, PerformanceMetrics};
use crate::portfolio::{CompletedTrade, EquityPoint, Portfolio};
use serde::{Deserialize, Serialize};
use velora_core::types::{Candle, Side};
use velora_strategy::{MarketSnapshot, PositionSide, Signal, Strategy, StrategyContext};

/// Main backtester struct
pub struct Backtester {
    config: BacktestConfig,
    strategy: Option<Box<dyn Strategy>>,
}

/// Complete backtest report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestReport {
    /// Configuration used
    pub config: BacktestConfig,

    /// Performance metrics
    pub metrics: PerformanceMetrics,

    /// Equity curve over time
    pub equity_curve: Vec<EquityPoint>,

    /// All completed trades
    pub trades: Vec<CompletedTrade>,
}

impl Backtester {
    /// Create a new backtester
    pub fn new(config: BacktestConfig) -> Self {
        Self {
            config,
            strategy: None,
        }
    }

    /// Add a strategy to backtest
    pub fn with_strategy(mut self, strategy: Box<dyn Strategy>) -> Self {
        self.strategy = Some(strategy);
        self
    }

    /// Run the backtest
    pub async fn run(mut self, candles: Vec<Candle>) -> BacktestResult<BacktestReport> {
        // Validate we have a strategy
        let mut strategy = self
            .strategy
            .take()
            .ok_or_else(|| BacktestError::InvalidConfig("No strategy provided".to_string()))?;

        // Validate we have data
        if candles.is_empty() {
            return Err(BacktestError::DataError("No candles provided".to_string()));
        }

        // Initialize components
        let mut portfolio = Portfolio::new(self.config.initial_capital);
        let mut simulator = ExecutionSimulator::new(self.config.execution.clone());
        let ctx = StrategyContext::new(self.config.initial_capital);

        // Initialize strategy
        strategy.initialize(&ctx).await?;

        // Sort candles by timestamp
        let mut sorted_candles = candles;
        sorted_candles.sort_by_key(|c| c.timestamp);

        println!("Running backtest with {} candles...", sorted_candles.len());

        // Main event loop
        for (idx, candle) in sorted_candles.iter().enumerate() {
            if idx % 1000 == 0 && idx > 0 {
                println!("Processed {idx} candles...");
            }

            // 1. Update market data in context
            let snapshot = MarketSnapshot {
                last_price: candle.close.into_inner(),
                timestamp: candle.timestamp,
                best_bid: Some(candle.close.into_inner() - 0.5),
                best_ask: Some(candle.close.into_inner() + 0.5),
                volume_24h: Some(candle.volume.into_inner()),
            };
            ctx.update_market_snapshot(candle.symbol.as_str(), snapshot)?;
            ctx.add_candle(candle.symbol.as_str(), candle.clone())?;

            // 2. Process pending orders (check for fills)
            let fills = simulator.process_candle(candle);
            for fill in fills {
                self.process_fill(&fill, &mut portfolio, &ctx)?;
            }

            // 3. Update portfolio prices
            portfolio.update_price(
                candle.symbol.as_str().to_string(),
                candle.close.into_inner(),
            );

            // 4. Call strategy with new candle
            let signal = strategy.on_candle(candle, &ctx).await?;

            // 5. Execute signal if actionable
            if signal.is_actionable() {
                self.execute_signal(
                    signal,
                    &mut simulator,
                    &mut portfolio,
                    &ctx,
                    candle.timestamp,
                )?;
            }

            // 6. Record equity snapshot
            portfolio.record_snapshot(candle.timestamp);
        }

        // Shutdown strategy
        strategy.shutdown(&ctx).await?;

        println!("Backtest complete!");
        println!("Processed {} candles", sorted_candles.len());
        println!("Completed {} trades", portfolio.trades().len());

        // Calculate metrics
        let metrics = calculate_metrics(
            portfolio.equity_curve(),
            portfolio.trades(),
            self.config.initial_capital,
        );

        // Build report
        let report = BacktestReport {
            config: self.config,
            metrics,
            equity_curve: portfolio.equity_curve().to_vec(),
            trades: portfolio.trades().to_vec(),
        };

        Ok(report)
    }

    /// Process a fill event
    fn process_fill(
        &self,
        fill: &Fill,
        portfolio: &mut Portfolio,
        ctx: &StrategyContext,
    ) -> BacktestResult<()> {
        match fill.side {
            Side::Buy => {
                // Opening a long position or closing a short
                if portfolio.has_position(&fill.symbol) {
                    // Closing short position
                    portfolio.close_position(
                        &fill.symbol,
                        fill.price,
                        fill.commission,
                        fill.timestamp,
                    );
                    ctx.remove_position(&fill.symbol)?;
                } else {
                    // Opening long position
                    portfolio.open_position(
                        fill.symbol.clone(),
                        PositionSide::Long,
                        fill.quantity,
                        fill.price,
                        fill.commission,
                        fill.timestamp,
                    );

                    if let Some(position) = portfolio.get_position(&fill.symbol) {
                        ctx.update_position(position.clone())?;
                    }
                }
            }
            Side::Sell => {
                // Opening a short position or closing a long
                if portfolio.has_position(&fill.symbol) {
                    // Closing long position
                    portfolio.close_position(
                        &fill.symbol,
                        fill.price,
                        fill.commission,
                        fill.timestamp,
                    );
                    ctx.remove_position(&fill.symbol)?;
                } else {
                    // Opening short position
                    portfolio.open_position(
                        fill.symbol.clone(),
                        PositionSide::Short,
                        fill.quantity,
                        fill.price,
                        fill.commission,
                        fill.timestamp,
                    );

                    if let Some(position) = portfolio.get_position(&fill.symbol) {
                        ctx.update_position(position.clone())?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Execute a signal
    fn execute_signal(
        &self,
        signal: Signal,
        simulator: &mut ExecutionSimulator,
        portfolio: &mut Portfolio,
        _ctx: &StrategyContext,
        timestamp: chrono::DateTime<chrono::Utc>,
    ) -> BacktestResult<()> {
        match signal {
            Signal::Buy { .. } | Signal::Sell { .. } => {
                // Submit new order
                simulator.submit_order(signal, timestamp)?;
            }
            Signal::Close { ref symbol, .. } => {
                // Close existing position
                if let Some(position) = portfolio.get_position(symbol) {
                    let close_side = match position.side {
                        PositionSide::Long => Side::Sell,
                        PositionSide::Short => Side::Buy,
                    };

                    simulator.submit_close_order(
                        symbol.clone(),
                        position.quantity,
                        close_side,
                        timestamp,
                    )?;
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl BacktestReport {
    /// Print a summary of the backtest results
    pub fn print_summary(&self) {
        self.metrics.print_summary();
    }

    /// Export equity curve to JSON
    pub fn export_equity_curve_json(&self) -> BacktestResult<String> {
        Ok(serde_json::to_string_pretty(&self.equity_curve)?)
    }

    /// Export trades to JSON
    pub fn export_trades_json(&self) -> BacktestResult<String> {
        Ok(serde_json::to_string_pretty(&self.trades)?)
    }

    /// Export full report to JSON
    pub fn to_json(&self) -> BacktestResult<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use chrono::Utc;
    use velora_core::types::Symbol;
    use velora_strategy::{StrategyConfig, StrategyState};

    struct DummyStrategy {
        config: StrategyConfig,
        state: StrategyState,
    }

    impl DummyStrategy {
        fn new() -> Self {
            Self {
                config: StrategyConfig::new("Dummy"),
                state: StrategyState::Initializing,
            }
        }
    }

    #[async_trait]
    impl Strategy for DummyStrategy {
        fn name(&self) -> &str {
            "Dummy"
        }
        fn config(&self) -> &StrategyConfig {
            &self.config
        }
        fn state(&self) -> StrategyState {
            self.state
        }

        async fn initialize(
            &mut self,
            _ctx: &StrategyContext,
        ) -> velora_strategy::StrategyResult<()> {
            self.state = StrategyState::Running;
            Ok(())
        }

        fn reset(&mut self) {
            self.state = StrategyState::Initializing;
        }
    }

    #[tokio::test]
    async fn test_backtester_creation() {
        let config = BacktestConfig::new();
        let backtester = Backtester::new(config);
        assert!(backtester.strategy.is_none());
    }

    #[tokio::test]
    async fn test_backtester_with_strategy() {
        let config = BacktestConfig::new();
        let strategy = Box::new(DummyStrategy::new());
        let backtester = Backtester::new(config).with_strategy(strategy);
        assert!(backtester.strategy.is_some());
    }

    #[tokio::test]
    async fn test_backtester_run_empty_data() {
        let config = BacktestConfig::new();
        let strategy = Box::new(DummyStrategy::new());
        let backtester = Backtester::new(config).with_strategy(strategy);

        let result = backtester.run(vec![]).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_backtester_run_with_data() {
        let config = BacktestConfig::new()
            .with_capital(10_000.0)
            .with_symbols(vec!["BTC-USD-PERP".to_string()]);

        let strategy = Box::new(DummyStrategy::new());
        let backtester = Backtester::new(config).with_strategy(strategy);

        // Create some test candles
        let candles = vec![Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp: Utc::now(),
            open: 50000.0.into(),
            high: 50100.0.into(),
            low: 49900.0.into(),
            close: 50000.0.into(),
            volume: 100.0.into(),
        }];

        let report = backtester.run(candles).await.unwrap();
        assert_eq!(report.metrics.total_trades, 0); // Dummy strategy doesn't trade
    }
}
