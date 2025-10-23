# Building Velora

This guide explains how to build and test the Velora umbrella crate.

## Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

## Building

### Build all features (default)

```bash
cargo build -p velora
```

### Build with specific features

```bash
# Only backtesting
cargo build -p velora --no-default-features --features backtest,ta,data

# Only live trading
cargo build -p velora --no-default-features --features engine,exchange,strategy,ta
```

### Build in release mode

```bash
cargo build -p velora --release
```

## Testing

### Run all tests

```bash
cargo test -p velora --all-features
```

### Run specific test

```bash
cargo test -p velora test_backtest_integration
```

### Run tests with output

```bash
cargo test -p velora --all-features -- --nocapture
```

## Examples

### List all examples

```bash
cargo run -p velora --example
```

### Run specific examples

```bash
# Simple backtest
cargo run -p velora --example simple_backtest

# Live trading (dry-run)
cargo run -p velora --example live_trading

# End-to-end strategy development
cargo run -p velora --example strategy_development
```

### Run example in release mode (faster)

```bash
cargo run -p velora --example simple_backtest --release
```

## Verification Checklist

When making changes to the velora crate, verify:

- [ ] Code compiles: `cargo build -p velora --all-features`
- [ ] Tests pass: `cargo test -p velora --all-features`
- [ ] Examples compile: `cargo check -p velora --examples --all-features`
- [ ] Examples run: Test each example manually
- [ ] Documentation builds: `cargo doc -p velora --all-features --no-deps`
- [ ] Clippy is happy: `cargo clippy -p velora --all-features`
- [ ] Format is correct: `cargo fmt -p velora --check`

## Common Issues

### Network errors with crates.io

If you see 403 errors when accessing crates.io:

```
error: failed to get `chrono` as a dependency
Caused by: got 403
```

This is usually a temporary network issue. Try:
1. Wait a few minutes and try again
2. Check your network connection
3. Try using a VPN if you're behind a restrictive firewall

### Compilation errors

If you see compilation errors:

1. Make sure you're using Rust 1.70+: `rustc --version`
2. Update your dependencies: `cargo update`
3. Clean and rebuild: `cargo clean && cargo build -p velora`

### Feature-specific errors

If a feature-specific test fails:

```bash
# Test only that feature
cargo test -p velora --features backtest test_backtest_integration
```

## Performance Testing

### Benchmark backtest performance

```bash
# Run in release mode for accurate performance
cargo run -p velora --example simple_backtest --release

# Time the execution
time cargo run -p velora --example simple_backtest --release
```

### Profile memory usage

```bash
# Install valgrind (Linux)
sudo apt-get install valgrind

# Run with massif
valgrind --tool=massif cargo run -p velora --example simple_backtest --release

# Analyze results
ms_print massif.out.*
```

## Documentation

### Build documentation

```bash
# Build docs for velora crate only
cargo doc -p velora --all-features --no-deps

# Build docs for all dependencies
cargo doc -p velora --all-features

# Open in browser
cargo doc -p velora --all-features --open
```

### Check doc tests

```bash
cargo test -p velora --doc --all-features
```

## Release Build Optimization

For production deployments, use:

```bash
cargo build -p velora --release

# Or build the entire workspace
cargo build --release
```

The release profile is configured in the workspace `Cargo.toml`:

```toml
[profile.release]
opt-level = 3        # Maximum optimization
lto = true          # Link-time optimization
codegen-units = 1   # Single-threaded for more optimization
strip = true        # Remove debug symbols
```

## Cross-compilation

To build for different platforms:

```bash
# Install cross (one-time setup)
cargo install cross

# Build for Linux ARM64
cross build -p velora --target aarch64-unknown-linux-gnu --release

# Build for macOS
cross build -p velora --target x86_64-apple-darwin --release
```

## CI/CD Integration

For continuous integration, use:

```bash
# Fast check (no codegen)
cargo check -p velora --all-features

# Run tests
cargo test -p velora --all-features

# Check formatting
cargo fmt -p velora --check

# Run clippy
cargo clippy -p velora --all-features -- -D warnings
```

## Troubleshooting

### Clear all caches

```bash
# Remove build artifacts
cargo clean

# Remove downloaded crates (careful!)
rm -rf ~/.cargo/registry
rm -rf ~/.cargo/git

# Rebuild
cargo build -p velora --all-features
```

### Verbose build output

```bash
cargo build -p velora --all-features -vv
```

### Check dependency tree

```bash
# Install cargo-tree (one-time)
cargo install cargo-tree

# View dependencies
cargo tree -p velora
```

## Getting Help

If you encounter issues:

1. Check this build guide
2. Search [GitHub Issues](https://github.com/crabby-ai/velora/issues)
3. Ask in [GitHub Discussions](https://github.com/crabby-ai/velora/discussions)
4. Read the [Contributing Guide](../../CONTRIBUTING.md)
