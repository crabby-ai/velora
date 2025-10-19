# velora-backtest

> Backtesting engine for Velora trading strategies

## Overview

Historical simulation engine for testing trading strategies with realistic order matching, commission, and slippage.

## Key Features

- **Backtest Engine**: Replay historical data through strategies
- **Order Simulator**: Realistic order matching and fills
- **Performance Metrics**: Sharpe ratio, max drawdown, win rate, profit factor
- **Equity Curve**: Track portfolio value over time
- **Commission & Slippage**: Realistic trading costs

## Public API (Planned)

```rust
pub struct BacktestEngine {
    pub async fn run(&mut self, strategy: &mut dyn Strategy, data: Vec<Candle>) -> Result<BacktestResult>;
}

pub struct BacktestResult {
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub trades: Vec<Trade>,
    pub equity_curve: Vec<(DateTime, f64)>,
}
```

## Status

🚧 **In Planning**

## License

MIT
