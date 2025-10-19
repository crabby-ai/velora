# Velora

> A modular, high-performance trading platform for cryptocurrency markets built in Rust

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
![Status](https://img.shields.io/badge/status-active_development-green.svg)

## What is Velora?

Velora is a full-featured algorithmic trading platform designed for cryptocurrency markets. Built with Rust for maximum performance and safety, it provides everything you need to build, test, and deploy trading strategies.

**Key Philosophy**: Modular architecture where each component is independently useful, testable, and production-ready.

## Project Structure

This is a Cargo workspace containing multiple specialized crates:

```
velora/
├── crates/
│   ├── velora-core/        ✅ Core types, errors, config
│   ├── velora-data/        ✅ Market data handling & storage
│   ├── velora-ta/          ✅ Technical analysis indicators
│   ├── velora-exchange/    🚧 Exchange integrations (Lighter, Paradex)
│   ├── velora-strategy/    ✅ Strategy framework & context
│   ├── velora-backtest/    ✅ Backtesting engine (fully tested)
│   └── velora-engine/      ✅ Live trading engine (dry-run ready)
├── Cargo.toml              # Workspace configuration
└── docs/                   # Documentation & design docs
```

**Legend**: ✅ Complete | 🚧 In Progress | 📋 Planned

## Features

### 🎯 Core Features

- **Type-Safe Trading**: Leverage Rust's type system to prevent common trading errors
- **Async-First**: Built on Tokio for high-performance concurrent operations
- **Modular Design**: Use only what you need, publish components independently
- **Event-Driven Architecture**: Real-time event processing with sub-millisecond latency

### 📊 Data Layer (velora-data)

- ✅ Multiple data source support (CSV, Parquet)
- ✅ Efficient candle data structures with ordered floats
- ✅ Symbol and timeframe management
- ✅ High-performance data loading with polars

### 🔧 Strategy Framework (velora-strategy)

- ✅ Flexible `Strategy` trait with lifecycle hooks
- ✅ `StrategyContext` for managing market data and positions
- ✅ Signal generation (Buy, Sell, Hold, Close, Modify)
- ✅ Built-in position and order tracking
- ✅ Metadata support for custom strategy data

### 🧪 Backtesting Engine (velora-backtest)

**Fully Implemented and Tested**:

- ✅ Historical data replay with realistic order matching
- ✅ Multiple order types (Market, Limit, Stop, Stop-Limit)
- ✅ Commission and slippage simulation
- ✅ Comprehensive performance metrics:
  - Total return, Sharpe ratio, Sortino ratio
  - Maximum drawdown, win rate, profit factor
  - Average trade duration, average win/loss
- ✅ Equity curve tracking and visualization data
- ✅ Trade history export
- ✅ Example strategies included

**Example**:

```rust
use velora_backtest::{BacktestEngine, BacktestConfig};
use velora_strategy::StrategyConfig;

let config = BacktestConfig::builder()
    .initial_capital(10_000.0)
    .commission_rate(0.001)
    .slippage_rate(0.0005)
    .build();

let mut engine = BacktestEngine::new(config)
    .with_strategy(Box::new(my_strategy));

let results = engine.run(candles).await?;
println!("Total Return: {:.2}%", results.total_return_pct);
println!("Sharpe Ratio: {:.2}", results.sharpe_ratio);
```

### 🚀 Live Trading Engine (velora-engine)

**Production-Ready Dry-Run Mode**:

- ✅ Event-driven architecture with async processing
- ✅ Order management system with rate limiting
- ✅ Real-time position tracking and P&L calculation
- ✅ Equity curve snapshots for monitoring
- ✅ Dry-run mode for risk-free strategy testing
- ✅ Dual execution modes (Live/DryRun)
- 🚧 Exchange integration (in progress)

**Features**:

- Sub-millisecond event processing
- Automatic order lifecycle management
- Real-time portfolio valuation
- Heartbeat monitoring and health checks
- Graceful shutdown handling

**Example**:

```rust
use velora_engine::{TradingEngine, EngineConfig, ExecutionMode};

let config = EngineConfig::builder()
    .mode(ExecutionMode::DryRun)
    .initial_capital(10_000.0)
    .max_orders_per_second(5)
    .build();

let mut engine = TradingEngine::new(config)
    .with_strategy(Box::new(my_strategy));

// In dry-run mode - no real money at risk
engine.start_with_receiver(market_rx).await?;
```

### 🔌 Exchange Integration (velora-exchange)

**In Development**:

- 🚧 Lighter.xyz (Arbitrum-based perpetuals)
- 🚧 Paradex (Starknet-based perpetuals)
- Unified REST and WebSocket interfaces
- EVM and Starknet wallet authentication
- Built-in rate limiting and error handling

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/itsparser/velora.git
cd velora

# Build the workspace
cargo build --workspace --release

# Run tests
cargo test --workspace

# Run backtest example
cargo run --example sma_crossover --release
```

### Running Examples

**Backtesting Example**:

```bash
# Run SMA crossover strategy backtest
cd crates/velora-backtest
cargo run --example sma_crossover --release
```

**Live Trading (Dry-Run) Example**:

```bash
# Run dry-run trading simulation
cd crates/velora-engine
cargo run --example dry_run_trading --release
```

### Using as a Dependency

```toml
[dependencies]
# Core components
velora-core = "0.1"
velora-data = "0.1"
velora-ta = "0.1"

# Strategy and execution
velora-strategy = "0.1"
velora-backtest = "0.1"
velora-engine = "0.1"

# Exchange integration (when ready)
# velora-exchange = "0.1"
```

### Design Principles

1. **Modularity**: Each crate is independently useful and publishable
2. **Performance**: Designed for high-frequency trading scenarios
3. **Safety**: Rust's type system prevents common trading bugs
4. **Testability**: Comprehensive test coverage with examples
5. **Simplicity**: Clear APIs and straightforward usage

## Performance

**Achieved Metrics** (on M1 Mac):

- Event processing: < 1ms latency
- Backtest throughput: 10,000+ candles/second
- Memory footprint: ~50MB for typical strategy
- Indicator calculations: Sub-microsecond for most indicators

## Examples

### Simple SMA Crossover Strategy

```rust
use async_trait::async_trait;
use velora_core::Candle;
use velora_strategy::{Strategy, StrategyContext, Signal};
use velora_ta::trend::SMA;

struct SmaCrossover {
    fast_sma: SMA,
    slow_sma: SMA,
    last_signal: Option<bool>, // true = bullish crossover
}

#[async_trait]
impl Strategy for SmaCrossover {
    async fn on_candle(
        &mut self,
        candle: &Candle,
        ctx: &StrategyContext,
    ) -> Result<Signal> {
        let fast = self.fast_sma.update(candle.close)?;
        let slow = self.slow_sma.update(candle.close)?;

        if let (Some(fast), Some(slow)) = (fast, slow) {
            let is_bullish = fast > slow;

            if self.last_signal != Some(is_bullish) {
                self.last_signal = Some(is_bullish);

                return Ok(if is_bullish {
                    Signal::Buy {
                        symbol: candle.symbol.to_string(),
                        quantity: 0.1,
                        limit_price: None,
                        stop_price: None,
                        metadata: HashMap::new(),
                    }
                } else {
                    Signal::Sell {
                        symbol: candle.symbol.to_string(),
                        quantity: 0.1,
                        limit_price: None,
                        stop_price: None,
                        metadata: HashMap::new(),
                    }
                });
            }
        }

        Ok(Signal::Hold)
    }
}
```

## Documentation

- **[High-Level Design](docs/LIVE_TRADING_ENGINE_HLD.md)** - Live trading engine architecture
- **Examples** - See `examples/` directory in each crate
- **API Docs** - Run `cargo doc --workspace --no-deps --open`

Each crate has comprehensive documentation:

- [velora-core](crates/velora-core/README.md)
- [velora-data](crates/velora-data/README.md)
- [velora-ta](crates/velora-ta/README.md)
- [velora-strategy](crates/velora-strategy/README.md)
- [velora-backtest](crates/velora-backtest/README.md)
- [velora-engine](crates/velora-engine/README.md)
- [velora-exchange](crates/velora-exchange/README.md)

## Contributing

We welcome contributions! Here's how you can help:

1. **Check Issues**: Look for `good-first-issue` labels
2. **Pick a Component**: Each crate can be developed independently
3. **Write Tests**: Maintain high test coverage (aim for >80%)
4. **Add Examples**: Show how to use new features
5. **Update Docs**: Keep READMEs and API docs current

## Technology Stack

- **Language**: Rust 2021 edition (1.70+)
- **Async Runtime**: Tokio
- **Data Processing**: Polars, Arrow
- **Serialization**: Serde (JSON, CSV, Parquet)
- **Time**: Chrono
- **Numerics**: OrderedFloat for price precision
- **Logging**: Tracing + tracing-subscriber
- **Errors**: Thiserror
- **Testing**: Criterion for benchmarks

## Safety Disclaimer

**⚠️ CRITICAL: Trading involves substantial risk of loss.**

This software is provided "as is" without warranty of any kind. The developers are not responsible for any financial losses incurred through use of this software.

**Best Practices**:

- ✅ Always backtest strategies thoroughly
- ✅ Use dry-run mode before live trading
- ✅ Start with minimal position sizes
- ✅ Implement proper risk management
- ✅ Never invest more than you can afford to lose
- ✅ Monitor your strategies constantly
- ✅ Have kill switches and circuit breakers

**This software is NOT financial advice.**

## Roadmap

### Completed ✅

- Core type system and error handling
- Data loading and management
- Technical analysis library (40+ indicators)
- Strategy framework
- Backtesting engine with full metrics
- Live trading engine with dry-run mode

### In Progress 🚧

- Exchange integrations (Lighter, Paradex)
- WebSocket market data streaming
- Live order execution

### Planned 📋

- CLI tool for strategy management
- Web dashboard for monitoring
- Additional exchanges (Binance, Coinbase)
- Strategy optimization framework
- Machine learning integration
- Portfolio management tools

## License

MIT License - see [LICENSE](LICENSE) for details

## Community

- **Issues**: [Report bugs](https://github.com/itsparser/velora/issues)
- **Discussions**: [Ask questions](https://github.com/itsparser/velora/discussions)
- **Pull Requests**: [Contribute code](https://github.com/itsparser/velora/pulls)

## Acknowledgments

Built with ❤️ using Rust

Special thanks to the Rust community and the excellent libraries that make this possible:

- Tokio for async runtime
- Polars for data processing
- Serde for serialization
- And many more...

---

**Status**: 🔨 **Active Development**

Core components are production-ready. Exchange integration in progress. Star ⭐ this repo to follow development!
