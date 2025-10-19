# Contributing to Velora

Thank you for considering contributing to Velora! This document provides guidelines and instructions for contributing to the project.

---

## Table of Contents

1. [Code of Conduct](#code-of-conduct)
2. [Getting Started](#getting-started)
3. [Development Workflow](#development-workflow)
4. [Code Standards](#code-standards)
5. [Testing Guidelines](#testing-guidelines)
6. [Documentation](#documentation)
7. [Pull Request Process](#pull-request-process)
8. [Areas for Contribution](#areas-for-contribution)

---

## Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, background, or identity.

### Expected Behavior

- Be respectful and constructive in all interactions
- Provide helpful feedback and accept feedback graciously
- Focus on what's best for the project and community
- Show empathy towards other community members

### Unacceptable Behavior

- Harassment, discrimination, or trolling
- Personal attacks or inflammatory comments
- Publishing others' private information
- Other conduct that would be inappropriate in a professional setting

---

## Getting Started

### Prerequisites

- **Rust**: Install via [rustup](https://rustup.rs/) (version 1.75+)
- **Git**: For version control
- **PostgreSQL** (optional): For data storage features
- **Exchange API Keys** (optional): For testing exchange integrations

### Initial Setup

1. **Fork the repository**
   ```bash
   # Go to https://github.com/crabby-ai/velora and click "Fork"
   ```

2. **Clone your fork**
   ```bash
   git clone https://github.com/YOUR_USERNAME/velora.git
   cd velora
   ```

3. **Add upstream remote**
   ```bash
   git remote add upstream https://github.com/crabby-ai/velora.git
   ```

4. **Build the project**
   ```bash
   cargo build --workspace
   ```

5. **Run tests**
   ```bash
   cargo test --workspace
   ```

6. **Check formatting and lints**
   ```bash
   cargo fmt --all -- --check
   cargo clippy --workspace --all-targets --all-features
   ```

### Development Environment

We recommend using:
- **VSCode** with `rust-analyzer` extension
- **RustRover** or **IntelliJ IDEA** with Rust plugin
- **Neovim/Vim** with `coc-rust-analyzer` or native LSP

---

## Development Workflow

### 1. Create a Feature Branch

```bash
# Sync with upstream
git fetch upstream
git checkout main
git merge upstream/main

# Create feature branch
git checkout -b feature/your-feature-name
```

Branch naming conventions:
- `feature/feature-name` - New features
- `fix/bug-description` - Bug fixes
- `docs/documentation-update` - Documentation changes
- `refactor/component-name` - Code refactoring
- `perf/optimization-description` - Performance improvements

### 2. Make Changes

Follow the [Code Standards](#code-standards) section below.

### 3. Test Your Changes

```bash
# Run all tests
cargo test --workspace

# Run tests for specific crate
cargo test -p velora-strategy

# Run with output
cargo test --workspace -- --nocapture

# Run specific test
cargo test test_sma_calculation
```

### 4. Format and Lint

```bash
# Format code
cargo fmt --all

# Run clippy
cargo clippy --workspace --all-targets --all-features

# Fix clippy warnings (when safe)
cargo clippy --workspace --all-targets --all-features --fix --allow-dirty
```

### 5. Commit Changes

Write clear, descriptive commit messages following the [Conventional Commits](https://www.conventionalcommits.org/) format:

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting, etc.)
- `refactor`: Code refactoring
- `perf`: Performance improvements
- `test`: Adding or updating tests
- `chore`: Maintenance tasks (dependencies, CI, etc.)

**Examples**:
```bash
git commit -m "feat(strategy): add RSI mean reversion strategy

- Implement RSI calculation with configurable periods
- Add overbought/oversold threshold parameters
- Include example in examples/ directory

Closes #42"
```

```bash
git commit -m "fix(backtest): correct slippage calculation for limit orders

Previously, limit orders were not accounting for partial fills.
Now properly simulates partial fills based on orderbook depth.

Fixes #123"
```

### 6. Push and Create Pull Request

```bash
# Push to your fork
git push origin feature/your-feature-name

# Create pull request on GitHub
```

---

## Code Standards

### Rust Style

Follow the official [Rust Style Guide](https://doc.rust-lang.org/nightly/style-guide/). Key points:

1. **Formatting**: Use `rustfmt` with default settings
   ```bash
   cargo fmt --all
   ```

2. **Naming Conventions**:
   - `snake_case` for functions, variables, modules
   - `CamelCase` for types, traits, enums
   - `SCREAMING_SNAKE_CASE` for constants
   - Prefix unused variables with `_`

3. **Documentation**:
   - All public APIs must have doc comments
   - Use `///` for doc comments, `//` for implementation comments
   - Include examples in doc comments where helpful

4. **Error Handling**:
   - Use `Result<T, Error>` for fallible operations
   - Use `?` operator for error propagation
   - Create descriptive error types with `thiserror`

### Code Organization

1. **Module Structure**:
   ```rust
   // Good: logical grouping
   mod strategy {
       pub mod sma;
       pub mod rsi;
       pub mod traits;
   }

   // Avoid: flat structure
   mod sma_strategy;
   mod rsi_strategy;
   mod strategy_trait;
   ```

2. **Imports**:
   ```rust
   // Group imports: std, external crates, internal crates
   use std::collections::HashMap;
   use std::sync::Arc;

   use async_trait::async_trait;
   use serde::{Deserialize, Serialize};

   use velora_core::{Order, Symbol};
   use crate::context::StrategyContext;
   ```

3. **File Size**: Keep files under 500 lines when possible. Split into submodules if larger.

### Performance Guidelines

1. **Avoid Unnecessary Allocations**:
   ```rust
   // Bad: allocates a new String
   fn format_symbol(symbol: &str) -> String {
       format!("{}USD", symbol)
   }

   // Good: borrows
   fn format_symbol(symbol: &str) -> impl Display {
       format!("{}USD", symbol)
   }
   ```

2. **Use References**:
   ```rust
   // Bad: takes ownership
   fn calculate_pnl(position: Position) -> f64 { ... }

   // Good: borrows
   fn calculate_pnl(position: &Position) -> f64 { ... }
   ```

3. **Prefer Iterators**:
   ```rust
   // Bad: allocates intermediate Vec
   let sum: f64 = values.iter()
       .map(|v| v * 2.0)
       .collect::<Vec<_>>()
       .iter()
       .sum();

   // Good: lazy evaluation
   let sum: f64 = values.iter()
       .map(|v| v * 2.0)
       .sum();
   ```

4. **Inline Hot Paths**:
   ```rust
   #[inline(always)]
   pub fn calculate_ema_step(prev: f64, value: f64, alpha: f64) -> f64 {
       alpha * value + (1.0 - alpha) * prev
   }
   ```

### Async Best Practices

1. **Use `async_trait` for Traits**:
   ```rust
   use async_trait::async_trait;

   #[async_trait]
   pub trait Exchange {
       async fn get_candles(&self, symbol: &Symbol) -> Result<Vec<Candle>>;
   }
   ```

2. **Prefer `tokio::select!` for Concurrency**:
   ```rust
   tokio::select! {
       candle = candle_stream.next() => {
           // Handle candle
       }
       trade = trade_stream.next() => {
           // Handle trade
       }
   }
   ```

3. **Use Bounded Channels**:
   ```rust
   // Good: prevents unbounded growth
   let (tx, rx) = mpsc::channel(100);

   // Avoid: can grow indefinitely
   let (tx, rx) = mpsc::unbounded_channel();
   ```

---

## Testing Guidelines

### Test Organization

```
crates/velora-strategy/
├── src/
│   ├── sma.rs          # Implementation
│   └── lib.rs
└── tests/
    ├── sma_tests.rs    # Integration tests
    └── common/         # Test utilities
        └── mod.rs
```

### Unit Tests

Place unit tests in the same file as the code:

```rust
// src/sma.rs
pub fn sma(values: &[f64], period: usize) -> Vec<f64> {
    // Implementation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_basic() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&values, 3);
        assert_eq!(result, vec![2.0, 3.0, 4.0]);
    }

    #[test]
    fn test_sma_empty_input() {
        let values = vec![];
        let result = sma(&values, 3);
        assert!(result.is_empty());
    }

    #[test]
    fn test_sma_insufficient_data() {
        let values = vec![1.0, 2.0];
        let result = sma(&values, 3);
        assert!(result.is_empty());
    }
}
```

### Integration Tests

Place in `tests/` directory:

```rust
// tests/backtest_integration.rs
use velora_backtest::BacktestEngine;
use velora_strategy::SMACrossover;

#[tokio::test]
async fn test_sma_crossover_backtest() {
    let strategy = SMACrossover::new(10, 20);
    let data = load_test_data("test_data.csv");

    let engine = BacktestEngine::builder()
        .strategy(strategy)
        .data(data)
        .initial_capital(10000.0)
        .build();

    let result = engine.run().await.unwrap();

    assert!(result.total_trades > 0);
    assert!(result.sharpe_ratio > 0.0);
}
```

### Property-Based Tests

Use `proptest` for testing invariants:

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_sma_length(
        values in prop::collection::vec(any::<f64>(), 10..100),
        period in 1..10usize
    ) {
        let result = sma(&values, period);

        if values.len() >= period {
            prop_assert_eq!(result.len(), values.len() - period + 1);
        } else {
            prop_assert!(result.is_empty());
        }
    }
}
```

### Benchmark Tests

Use `criterion` for performance benchmarks:

```rust
// benches/sma_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use velora_ta::sma;

fn bench_sma(c: &mut Criterion) {
    let values: Vec<f64> = (0..10000).map(|i| i as f64).collect();

    c.bench_function("sma_10000_values_period_20", |b| {
        b.iter(|| sma(black_box(&values), black_box(20)))
    });
}

criterion_group!(benches, bench_sma);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench -p velora-ta
```

### Test Coverage

We aim for 80%+ code coverage. Check coverage with `tarpaulin`:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run coverage
cargo tarpaulin --workspace --out Html --output-dir coverage/
```

---

## Documentation

### Doc Comments

All public items must have documentation:

```rust
/// Calculates the Simple Moving Average (SMA) over a sliding window.
///
/// The SMA is the unweighted mean of the previous `period` data points.
/// For a period of `n`, the SMA is calculated as:
///
/// ```text
/// SMA(t) = (P(t) + P(t-1) + ... + P(t-n+1)) / n
/// ```
///
/// # Arguments
///
/// * `values` - Time series data as a slice of f64
/// * `period` - Number of periods to average over
///
/// # Returns
///
/// A `Vec<f64>` containing the SMA values. The length will be
/// `values.len() - period + 1` if there's sufficient data, or empty otherwise.
///
/// # Examples
///
/// ```
/// use velora_ta::sma;
///
/// let prices = vec![1.0, 2.0, 3.0, 4.0, 5.0];
/// let sma_values = sma(&prices, 3);
/// assert_eq!(sma_values, vec![2.0, 3.0, 4.0]);
/// ```
///
/// # Performance
///
/// Time complexity: O(n) where n is the length of `values`
/// Space complexity: O(n - period + 1) for the output vector
pub fn sma<T: AsRef<[f64]>>(values: T, period: usize) -> Vec<f64> {
    // Implementation
}
```

### Code Examples

Include working examples in `examples/` directory:

```rust
// examples/sma_crossover_backtest.rs
use velora_backtest::BacktestEngine;
use velora_strategy::SMACrossover;
use velora_data::CsvDataSource;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load historical data
    let data = CsvDataSource::new("data/BTCUSDT_1h.csv")?;

    // Create strategy
    let strategy = SMACrossover::builder()
        .fast_period(10)
        .slow_period(20)
        .build();

    // Configure backtest
    let engine = BacktestEngine::builder()
        .strategy(strategy)
        .data(data)
        .initial_capital(10000.0)
        .commission(0.001)  // 0.1%
        .build();

    // Run backtest
    let result = engine.run().await?;

    // Print results
    println!("Total Return: {:.2}%", result.total_return * 100.0);
    println!("Sharpe Ratio: {:.2}", result.sharpe_ratio);
    println!("Max Drawdown: {:.2}%", result.max_drawdown * 100.0);
    println!("Win Rate: {:.2}%", result.win_rate * 100.0);

    Ok(())
}
```

### README Updates

When adding new features, update relevant README files:
- Main `README.md` for high-level changes
- Crate-specific `README.md` files for detailed documentation

---

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass**:
   ```bash
   cargo test --workspace
   ```

2. **Format code**:
   ```bash
   cargo fmt --all
   ```

3. **Fix clippy warnings**:
   ```bash
   cargo clippy --workspace --all-targets --all-features
   ```

4. **Update documentation**:
   - Add/update doc comments
   - Update README if needed
   - Add example if appropriate

5. **Add tests**:
   - Unit tests for new functions
   - Integration tests for new features
   - Update existing tests if behavior changed

### PR Template

Use this template for your pull request:

```markdown
## Description

Brief description of what this PR does.

## Motivation

Why is this change needed? What problem does it solve?

## Changes

- List of changes made
- Another change
- Yet another change

## Testing

How has this been tested?

- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Manual testing performed

## Performance Impact

Does this change affect performance? If yes, provide benchmarks.

## Breaking Changes

Does this introduce breaking changes? If yes, describe migration path.

## Checklist

- [ ] Code follows style guidelines
- [ ] Self-review of code completed
- [ ] Documentation updated
- [ ] Tests added/updated
- [ ] All tests pass
- [ ] No clippy warnings
- [ ] Commits follow conventional commit format
```

### Review Process

1. **Automated Checks**: CI will run tests, clippy, and format checks
2. **Code Review**: Maintainers will review your code
3. **Feedback**: Address any requested changes
4. **Approval**: Once approved, your PR will be merged

### Merging

- PRs are merged using "Squash and Merge" to keep history clean
- Ensure commit message is clear and descriptive
- Delete branch after merge

---

## Areas for Contribution

### High Priority

1. **Exchange Integrations**
   - Complete Binance implementation (REST + WebSocket)
   - Finish Lighter (Arbitrum) integration
   - Complete Paradex (Starknet) integration
   - Add new exchanges (Coinbase, Kraken, etc.)

2. **Strategy Examples**
   - More example strategies (RSI, Bollinger Bands, etc.)
   - Multi-timeframe strategies
   - Portfolio strategies
   - Machine learning strategies

3. **Documentation**
   - Tutorials for beginners
   - Strategy development guides
   - Deployment guides
   - Video tutorials

4. **Testing**
   - Increase test coverage
   - Add integration tests
   - Property-based tests
   - Performance benchmarks

### Medium Priority

1. **Risk Management**
   - Position sizing algorithms
   - Portfolio risk metrics
   - Drawdown controls
   - VaR calculations

2. **Performance Optimization**
   - SIMD for TA calculations
   - Reduce allocations in hot paths
   - Async optimization
   - Database query optimization

3. **Monitoring & Observability**
   - Prometheus metrics
   - Grafana dashboards
   - Structured logging
   - Distributed tracing

4. **Data Infrastructure**
   - TimescaleDB integration
   - Data quality checks
   - Real-time data pipelines
   - Historical data replay

### Good First Issues

Look for issues labeled `good-first-issue`:
- Documentation improvements
- Adding tests
- Code cleanup
- Minor bug fixes

### Advanced Contributions

For experienced contributors:
- Distributed system design (high availability)
- Machine learning integration
- GPU/FPGA acceleration
- Novel trading strategies

---

## Community

### Getting Help

- **Discord**: [Join our Discord server](https://discord.gg/velora) (coming soon)
- **GitHub Discussions**: Ask questions, share ideas
- **GitHub Issues**: Report bugs, request features

### Monthly Community Calls

We hold monthly community calls (schedule TBA):
- Project updates
- Feature demos
- Q&A session
- Contributor recognition

### Recognition

Contributors are recognized in:
- `CONTRIBUTORS.md` file
- Release notes
- Project README

---

## License

By contributing to Velora, you agree that your contributions will be licensed under the MIT License.

---

## Questions?

If you have questions about contributing, please:
1. Check existing documentation
2. Search GitHub issues/discussions
3. Ask on Discord
4. Open a GitHub discussion

Thank you for contributing to Velora! 🚀

---

**Last Updated**: 2025-10-20
