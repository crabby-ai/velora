//! Integration tests for the Velora platform
//!
//! These tests verify that all components work together correctly.

use async_trait::async_trait;
use velora::prelude::*;

/// Simple test strategy for integration testing
struct TestStrategy {
    config: StrategyConfig,
    state: StrategyState,
    symbol: String,
}

impl TestStrategy {
    fn new(symbol: impl Into<String>) -> StrategyResult<Self> {
        let symbol = symbol.into();
        let config = StrategyConfig::new("Test Strategy")
            .with_symbols(vec![symbol.clone()])
            .with_capital(10_000.0);

        Ok(Self {
            config,
            state: StrategyState::Initializing,
            symbol,
        })
    }
}

#[async_trait]
impl Strategy for TestStrategy {
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
        _candle: &Candle,
        _ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        Ok(Signal::Hold)
    }

    fn reset(&mut self) {
        self.state = StrategyState::Initializing;
    }
}

/// Generate simple test candles
fn generate_test_candles(count: usize, symbol: &str) -> Vec<Candle> {
    use chrono::Utc;

    let mut candles = Vec::with_capacity(count);
    let start_time = Utc::now() - chrono::Duration::hours(count as i64);

    for i in 0..count {
        let price = 100.0 + i as f64;
        candles.push(Candle {
            symbol: Symbol::new(symbol),
            timestamp: start_time + chrono::Duration::hours(i as i64),
            open: price.into(),
            high: (price + 1.0).into(),
            low: (price - 1.0).into(),
            close: price.into(),
            volume: 100.0.into(),
        });
    }

    candles
}

#[tokio::test]
async fn test_core_types() {
    // Test Symbol creation
    let symbol = Symbol::new("BTC-USD");
    assert_eq!(symbol.as_str(), "BTC-USD");

    // Test OrderedFloat
    let price: OrderedFloat<f64> = 100.5.into();
    assert_eq!(price.into_inner(), 100.5);

    // Test Candle creation
    let candle = Candle {
        symbol: Symbol::new("TEST"),
        timestamp: chrono::Utc::now(),
        open: 100.0.into(),
        high: 101.0.into(),
        low: 99.0.into(),
        close: 100.5.into(),
        volume: 1000.0.into(),
    };

    assert_eq!(candle.symbol.as_str(), "TEST");
    assert!(candle.high >= candle.low);
}

#[tokio::test]
async fn test_strategy_creation() {
    let strategy = TestStrategy::new("BTC-USD").unwrap();
    assert_eq!(strategy.name(), "Test Strategy");
    assert_eq!(strategy.state(), StrategyState::Initializing);
    assert_eq!(strategy.config().initial_capital, 10_000.0);
}

#[cfg(feature = "backtest")]
#[tokio::test]
async fn test_backtest_integration() {
    // Create strategy
    let strategy = TestStrategy::new("TEST-PERP").unwrap();

    // Generate test data
    let candles = generate_test_candles(100, "TEST-PERP");
    assert_eq!(candles.len(), 100);

    // Configure backtest
    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_symbols(vec!["TEST-PERP".to_string()])
        .with_execution(ExecutionConfig {
            commission_rate: 0.001,
            slippage_bps: 5.0,
            fill_delay_ms: 0,
            fill_model: FillModel::Realistic,
        });

    // Run backtest
    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await
        .unwrap();

    // Verify report
    assert!(report.metrics.total_return.is_finite());
    assert!(report.equity_curve.len() > 0);
}

#[cfg(all(feature = "ta", feature = "strategy"))]
#[tokio::test]
async fn test_technical_indicators() {
    use velora::ta::trend::sma;

    // Test SMA calculation
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0];
    let sma_values = sma(&prices, 3).unwrap();

    // Should have 3 values (5 prices - 3 period + 1)
    assert_eq!(sma_values.len(), 3);

    // Verify first SMA value: (100 + 102 + 101) / 3 = 101.0
    assert!((sma_values[0] - 101.0).abs() < 0.01);
}

#[cfg(all(feature = "ta", feature = "strategy"))]
#[tokio::test]
async fn test_indicator_integration() {
    use chrono::Utc;

    // Create SMA indicator
    let mut sma = SMA::new(5).unwrap();

    // Feed it prices
    let base_time = Utc::now();
    let prices = vec![100.0, 102.0, 101.0, 103.0, 105.0, 104.0];

    let mut results = Vec::new();
    for (i, &price) in prices.iter().enumerate() {
        let timestamp = base_time + chrono::Duration::hours(i as i64);
        if let Some(value) = sma.update(price, timestamp).unwrap() {
            results.push(value);
        }
    }

    // Should have 2 values (6 prices - 5 period + 1)
    assert_eq!(results.len(), 2);
    assert!(results[0] > 0.0);
    assert!(results[1] > 0.0);
}

#[test]
fn test_version_info() {
    // Verify version string is correct
    let version = velora::version_string();
    assert!(version.contains("Velora"));
    assert!(version.contains("v"));

    // Verify constants
    assert_eq!(velora::PLATFORM_NAME, "Velora");
    assert!(!velora::VERSION.is_empty());
}

#[cfg(feature = "utils")]
#[test]
fn test_utils_config() {
    use velora::utils::BacktestConfig as UtilBacktestConfig;

    // Test default config
    let config = UtilBacktestConfig::default();
    assert_eq!(config.initial_capital, 10_000.0);
    assert_eq!(config.commission_rate, 0.001);
    assert_eq!(config.slippage_bps, 5.0);
}

#[cfg(all(feature = "engine", feature = "utils"))]
#[test]
fn test_live_trading_config() {
    use velora::utils::LiveTradingConfig;

    // Test default config
    let config = LiveTradingConfig::default();
    assert_eq!(config.heartbeat_interval, 30);
    assert!(config.health_checks);
}

#[cfg(feature = "backtest")]
#[tokio::test]
async fn test_full_workflow() {
    // This test demonstrates the complete workflow:
    // 1. Create strategy
    // 2. Generate data
    // 3. Run backtest
    // 4. Analyze results

    // Step 1: Create strategy
    let strategy = TestStrategy::new("BTC-USD").unwrap();
    assert_eq!(strategy.name(), "Test Strategy");

    // Step 2: Generate data
    let candles = generate_test_candles(50, "BTC-USD");
    assert_eq!(candles.len(), 50);

    // Step 3: Run backtest
    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_symbols(vec!["BTC-USD".to_string()])
        .with_execution(ExecutionConfig::realistic());

    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await
        .unwrap();

    // Step 4: Verify results
    assert!(report.metrics.total_return.is_finite());
    assert!(report.metrics.sharpe_ratio.is_finite());
    assert!(report.metrics.max_drawdown >= 0.0);
    assert!(report.equity_curve.len() > 0);

    // Verify equity curve integrity
    for snapshot in &report.equity_curve {
        assert!(snapshot.total_equity >= 0.0);
        assert!(snapshot.cash >= 0.0);
    }
}
