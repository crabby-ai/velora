//! Live Trading Example (Dry-Run Mode)
//!
//! This example demonstrates how to run a live trading engine in dry-run (paper trading) mode.
//! This is perfect for testing strategies in real-time without risking real money.
//!
//! Run with:
//! ```bash
//! cargo run --example live_trading
//! ```

use async_trait::async_trait;
use velora::prelude::*;

/// A simple RSI-based strategy
struct RsiStrategy {
    config: StrategyConfig,
    state: StrategyState,
    symbol: String,
}

impl RsiStrategy {
    fn new(symbol: impl Into<String>) -> StrategyResult<Self> {
        let symbol = symbol.into();
        let config = StrategyConfig::new("RSI Strategy")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0)
            .with_max_position_size(5.0);

        Ok(Self {
            config,
            state: StrategyState::Initializing,
            symbol,
        })
    }
}

#[async_trait]
impl Strategy for RsiStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn state(&self) -> StrategyState {
        self.state
    }

    async fn initialize(&mut self, _ctx: &StrategyContext) -> StrategyResult<()> {
        self.state = StrategyState::Running;
        println!("   ✓ Strategy initialized");
        Ok(())
    }

    async fn on_candle(
        &mut self,
        candle: &Candle,
        ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        // Simple logic: monitor price movements
        // In a real strategy, you would use technical indicators here

        let has_position = ctx.has_position(&self.symbol)?;
        let price = candle.close.into_inner();

        // Demo: Simple price-based signals
        if !has_position && price < 45_000.0 {
            // Buy when price is low
            let capital = ctx.available_capital()?;
            let quantity = (capital * 0.1) / price; // Use 10% of capital

            println!(
                "   📈 BUY signal generated @ ${:.2} (qty: {:.4})",
                price, quantity
            );
            return Ok(Signal::buy(&self.symbol, quantity));
        } else if has_position && price > 48_000.0 {
            // Sell when price is high
            println!("   📉 SELL signal generated @ ${:.2}", price);
            return Ok(Signal::close(&self.symbol));
        }

        Ok(Signal::Hold)
    }

    async fn on_order_filled(
        &mut self,
        _order_id: &str,
        _fill_price: f64,
        _fill_quantity: f64,
        _ctx: &StrategyContext,
    ) -> StrategyResult<()> {
        println!("   ✓ Order filled!");
        Ok(())
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
    }
}

