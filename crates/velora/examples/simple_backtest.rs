//! Simple Backtest Example
//!
//! This example demonstrates how to run a basic backtest using the velora umbrella crate.
//! It shows the simplest possible workflow: create a strategy, generate data, and run a backtest.
//!
//! Run with:
//! ```bash
//! cargo run --example simple_backtest
//! ```

use async_trait::async_trait;
use velora::prelude::*;

/// A simple momentum strategy that buys on strong upward momentum
struct SimpleMomentumStrategy {
    config: StrategyConfig,
    state: StrategyState,
    symbol: String,
}

impl SimpleMomentumStrategy {
    fn new(symbol: impl Into<String>) -> StrategyResult<Self> {
        let symbol = symbol.into();
        let config = StrategyConfig::new("Simple Momentum")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0)
            .with_max_position_size(10.0);

        Ok(Self {
            config,
            state: StrategyState::Initializing,
            symbol,
        })
    }
}

#[async_trait]
impl Strategy for SimpleMomentumStrategy {
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
        Ok(())
    }

    async fn on_candle(
        &mut self,
        candle: &Candle,
        ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        let has_position = ctx.has_position(&self.symbol)?;

        // Simple strategy: buy if price increased more than 2% from open
        let price_change = (candle.close.into_inner() - candle.open.into_inner())
            / candle.open.into_inner();

        if !has_position && price_change > 0.02 {
            // Buy signal
            let capital = ctx.available_capital()?;
            let position_value = capital * 0.5; // Use 50% of capital
            let quantity = position_value / candle.close.into_inner();

            return Ok(Signal::buy(&self.symbol, quantity));
        } else if has_position && price_change < -0.015 {
            // Sell signal if price drops more than 1.5%
            return Ok(Signal::close(&self.symbol));
        }

        Ok(Signal::Hold)
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
    }
}

/// Generate simple test data with trending behavior
fn generate_test_data(count: usize) -> Vec<Candle> {
    use chrono::Utc;

    let mut candles = Vec::with_capacity(count);
    let mut price = 100.0;
    let start_time = Utc::now() - chrono::Duration::days(count as i64);

    for i in 0..count {
        let trend = (i as f64 * 0.02).sin() * 0.5;
        let volatility = (i as f64 * 0.1).cos() * 0.3;

        let open = price;
        price *= 1.0 + trend + volatility;
        let close = price;
        let high = price.max(open) * 1.005;
        let low = price.min(open) * 0.995;

        candles.push(Candle {
            symbol: Symbol::new("TEST-PERP"),
            timestamp: start_time + chrono::Duration::hours(i as i64),
            open: open.into(),
            high: high.into(),
            low: low.into(),
            close: close.into(),
            volume: 1000.0.into(),
        });
    }

    candles
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    #[cfg(feature = "utils")]
    velora::utils::init_tracing("info")?;

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║     Velora - Simple Backtest Example             ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("Platform: {}\n", velora::version_string());

    // 1. Create strategy
    println!("📊 Creating strategy...");
    let strategy = SimpleMomentumStrategy::new("TEST-PERP")?;
    println!("   ✓ Strategy: {}", strategy.name());

    // 2. Generate test data
    println!("\n📈 Generating test data...");
    let candles = generate_test_data(100);
    println!("   ✓ Generated {} candles", candles.len());

    // 3. Configure backtest
    println!("\n⚙️  Configuring backtest...");
    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_symbols(vec!["TEST-PERP".to_string()])
        .with_execution(ExecutionConfig {
            commission_rate: 0.001, // 0.1% per trade
            slippage_bps: 5.0,
            fill_delay_ms: 0,
            fill_model: FillModel::Realistic,
        });
    println!("   ✓ Initial capital: ${:,.2}", 10_000.0);
    println!("   ✓ Commission rate: {}%", 0.1);

    // 4. Run backtest
    println!("\n🚀 Running backtest...\n");
    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await?;

    // 5. Display results
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║              Backtest Results                     ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    report.print_summary();

    println!("\n✅ Backtest complete!");

    Ok(())
}
