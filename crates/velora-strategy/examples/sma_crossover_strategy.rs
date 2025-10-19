//! SMA Crossover Strategy Example
//!
//! This example demonstrates how to build a complete trading strategy using
//! the Velora strategy framework.
//!
//! Strategy Logic:
//! - Buy when fast SMA crosses above slow SMA
//! - Sell when fast SMA crosses below slow SMA
//! - Position sizing: 10% of capital per trade
//!
//! Run with:
//! ```bash
//! cargo run --example sma_crossover_strategy
//! ```

use async_trait::async_trait;
use chrono::Utc;
use velora_core::types::{Candle, Symbol};
use velora_strategy::{
    indicators::{SingleIndicator, SMA},
    Indicator, MarketSnapshot, Position, PositionSide, Signal, Strategy, StrategyConfig,
    StrategyContext, StrategyResult, StrategyState,
};

/// Simple Moving Average Crossover Strategy
pub struct SmaCrossoverStrategy {
    /// Strategy configuration
    config: StrategyConfig,

    /// Current strategy state
    state: StrategyState,

    /// Fast SMA (10 periods)
    fast_sma: SMA,

    /// Slow SMA (50 periods)
    slow_sma: SMA,

    /// Previous fast SMA value (for crossover detection)
    prev_fast: Option<f64>,

    /// Previous slow SMA value (for crossover detection)
    prev_slow: Option<f64>,

    /// Symbol to trade
    symbol: String,
}

impl SmaCrossoverStrategy {
    /// Create a new SMA Crossover strategy
    pub fn new(
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
            fast_sma: SMA::new(fast_period)?,
            slow_sma: SMA::new(slow_period)?,
            prev_fast: None,
            prev_slow: None,
            symbol,
        })
    }

    /// Detect crossover
    fn detect_crossover(&self, current_fast: f64, current_slow: f64) -> Option<CrossoverType> {
        if let (Some(prev_fast), Some(prev_slow)) = (self.prev_fast, self.prev_slow) {
            // Golden Cross: fast crosses above slow (bullish)
            if prev_fast <= prev_slow && current_fast > current_slow {
                return Some(CrossoverType::GoldenCross);
            }

            // Death Cross: fast crosses below slow (bearish)
            if prev_fast >= prev_slow && current_fast < current_slow {
                return Some(CrossoverType::DeathCross);
            }
        }
        None
    }

    /// Calculate position size based on capital and percentage
    fn calculate_position_size(&self, price: f64, ctx: &StrategyContext) -> StrategyResult<f64> {
        let capital = ctx.available_capital()?;
        let position_value = capital * (self.config.max_position_size_pct / 100.0);
        Ok(position_value / price)
    }
}

#[derive(Debug, Clone, Copy)]
enum CrossoverType {
    GoldenCross, // Fast crosses above slow
    DeathCross,  // Fast crosses below slow
}

#[async_trait]
impl Strategy for SmaCrossoverStrategy {
    fn name(&self) -> &str {
        &self.config.name
    }

    fn version(&self) -> &str {
        "1.0.0"
    }

    fn config(&self) -> &StrategyConfig {
        &self.config
    }

    fn state(&self) -> StrategyState {
        self.state
    }

    async fn initialize(&mut self, ctx: &StrategyContext) -> StrategyResult<()> {
        println!("Initializing {} strategy...", self.name());
        println!("Symbol: {}", self.symbol);
        println!("Fast SMA: {} periods", self.fast_sma.period());
        println!("Slow SMA: {} periods", self.slow_sma.period());
        println!("Initial Capital: ${:.2}", ctx.available_capital()?);

        self.state = StrategyState::Running;
        Ok(())
    }

    async fn on_candle(
        &mut self,
        candle: &Candle,
        ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        let timestamp = candle.timestamp;
        let close_price = candle.close.into_inner();

        // Update indicators
        let fast_result = self.fast_sma.update(close_price, timestamp)?;
        let slow_result = self.slow_sma.update(close_price, timestamp)?;

        // Wait until both indicators are ready
        let (current_fast, current_slow) = match (fast_result, slow_result) {
            (Some(fast), Some(slow)) => (fast, slow),
            _ => {
                println!("[{timestamp}] Warming up indicators... (close: {close_price:.2})");
                return Ok(Signal::Hold);
            }
        };

        println!(
            "[{timestamp}] Price: {close_price:.2}, Fast SMA: {current_fast:.2}, Slow SMA: {current_slow:.2}"
        );

        // Detect crossover
        if let Some(crossover) = self.detect_crossover(current_fast, current_slow) {
            let has_position = ctx.has_position(&self.symbol)?;

            match crossover {
                CrossoverType::GoldenCross => {
                    if !has_position {
                        let quantity = self.calculate_position_size(close_price, ctx)?;
                        println!(
                            "🟢 GOLDEN CROSS DETECTED! Generating BUY signal (qty: {quantity:.4})"
                        );

                        // Store current values for next crossover detection
                        self.prev_fast = Some(current_fast);
                        self.prev_slow = Some(current_slow);

                        return Ok(Signal::buy(&self.symbol, quantity));
                    }
                }
                CrossoverType::DeathCross => {
                    if has_position {
                        println!("🔴 DEATH CROSS DETECTED! Generating SELL signal");

                        // Store current values for next crossover detection
                        self.prev_fast = Some(current_fast);
                        self.prev_slow = Some(current_slow);

                        return Ok(Signal::close(&self.symbol));
                    }
                }
            }
        }

        // Store current values for next crossover detection
        self.prev_fast = Some(current_fast);
        self.prev_slow = Some(current_slow);

        Ok(Signal::Hold)
    }

