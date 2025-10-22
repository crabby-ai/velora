//! End-to-End Strategy Development Example
//!
//! This example demonstrates the complete workflow for developing, testing, and deploying
//! a trading strategy using Velora:
//!
//! 1. Create a strategy using technical indicators
//! 2. Backtest the strategy with historical data
//! 3. Analyze backtest results
//! 4. (Optionally) Deploy to live trading in dry-run mode
//!
//! Run with:
//! ```bash
//! cargo run --example strategy_development
//! ```

use async_trait::async_trait;
use velora::prelude::*;

/// SMA Crossover Strategy with Risk Management
///
/// This strategy:
/// - Buys when fast SMA crosses above slow SMA (golden cross)
/// - Sells when fast SMA crosses below slow SMA (death cross)
/// - Uses position sizing based on available capital
/// - Implements basic risk management
struct SmaCrossoverStrategy {
    config: StrategyConfig,
    state: StrategyState,
    symbol: String,
    fast_period: usize,
    slow_period: usize,
    fast_sma: SMA,
    slow_sma: SMA,
    prev_fast: Option<f64>,
    prev_slow: Option<f64>,
}

impl SmaCrossoverStrategy {
    fn new(
        symbol: impl Into<String>,
        fast_period: usize,
        slow_period: usize,
    ) -> StrategyResult<Self> {
        let symbol = symbol.into();
        let config = StrategyConfig::new("SMA Crossover")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0)
            .with_max_position_size(10.0);

        Ok(Self {
            config,
            state: StrategyState::Initializing,
            symbol,
            fast_period,
            slow_period,
            fast_sma: SMA::new(fast_period)?,
            slow_sma: SMA::new(slow_period)?,
            prev_fast: None,
            prev_slow: None,
        })
    }

    fn detect_crossover(&self, fast: f64, slow: f64) -> Option<CrossoverType> {
        if let (Some(prev_fast), Some(prev_slow)) = (self.prev_fast, self.prev_slow) {
            // Golden cross: fast crosses above slow
            if prev_fast <= prev_slow && fast > slow {
                return Some(CrossoverType::GoldenCross);
            }
            // Death cross: fast crosses below slow
            if prev_fast >= prev_slow && fast < slow {
                return Some(CrossoverType::DeathCross);
            }
        }
        None
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
        let price = candle.close.into_inner();

        // Update indicators
        let fast_result = self.fast_sma.update(price, candle.timestamp)?;
        let slow_result = self.slow_sma.update(price, candle.timestamp)?;

        // Wait until both indicators are ready
        let (fast, slow) = match (fast_result, slow_result) {
            (Some(f), Some(s)) => (f, s),
            _ => return Ok(Signal::Hold),
        };

        // Detect crossover
        if let Some(crossover) = self.detect_crossover(fast, slow) {
            let has_position = ctx.has_position(&self.symbol)?;

            match crossover {
                CrossoverType::GoldenCross if !has_position => {
                    // Calculate position size (use 50% of available capital)
                    let capital = ctx.available_capital()?;
                    let position_value = capital * 0.5;
                    let quantity = position_value / price;

                    self.prev_fast = Some(fast);
                    self.prev_slow = Some(slow);

                    return Ok(Signal::buy(&self.symbol, quantity));
                }
                CrossoverType::DeathCross if has_position => {
                    self.prev_fast = Some(fast);
                    self.prev_slow = Some(slow);

                    return Ok(Signal::close(&self.symbol));
                }
                _ => {}
            }
        }

        // Update previous values
        self.prev_fast = Some(fast);
        self.prev_slow = Some(slow);

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

/// Generate synthetic market data for testing
fn generate_market_data(count: usize) -> Vec<Candle> {
    use chrono::Utc;

    let mut candles = Vec::with_capacity(count);
    let mut price = 50_000.0;
    let start_time = Utc::now() - chrono::Duration::hours(count as i64);

    for i in 0..count {
        // Create different market phases
        let trend = if i < 100 {
            -20.0 // Downtrend
        } else if i < 300 {
            60.0 // Strong uptrend (should trigger golden cross)
        } else if i < 400 {
            10.0 // Consolidation
        } else {
            -40.0 // Downtrend (should trigger death cross)
        };

        let volatility = (i as f64 * 0.2).sin() * 30.0;
        price += trend + volatility;

        candles.push(Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp: start_time + chrono::Duration::hours(i as i64),
            open: (price - 10.0).into(),
            high: (price + 50.0).into(),
            low: (price - 50.0).into(),
            close: price.into(),
            volume: (100.0 + (i as f64 * 0.3).sin() * 20.0).into(),
        });
    }

    candles
}

async fn run_backtest(
    strategy: SmaCrossoverStrategy,
    candles: Vec<Candle>,
) -> Result<BacktestReport, Box<dyn std::error::Error>> {
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║          Step 2: Run Backtest                     ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_symbols(vec!["BTC-USD-PERP".to_string()])
        .with_execution(ExecutionConfig {
            commission_rate: 0.001,
            slippage_bps: 5.0,
            fill_delay_ms: 0,
            fill_model: FillModel::Realistic,
        });

    println!("⚙️  Backtest configuration:");
    println!("   Initial capital: ${:,.2}", 10_000.0);
    println!("   Commission: 0.1%");
    println!("   Slippage: 5 bps");
    println!("   Candles: {}", candles.len());

    println!("\n🚀 Running backtest...\n");

    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await?;

    Ok(report)
}

