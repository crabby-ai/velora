//! Dry-run trading example
//!
//! This example demonstrates running a live trading engine in dry-run mode
//! with simulated market data. Perfect for testing strategies without risking real money.

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::mpsc;
use velora_core::{Candle, Symbol};
use velora_engine::{EngineConfig, ExecutionMode, MarketEvent, TradingEngine};
use velora_strategy::{
    Signal, Strategy, StrategyConfig, StrategyContext, StrategyResult, StrategyState,
};

// Simple test strategy that buys on every 5th candle
struct TestStrategy {
    config: StrategyConfig,
    state: StrategyState,
    symbol: String,
    candle_count: usize,
    has_position: bool,
}

impl TestStrategy {
    fn new(symbol: impl Into<String>) -> Self {
        let symbol = symbol.into();
        let config = StrategyConfig::new("Test Strategy")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0);

        Self {
            config,
            state: StrategyState::Running,
            symbol,
            candle_count: 0,
            has_position: false,
        }
    }
}

#[async_trait]
impl Strategy for TestStrategy {
    fn name(&self) -> &str {
        "Test Strategy"
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn state(&self) -> StrategyState {
        self.state
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
        self.candle_count = 0;
        self.has_position = false;
    }

    async fn on_candle(
        &mut self,
        candle: &Candle,
        _ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        self.candle_count += 1;

        // Buy on every 20th candle if we don't have a position
        if self.candle_count % 20 == 0 && !self.has_position {
            self.has_position = true;
            println!("  📈 Generating BUY signal at candle {}", self.candle_count);
            return Ok(Signal::Buy {
                symbol: self.symbol.clone(),
                quantity: 0.01,
                limit_price: None,
                stop_price: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        // Sell 10 candles after buying
        if self.candle_count % 20 == 10 && self.has_position {
            self.has_position = false;
            println!(
                "  📉 Generating SELL signal at candle {}",
                self.candle_count
            );
            return Ok(Signal::Sell {
                symbol: self.symbol.clone(),
                quantity: 0.01,
                limit_price: None,
                stop_price: None,
                metadata: std::collections::HashMap::new(),
            });
        }

        Ok(Signal::Hold)
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    println!("\n=== Velora Live Trading Engine - Dry Run Example ===\n");

    // Create strategy
    let strategy = TestStrategy::new("BTC-USD-PERP");
    println!("✓ Created Test Strategy");

    // Configure engine for dry-run
    let config = EngineConfig::builder()
        .mode(ExecutionMode::DryRun)
        .add_symbol("BTC-USD-PERP".to_string())
        .initial_capital(10_000.0)
        .max_orders_per_second(5)
        .heartbeat_interval_ms(5000)
        .enable_risk_checks(true)
        .build();

    println!("✓ Configured engine:");
    println!("  - Mode: Dry-Run (paper trading)");
    println!("  - Capital: $10,000");
    println!("  - Symbol: BTC-USD-PERP");
    println!("  - Max Orders/sec: 5");

    // Create engine
    let mut engine = TradingEngine::new(config).with_strategy(Box::new(strategy));

    println!("✓ Trading engine created\n");

    // Create channel for market events
    let (market_tx, market_rx) = mpsc::unbounded_channel();

    // Spawn task to send simulated market data
    let data_task = tokio::spawn(async move {
        println!("📊 Starting market data simulation...\n");

        // Give engine time to start
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        // Generate simulated candles
        let mut base_price = 45_000.0;
        let start_time = Utc::now();

        for i in 0..100 {
            // Simulate price movement
            let change = (i as f64 * 100.0).sin() * 500.0;
            base_price += change;

            let candle = Candle {
                symbol: Symbol::new("BTC-USD-PERP"),
                timestamp: start_time + chrono::Duration::minutes(i),
                open: (base_price - 50.0).into(),
                high: (base_price + 100.0).into(),
                low: (base_price - 100.0).into(),
                close: base_price.into(),
                volume: (1000.0 + (i as f64 * 10.0)).into(),
            };

            // Send candle to engine
            if let Err(e) = market_tx.send(MarketEvent::Candle(candle)) {
                eprintln!("Failed to send candle: {e}");
                break;
            }

            // Print progress every 10 candles
            if (i + 1) % 10 == 0 {
                println!(
                    "  Processed {} candles... (price: ${:.2})",
                    i + 1,
                    base_price
                );
            }

            // Simulate real-time data (1 candle per 100ms for demo)
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        println!("\n✓ Market data simulation complete (100 candles)");

        // Drop sender to signal engine to stop
        drop(market_tx);

        println!("✓ Signaled engine to stop\n");

        // Wait for engine to finish processing
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    });

    // Start the engine - it will process events until the channel closes
    let engine_task = tokio::spawn(async move {
        if let Err(e) = engine.start_with_receiver(market_rx).await {
            eprintln!("Engine error: {e}");
        }

        // Print final status
        println!("=== Final Engine Status ===\n");
        let status = engine.status();
        println!("State: {:?}", status.state);
        println!("Uptime: {} seconds", status.uptime_secs);
        println!("Total Orders: {}", status.total_orders);
        println!("Active Orders: {}", status.active_orders);
        println!("Open Positions: {}", status.open_positions);
        println!("\n=== Performance ===\n");
        println!("Current Equity: ${:.2}", status.current_equity);
        println!("Unrealized P&L: ${:.2}", status.unrealized_pnl);
        println!("Realized P&L: ${:.2}", status.realized_pnl);
        println!(
            "Total P&L: ${:.2}",
            status.unrealized_pnl + status.realized_pnl
        );

        let return_pct = ((status.current_equity - 10_000.0) / 10_000.0) * 100.0;
        println!("Return: {return_pct:.2}%");

        // Print equity curve
        println!("\n=== Equity Curve ===\n");
        let history = engine.get_equity_history();
        if !history.is_empty() {
            println!("Snapshots recorded: {}", history.len());

            // Show first and last few
            if history.len() <= 5 {
                for snapshot in history {
                    println!(
                        "  {} - Equity: ${:.2}, P&L: ${:.2}",
                        snapshot.timestamp.format("%H:%M:%S"),
                        snapshot.total_equity,
                        snapshot.unrealized_pnl + snapshot.realized_pnl
                    );
                }
            } else {
                // Show first 2 and last 2
                for snapshot in &history[..2] {
                    println!(
                        "  {} - Equity: ${:.2}, P&L: ${:.2}",
                        snapshot.timestamp.format("%H:%M:%S"),
                        snapshot.total_equity,
                        snapshot.unrealized_pnl + snapshot.realized_pnl
                    );
                }
                println!("  ...");
                for snapshot in &history[history.len() - 2..] {
                    println!(
                        "  {} - Equity: ${:.2}, P&L: ${:.2}",
                        snapshot.timestamp.format("%H:%M:%S"),
                        snapshot.total_equity,
                        snapshot.unrealized_pnl + snapshot.realized_pnl
                    );
                }
            }
        } else {
            println!("No equity snapshots recorded");
        }

        engine
    });

    // Wait for data simulation to complete
    data_task.await?;

    // Wait for engine to finish
    let final_engine = engine_task.await?;

    println!("\n=== Dry Run Complete ===\n");
    println!("This was a paper trading simulation.");
    println!("No real orders were placed or money was risked.");
    println!("\nTo run with live trading:");
    println!("1. Set ExecutionMode::Live");
    println!("2. Configure exchange API credentials");
    println!("3. START SMALL and test thoroughly!\n");

    // Could export data here if needed
    let _ = final_engine;

    Ok(())
}