    async fn shutdown(&mut self, ctx: &StrategyContext) -> StrategyResult<()> {
        println!("\n=== Strategy Shutdown ===");
        println!("Final Capital: ${:.2}", ctx.available_capital()?);
        println!("Total Equity: ${:.2}", ctx.total_equity()?);
        println!("Unrealized P&L: ${:.2}", ctx.total_unrealized_pnl()?);

        // Close any open positions
        if ctx.has_position(&self.symbol)? {
            println!("Closing open position for {}", self.symbol);
        }

        self.state = StrategyState::Stopped;
        Ok(())
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
        self.fast_sma.reset();
        self.slow_sma.reset();
        self.prev_fast = None;
        self.prev_slow = None;
    }
}

#[tokio::main]
async fn main() -> StrategyResult<()> {
    println!("=== SMA Crossover Strategy Example ===\n");

    // Create strategy
    let mut strategy = SmaCrossoverStrategy::new("BTC-USD-PERP", 10, 50)?;

    // Create context
    let ctx = StrategyContext::new(10_000.0);

    // Initialize strategy
    strategy.initialize(&ctx).await?;

    // Simulate market data (simple price movements)
    println!("\n=== Simulating Market Data ===\n");

    let base_price = 50_000.0;
    let prices = generate_price_series(base_price, 100);

    for (i, price) in prices.iter().enumerate() {
        let timestamp = Utc::now();

        // Create a candle
        let candle = Candle {
            symbol: Symbol::new("BTC-USD-PERP"),
            timestamp,
            open: (*price).into(),
            high: (*price * 1.001).into(),
            low: (*price * 0.999).into(),
            close: (*price).into(),
            volume: 100.0.into(),
        };

        // Update market snapshot
        ctx.update_market_snapshot(
            "BTC-USD-PERP",
            MarketSnapshot {
                last_price: *price,
                timestamp,
                best_bid: Some(price - 0.5),
                best_ask: Some(price + 0.5),
                volume_24h: Some(1_000_000.0),
            },
        )?;

        // Process candle
        let signal = strategy.on_candle(&candle, &ctx).await?;

        // Execute signal (simulated)
        if signal.is_actionable() {
            match signal {
                Signal::Buy {
                    symbol, quantity, ..
                } => {
                    let position = Position::new(&symbol, PositionSide::Long, quantity, *price);
                    ctx.update_position(position)?;

                    let new_capital = ctx.available_capital()? - (quantity * price);
                    ctx.update_capital(new_capital)?;

                    println!("✅ Executed BUY: {quantity:.4} @ ${price:.2}");
                }
                Signal::Close { symbol, .. } => {
                    if let Some(mut position) = ctx.remove_position(&symbol)? {
                        position.update_price(*price);
                        let pnl = position.unrealized_pnl;

                        let new_capital = ctx.available_capital()? + position.value();
                        ctx.update_capital(new_capital)?;

                        println!("✅ Executed CLOSE: P&L = ${pnl:.2}");
                    }
                }
                _ => {}
            }
        }

        // Update position prices
        ctx.update_position_prices()?;
    }

    // Shutdown strategy
    strategy.shutdown(&ctx).await?;

    Ok(())
}

/// Generate a price series with a trend and some volatility
fn generate_price_series(base_price: f64, count: usize) -> Vec<f64> {
    let mut prices = Vec::with_capacity(count);
    let mut price = base_price;

    for i in 0..count {
        // Create a trend that changes direction
        let trend = if i < 30 {
            -50.0 // Downtrend
        } else if i < 60 {
            100.0 // Uptrend (should trigger golden cross)
        } else if i < 80 {
            20.0 // Slight uptrend
        } else {
            -80.0 // Downtrend (should trigger death cross)
        };

        // Add some random volatility
        let volatility = (i as f64 * 0.5).sin() * 30.0;

        price += trend + volatility;
        prices.push(price);
    }

    prices
}