/// Simulate market data stream
async fn simulate_market_data(tx: tokio::sync::mpsc::UnboundedSender<MarketEvent>) {
    use chrono::Utc;

    println!("\n📊 Starting market data simulation...\n");

    // Give engine time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

    let mut base_price = 46_000.0;
    let start_time = Utc::now();

    for i in 0..50 {
        // Simulate realistic BTC price movements
        let trend = (i as f64 * 0.1).sin() * 100.0;
        let volatility = (i as f64 * 0.05).cos() * 50.0;
        base_price += trend + volatility;

        let candle = Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp: start_time + chrono::Duration::minutes(i),
            open: (base_price - 25.0).into(),
            high: (base_price + 50.0).into(),
            low: (base_price - 50.0).into(),
            close: base_price.into(),
            volume: (500.0 + (i as f64 * 10.0)).into(),
        };

        if let Err(e) = tx.send(MarketEvent::Candle(candle)) {
            eprintln!("Failed to send candle: {e}");
            break;
        }

        // Print progress
        if (i + 1) % 10 == 0 {
            println!(
                "   Processed {} candles (price: ${:.2})",
                i + 1,
                base_price
            );
        }

        // Simulate real-time data (1 candle per 200ms)
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }

    println!("\n   ✓ Market data simulation complete\n");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    #[cfg(feature = "utils")]
    velora::utils::init_tracing("info")?;

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║     Velora - Live Trading Example (Dry-Run)      ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("Platform: {}\n", velora::version_string());

    // 1. Create strategy
    println!("🎯 Creating strategy...");
    let strategy = RsiStrategy::new("BTC-USD-PERP")?;
    println!("   ✓ Strategy: {}", strategy.name());

    // 2. Configure engine for dry-run
    println!("\n⚙️  Configuring trading engine...");
    let config = EngineConfig::builder()
        .mode(ExecutionMode::DryRun)
        .add_symbol("BTC-USD-PERP".to_string())
        .initial_capital(10_000.0)
        .max_orders_per_second(10)
        .heartbeat_interval_ms(5000)
        .enable_risk_checks(true)
        .build();

    println!("   ✓ Mode: DryRun (paper trading - NO REAL MONEY)");
    println!("   ✓ Initial capital: ${:,.2}", 10_000.0);
    println!("   ✓ Symbol: BTC-USD-PERP");
    println!("   ✓ Risk checks: Enabled");

    // 3. Create engine
    println!("\n🚀 Starting trading engine...");
    let mut engine = TradingEngine::new(config).with_strategy(Box::new(strategy));

    // 4. Create channel for market events
    let (market_tx, market_rx) = tokio::sync::mpsc::unbounded_channel();

    // 5. Spawn market data simulation task
    let data_task = tokio::spawn(async move {
        simulate_market_data(market_tx).await;
    });

    // 6. Start the engine
    let engine_task = tokio::spawn(async move {
        if let Err(e) = engine.start_with_receiver(market_rx).await {
            eprintln!("Engine error: {e}");
        }

        // Print final status
        println!("╔═══════════════════════════════════════════════════╗");
        println!("║              Final Performance                    ║");
        println!("╚═══════════════════════════════════════════════════╝\n");

        let status = engine.status();

        println!("📊 Engine Status:");
        println!("   State: {:?}", status.state);
        println!("   Uptime: {} seconds", status.uptime_secs);
        println!("   Total Orders: {}", status.total_orders);
        println!("   Active Orders: {}", status.active_orders);
        println!("   Open Positions: {}", status.open_positions);

        println!("\n💰 Performance:");
        println!("   Initial Capital: ${:,.2}", 10_000.0);
        println!("   Current Equity: ${:,.2}", status.current_equity);
        println!("   Unrealized P&L: ${:,.2}", status.unrealized_pnl);
        println!("   Realized P&L: ${:,.2}", status.realized_pnl);

        let total_pnl = status.unrealized_pnl + status.realized_pnl;
        let return_pct = (total_pnl / 10_000.0) * 100.0;

        println!("\n📈 Returns:");
        println!("   Total P&L: ${:,.2}", total_pnl);
        println!(
            "   Return: {}{:.2}%",
            if return_pct >= 0.0 { "+" } else { "" },
            return_pct
        );

        // Equity curve
        let history = engine.get_equity_history();
        if !history.is_empty() {
            println!("\n📊 Equity Curve:");
            println!("   Snapshots: {}", history.len());

            if history.len() >= 2 {
                let first = &history[0];
                let last = &history[history.len() - 1];
                println!(
                    "   Start: ${:,.2} @ {}",
                    first.total_equity,
                    first.timestamp.format("%H:%M:%S")
                );
                println!(
                    "   End:   ${:,.2} @ {}",
                    last.total_equity,
                    last.timestamp.format("%H:%M:%S")
                );
            }
        }

        engine
    });

    // 7. Wait for completion
    data_task.await?;
    let final_engine = engine_task.await?;

    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║              Dry Run Complete                     ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("ℹ️  This was a paper trading simulation.");
    println!("   No real orders were placed.");
    println!("   No real money was risked.\n");

    println!("⚠️  Before live trading:");
    println!("   1. Set ExecutionMode::Live in config");
    println!("   2. Configure exchange API credentials");
    println!("   3. Test with SMALL amounts first");
    println!("   4. Monitor closely and have stop-losses!\n");

    let _ = final_engine;

    Ok(())
}
