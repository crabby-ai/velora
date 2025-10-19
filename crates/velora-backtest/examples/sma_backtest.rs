//! SMA Crossover Strategy Backtest Example
//!
//! This example demonstrates how to backtest a trading strategy using the
//! velora-backtest engine with the SMA crossover strategy from velora-strategy.
//!
//! Run with:
//! ```bash
//! cargo run --example sma_backtest
//! ```

use async_trait::async_trait;
use chrono::Utc;
use velora_backtest::{BacktestConfig, Backtester, ExecutionConfig, FillModel};
use velora_core::types::{Candle, Symbol};
use velora_strategy::{
    indicators::{SingleIndicator, SMA},
    Indicator, Signal, Strategy, StrategyConfig, StrategyContext, StrategyResult, StrategyState,
};

/// Simple Moving Average Crossover Strategy
struct SmaCrossoverStrategy {
    config: StrategyConfig,
    state: StrategyState,
    fast_sma: SMA,
    slow_sma: SMA,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
    symbol: String,
}

impl SmaCrossoverStrategy {
    fn new(
        symbol: impl Into<String>,
        fast_period: usize,
        slow_period: usize,
    ) -> StrategyResult<Self> {
        let symbol = symbol.into();
        let config = StrategyConfig::new("SMA Crossover Backtest")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0)
            .with_max_position_size(10.0);

        Ok(Self {
            config,
            state: StrategyState::Initializing,
            fast_sma: SMA::new(fast_period)?,
            slow_sma: SMA::new(slow_period)?,
            prev_fast: None,
            prev_slow: None,
            symbol,
        })
    }

    fn detect_crossover(&self, current_fast: f64, current_slow: f64) -> Option<CrossoverType> {
        if let (Some(prev_fast), Some(prev_slow)) = (self.prev_fast, self.prev_slow) {
            if prev_fast <= prev_slow && current_fast > current_slow {
                return Some(CrossoverType::GoldenCross);
            }
            if prev_fast >= prev_slow && current_fast < current_slow {
                return Some(CrossoverType::DeathCross);
            }
        }
        None
    }

    fn calculate_position_size(&self, price: f64, ctx: &StrategyContext) -> StrategyResult<f64> {
        let capital = ctx.available_capital()?;
        let position_value = capital * (self.config.max_position_size_pct / 100.0);
        Ok(position_value / price)
    }
}

#[derive(Debug, Clone, Copy)]
enum CrossoverType {
    GoldenCross,
    DeathCross,
}

#[async_trait]
impl Strategy for SmaCrossoverStrategy {
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
        let close_price = candle.close.into_inner();
        let timestamp = candle.timestamp;

        let fast_result = self.fast_sma.update(close_price, timestamp)?;
        let slow_result = self.slow_sma.update(close_price, timestamp)?;

        let (current_fast, current_slow) = match (fast_result, slow_result) {
            (Some(fast), Some(slow)) => (fast, slow),
            _ => return Ok(Signal::Hold),
        };

        if let Some(crossover) = self.detect_crossover(current_fast, current_slow) {
            let has_position = ctx.has_position(&self.symbol)?;

            match crossover {
                CrossoverType::GoldenCross => {
                    if !has_position {
                        let quantity = self.calculate_position_size(close_price, ctx)?;
                        self.prev_fast = Some(current_fast);
                        self.prev_slow = Some(current_slow);
                        return Ok(Signal::buy(&self.symbol, quantity));
                    }
                }
                CrossoverType::DeathCross => {
                    if has_position {
                        self.prev_fast = Some(current_fast);
                        self.prev_slow = Some(current_slow);
                        return Ok(Signal::close(&self.symbol));
                    }
                }
            }
        }

        self.prev_fast = Some(current_fast);
        self.prev_slow = Some(current_slow);

        Ok(Signal::Hold)
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
        self.fast_sma.reset();
        self.slow_sma.reset();
        self.prev_fast = None;
        self.prev_slow = None;
    }
}

/// Generate synthetic price data for backtesting
fn generate_test_data(count: usize) -> Vec<Candle> {
    let mut candles = Vec::with_capacity(count);
    let base_price = 50_000.0;
    let mut price = base_price;
    let start_time = Utc::now() - chrono::Duration::days(count as i64);

    for i in 0..count {
        // Create market phases
        let trend = if i < 150 {
            -30.0 // Downtrend
        } else if i < 400 {
            80.0 // Strong uptrend (golden cross)
        } else if i < 550 {
            15.0 // Consolidation
        } else {
            -60.0 // Downtrend (death cross)
        };

        let volatility = (i as f64 * 0.3).sin() * 25.0;
        price += trend + volatility;

        let timestamp = start_time + chrono::Duration::hours(i as i64);

        candles.push(Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp,
            open: price.into(),
            high: (price * 1.002).into(),
            low: (price * 0.998).into(),
            close: price.into(),
            volume: (100.0 + (i as f64 * 0.5).sin() * 20.0).into(),
        });
    }

    candles
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== SMA Crossover Strategy Backtest ===\n");

    // Create strategy
    let strategy = SmaCrossoverStrategy::new("BTC-USD-PERP", 10, 50)?;

    // Configure backtest
    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_symbols(vec!["BTC-USD-PERP".to_string()])
        .with_execution(ExecutionConfig {
            commission_rate: 0.001, // 0.1% per trade
            slippage_bps: 5.0,      // 5 basis points slippage
            fill_delay_ms: 0,
            fill_model: FillModel::Realistic,
        });

    // Generate test data
    println!("Generating synthetic market data...");
    let candles = generate_test_data(700);
    println!("Generated {} hourly candles", candles.len());
    println!(
        "Period: {} to {}\n",
        candles.first().unwrap().timestamp.format("%Y-%m-%d"),
        candles.last().unwrap().timestamp.format("%Y-%m-%d")
    );

    // Run backtest
    println!("Starting backtest...\n");
    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await?;

    // Print results
    report.print_summary();

    // Print trade details
    if !report.trades.is_empty() {
        println!("\n=== Trade Details ===\n");
        for (i, trade) in report.trades.iter().enumerate() {
            println!("Trade #{}", i + 1);
            println!("  Symbol:     {}", trade.symbol);
            println!("  Side:       {:?}", trade.side);
            println!(
                "  Entry:      ${:.2} @ {}",
                trade.entry_price,
                trade.entry_time.format("%Y-%m-%d %H:%M")
            );
            println!(
                "  Exit:       ${:.2} @ {}",
                trade.exit_price,
                trade.exit_time.format("%Y-%m-%d %H:%M")
            );
            println!("  Quantity:   {:.4}", trade.quantity);
            println!("  P&L:        ${:.2} ({:.2}%)", trade.pnl, trade.pnl_pct);
            println!("  Commission: ${:.2}", trade.commission);
            println!();
        }
    }

    // Export results
    println!("=== Exporting Results ===\n");

    let equity_json = report.export_equity_curve_json()?;
    println!("Equity curve has {} data points", report.equity_curve.len());

    let trades_json = report.export_trades_json()?;
    println!("Exported {} trades", report.trades.len());

    // Save to files (optional)
    std::fs::write("backtest_equity.json", equity_json)?;
    std::fs::write("backtest_trades.json", trades_json)?;
    std::fs::write("backtest_report.json", report.to_json()?)?;

    println!("\nResults saved to:");
    println!("  - backtest_equity.json");
    println!("  - backtest_trades.json");
    println!("  - backtest_report.json");

    Ok(())
}
