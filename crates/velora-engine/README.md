# velora-engine

> Live trading engine for Velora

## Overview

Real-time trading engine that executes strategies in live markets with order management, position monitoring, and risk controls.

## Key Features

- **Trading Engine**: Execute strategies in real-time
- **Order Executor**: Submit and manage orders
- **Position Monitor**: Track positions and P&L
- **Dry-Run Mode**: Test without real money
- **Event Loop**: Coordinate data, strategy, and execution

## Public API (Planned)

```rust
pub struct TradingEngine {
    pub async fn start(&mut self) -> Result<()>;
    pub async fn stop(&mut self) -> Result<()>;
}

pub struct OrderExecutor {
    pub async fn execute(&self, signal: Signal) -> Result<Order>;
}

pub struct PositionMonitor {
    pub fn update(&mut self, tick: &Tick);
    pub fn get_positions(&self) -> Vec<Position>;
}
```

## Status

🚧 **In Planning**

## License

MIT