fn analyze_results(report: &BacktestReport) {
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║          Step 3: Analyze Results                  ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    report.print_summary();

    // Detailed trade analysis
    if !report.trades.is_empty() {
        println!("\n📊 Trade Analysis:");
        println!("   Total trades: {}", report.trades.len());

        let winning_trades = report.trades.iter().filter(|t| t.pnl > 0.0).count();
        let losing_trades = report.trades.iter().filter(|t| t.pnl <= 0.0).count();

        println!("   Winning trades: {}", winning_trades);
        println!("   Losing trades: {}", losing_trades);

        if !report.trades.is_empty() {
            let avg_win = report
                .trades
                .iter()
                .filter(|t| t.pnl > 0.0)
                .map(|t| t.pnl)
                .sum::<f64>()
                / winning_trades.max(1) as f64;

            let avg_loss = report
                .trades
                .iter()
                .filter(|t| t.pnl <= 0.0)
                .map(|t| t.pnl)
                .sum::<f64>()
                / losing_trades.max(1) as f64;

            println!("   Average win: ${:.2}", avg_win);
            println!("   Average loss: ${:.2}", avg_loss);
        }

        // Show sample trades
        println!("\n   Sample trades:");
        for (i, trade) in report.trades.iter().take(3).enumerate() {
            println!(
                "   Trade #{}: {} @ ${:.2} -> ${:.2} | P&L: ${:.2} ({:.2}%)",
                i + 1,
                trade.symbol,
                trade.entry_price,
                trade.exit_price,
                trade.pnl,
                trade.pnl_pct
            );
        }
    }
}

fn provide_recommendations(report: &BacktestReport) {
    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║          Step 4: Recommendations                  ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    let metrics = &report.metrics;

    println!("📋 Strategy Assessment:\n");

    // Assess profitability
    if metrics.total_return > 0.0 {
        println!("   ✓ Profitable strategy ({:.2}% return)", metrics.total_return * 100.0);
    } else {
        println!("   ✗ Unprofitable strategy ({:.2}% return)", metrics.total_return * 100.0);
        println!("     → Consider adjusting parameters or trying different indicators");
    }

    // Assess risk-adjusted returns
    if metrics.sharpe_ratio > 1.0 {
        println!("   ✓ Good risk-adjusted returns (Sharpe: {:.2})", metrics.sharpe_ratio);
    } else {
        println!("   ⚠ Low risk-adjusted returns (Sharpe: {:.2})", metrics.sharpe_ratio);
        println!("     → Consider improving signal quality or risk management");
    }

    // Assess drawdown
    if metrics.max_drawdown < 0.15 {
        println!("   ✓ Acceptable drawdown ({:.2}%)", metrics.max_drawdown * 100.0);
    } else {
        println!("   ⚠ High drawdown ({:.2}%)", metrics.max_drawdown * 100.0);
        println!("     → Consider adding stop-losses or reducing position sizes");
    }

    // Overall recommendation
    println!("\n💡 Next Steps:\n");

    if metrics.total_return > 0.0 && metrics.sharpe_ratio > 1.0 && metrics.max_drawdown < 0.2 {
        println!("   This strategy shows promise! Consider:");
        println!("   1. Testing with different parameters (fast/slow periods)");
        println!("   2. Running on different time periods and market conditions");
        println!("   3. Adding additional filters or confirmations");
        println!("   4. Testing in dry-run mode with live data");
    } else {
        println!("   This strategy needs improvement. Try:");
        println!("   1. Adjusting SMA periods (current: fast=10, slow=50)");
        println!("   2. Adding trend filters or volume confirmation");
        println!("   3. Implementing better position sizing");
        println!("   4. Testing on different timeframes");
    }

    println!("\n   ⚠️  Remember: Past performance does not guarantee future results!");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    #[cfg(feature = "utils")]
    velora::utils::init_tracing("warn")?;

    println!("╔═══════════════════════════════════════════════════╗");
    println!("║   Velora - End-to-End Strategy Development       ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("Platform: {}\n", velora::version_string());

    // STEP 1: Create Strategy
    println!("╔═══════════════════════════════════════════════════╗");
    println!("║          Step 1: Create Strategy                  ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    let fast_period = 10;
    let slow_period = 50;

    println!("📊 Strategy: SMA Crossover");
    println!("   Fast SMA: {} periods", fast_period);
    println!("   Slow SMA: {} periods", slow_period);
    println!("   Symbol: BTC-USD-PERP");

    let strategy = SmaCrossoverStrategy::new("BTC-USD-PERP", fast_period, slow_period)?;
    println!("\n   ✓ Strategy created");

    // Generate test data
    println!("\n📈 Generating historical data...");
    let candles = generate_market_data(500);
    println!(
        "   ✓ Generated {} hourly candles",
        candles.len()
    );

    // STEP 2: Run Backtest
    let report = run_backtest(strategy, candles).await?;
    println!("   ✓ Backtest complete");

    // STEP 3: Analyze Results
    analyze_results(&report);

    // STEP 4: Provide Recommendations
    provide_recommendations(&report);

    println!("\n╔═══════════════════════════════════════════════════╗");
    println!("║          Strategy Development Complete            ║");
    println!("╚═══════════════════════════════════════════════════╝\n");

    println!("📁 You can now:");
    println!("   1. Run more backtests with different parameters");
    println!("   2. Test on different symbols and timeframes");
    println!("   3. Deploy to dry-run mode (see examples/live_trading.rs)");
    println!("   4. Eventually deploy to live trading (with caution!)\n");

    Ok(())
}
