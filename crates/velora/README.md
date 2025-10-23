# Velora

**A modular, high-performance algorithmic trading platform for cryptocurrency markets.**

[![Crates.io](https://img.shields.io/crates/v/velora.svg)](https://crates.io/crates/velora)
[![Documentation](https://docs.rs/velora/badge.svg)](https://docs.rs/velora)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](../../../LICENSE)

## Overview

Velora is a comprehensive trading platform built in Rust that provides everything you need to develop, test, and deploy trading strategies:

- **Market Data** - Ingest and manage data from CSV, Parquet, PostgreSQL
- **Technical Analysis** - 40+ indicators and candlestick pattern recognition
- **Strategy Framework** - Async trait-based strategy development
- **Backtesting** - Realistic simulation with comprehensive metrics
- **Live Trading** - Event-driven engine with dry-run support
- **Exchange Integration** - Connect to DEXs (Lighter, Paradex) and CEXs
- **Risk Management** - Position sizing and limit enforcement

## Quick Start

Add Velora to your `Cargo.toml`:

```toml
[dependencies]
velora = "0.0.1"
```

### Example: Simple Backtest

```rust
use velora::prelude::*;
use async_trait::async_trait;

#[derive(Debug)]
struct MyStrategy {
    // Your strategy fields
}

#[async_trait]
impl Strategy for MyStrategy {
    // Implement strategy trait methods
    async fn on_candle(&mut self, candle: &Candle, ctx: &StrategyContext)
        -> StrategyResult<Signal>
    {
        // Your trading logic here
        Ok(Signal::Hold)
    }

    // ... other required methods
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load data
    let candles = /* load your candles */;

    // Create strategy
    let strategy = MyStrategy::new()?;

    // Configure backtest
    let config = BacktestConfig::new()
        .with_capital(10_000.0)
        .with_execution(ExecutionConfig::realistic());

    // Run backtest
    let report = Backtester::new(config)
        .with_strategy(Box::new(strategy))
        .run(candles)
        .await?;

    // Analyze results
    report.print_summary();

    Ok(())
}
```

## Features

The crate uses feature flags for selective compilation:

- `full` (default) - All features enabled
- `data` - Data ingestion and management
- `ta` - Technical analysis indicators
- `strategy` - Strategy development framework
- `backtest` - Backtesting engine
- `engine` - Live trading engine
- `exchange` - Exchange integrations
- `risk` - Risk management
- `utils` - Integration utilities

### Minimal Installation

To use only specific features:

```toml
[dependencies]
velora = { version = "0.0.1", default-features = false, features = ["backtest", "ta"] }
```

## Architecture

Velora is organized into specialized crates:

```
velora (umbrella crate)
├── velora-core      - Core types and configuration
├── velora-data      - Data ingestion and storage
├── velora-ta        - Technical analysis indicators
├── velora-strategy  - Strategy development framework
├── velora-backtest  - Backtesting engine
├── velora-engine    - Live trading engine
├── velora-exchange  - Exchange connectivity
└── velora-risk      - Risk management
```

Each crate can also be used independently if you only need specific functionality.

## Examples

See the `examples/` directory for complete working examples:

- [`simple_backtest.rs`](examples/simple_backtest.rs) - Basic backtesting workflow
- [`live_trading.rs`](examples/live_trading.rs) - Live trading with dry-run mode
- [`strategy_development.rs`](examples/strategy_development.rs) - End-to-end strategy creation

Run an example:

```bash
cargo run --example simple_backtest
```

## Documentation

### Crate Documentation

- [velora-core](../velora-core) - Core types and utilities
- [velora-data](../velora-data) - Data management
- [velora-ta](../velora-ta) - Technical analysis
- [velora-strategy](../velora-strategy) - Strategy framework
- [velora-backtest](../velora-backtest) - Backtesting
- [velora-engine](../velora-engine) - Live trading
- [velora-exchange](../velora-exchange) - Exchange integration
- [velora-risk](../velora-risk) - Risk management

### Project Documentation

- [Architecture Guide](../../ARCHITECTURE.md) - Detailed technical architecture
- [Development Roadmap](../../ROADMAP.md) - Future development plans
- [Contributing Guidelines](../../CONTRIBUTING.md) - How to contribute

## Performance

Velora is designed for high-frequency trading with:

- **Sub-millisecond latency** for event processing
- **10,000+ candles/second** backtesting throughput
- **~50MB memory** footprint for typical strategies
- **Zero-cost abstractions** leveraging Rust's type system

## Safety

Velora uses Rust's type system to prevent common trading bugs:

- **Type-safe orders** - Compile-time guarantees on order validity
- **No unsafe code** - Memory safety without garbage collection
- **Async-first** - Efficient concurrent operations
- **Comprehensive error handling** - No unwraps in production code

## Testing

Run all tests:

```bash
cargo test -p velora --all-features
```

Run specific tests:

```bash
cargo test -p velora test_backtest_integration
```

## License

This project is licensed under the MIT License - see the [LICENSE](../../LICENSE) file for details.

## Disclaimer

**IMPORTANT**: This software is for educational and research purposes only.

- Trading cryptocurrencies carries significant financial risk
- Past performance does not guarantee future results
- Always test strategies thoroughly in dry-run mode before live trading
- Start with small amounts when going live
- The authors are not responsible for any financial losses

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

## Community

- [GitHub Issues](https://github.com/crabby-ai/velora/issues) - Bug reports and feature requests
- [GitHub Discussions](https://github.com/crabby-ai/velora/discussions) - Questions and discussions

## Acknowledgments

Built with:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tokio](https://tokio.rs/) - Async runtime
- [Polars](https://www.pola.rs/) - Fast data processing
- And many other amazing open-source projects

---

**Made with ❤️ by the Velora community**
