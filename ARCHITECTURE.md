# Velora Architecture

This document provides a comprehensive technical overview of Velora's architecture, design decisions, and implementation details.

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architectural Principles](#architectural-principles)
3. [Crate Structure](#crate-structure)
4. [Data Flow](#data-flow)
5. [Core Components](#core-components)
6. [Performance Considerations](#performance-considerations)
7. [Testing Strategy](#testing-strategy)
8. [Deployment Architecture](#deployment-architecture)

---

## System Overview

Velora is a **high-frequency trading (HFT) framework** designed for cryptocurrency markets. It follows a **modular, event-driven architecture** built on Rust's async/await model using the Tokio runtime.

### Key Characteristics

- **Event-Driven**: All components communicate via async events (market data, orders, fills)
- **Zero-Copy**: Minimize allocations in hot paths using references and arena allocation
- **Type-Safe**: Leverage Rust's type system to prevent bugs at compile time
- **Composable**: Crates are independent and can be used standalone or together
- **Observable**: Comprehensive instrumentation with `tracing` for debugging and monitoring

### High-Level Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Application Layer                          │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Strategy   │  │  Backtester  │  │ Live Engine  │          │
│  │  (velora)    │  │(velora-backtest)│(velora-engine)│          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Business Logic Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Strategies  │  │     Risk     │  │      TA      │          │
│  │(velora-strategy)│(velora-risk) │(velora-ta)     │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      Infrastructure Layer                       │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │   Exchange   │  │     Data     │  │     Core     │          │
│  │(velora-exchange)│(velora-data) │(velora-core)   │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
                              │
┌─────────────────────────────────────────────────────────────────┐
│                      External Systems                           │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
│  │  Exchanges   │  │  Databases   │  │  Monitoring  │          │
│  │ (CEX/DEX)    │  │(PostgreSQL)  │  │(Prometheus)  │          │
│  └──────────────┘  └──────────────┘  └──────────────┘          │
└─────────────────────────────────────────────────────────────────┘
```

---

## Architectural Principles

### 1. Separation of Concerns

Each crate has a single, well-defined responsibility:

- **velora-core**: Common types and utilities (no business logic)
- **velora-data**: Data storage and retrieval (no trading logic)
- **velora-ta**: Technical analysis (pure functions, no state)
- **velora-strategy**: Strategy abstraction (no execution logic)
- **velora-backtest**: Historical simulation (no live trading)
- **velora-engine**: Live trading orchestration (no strategy logic)
- **velora-exchange**: Exchange connectivity (no strategy logic)
- **velora-risk**: Risk management (independent of strategies)

### 2. Dependency Inversion

Higher-level modules depend on abstractions (traits), not concrete implementations:

```rust
// Strategy depends on abstract Context, not concrete Engine
#[async_trait]
pub trait Strategy {
    async fn on_candle(&mut self, candle: &Candle, ctx: &StrategyContext);
}

// Engine provides concrete implementation
pub struct LiveEngine {
    // ...
}

impl LiveEngine {
    fn create_context(&self) -> StrategyContext {
        // Concrete implementation
    }
}
```

### 3. Event-Driven Design

All interactions are asynchronous events:

```rust
pub enum Event {
    MarketData(MarketDataEvent),
    Order(OrderEvent),
    Fill(FillEvent),
    Position(PositionEvent),
    Signal(SignalEvent),
}
```

This enables:
- **Decoupling**: Producers don't need to know about consumers
- **Testability**: Easy to inject mock events
- **Replay**: Historical data can be replayed deterministically

### 4. Zero-Cost Abstractions

Performance-critical paths use zero-cost abstractions:

```rust
// Generic over ownership model (T can be &[f64] or Vec<f64>)
pub fn sma<T: AsRef<[f64]>>(values: T, period: usize) -> Vec<f64> {
    // No runtime cost for abstraction
}

// Inline hints for hot paths
#[inline(always)]
fn calculate_ema_step(prev: f64, value: f64, alpha: f64) -> f64 {
    alpha * value + (1.0 - alpha) * prev
}
```

### 5. Type-Driven Design

Use the type system to prevent bugs:

```rust
// NewType pattern prevents mixing up IDs
pub struct OrderId(Uuid);
pub struct StrategyId(Uuid);

// Phantom types for compile-time state machines
pub struct Order<State> {
    id: OrderId,
    _state: PhantomData<State>,
}

pub struct Created;
pub struct Submitted;
pub struct Filled;

// Can only submit a Created order
impl Order<Created> {
    pub fn submit(self) -> Order<Submitted> { ... }
}

// Can only fill a Submitted order
impl Order<Submitted> {
    pub fn fill(self) -> Order<Filled> { ... }
}
```

---

## Crate Structure

### velora-core

**Purpose**: Shared types and utilities used across all crates.

**Key Components**:
```
velora-core/
├── types.rs           # Core trading types (Order, Trade, Candle, etc.)
├── config/            # Configuration management
│   ├── mod.rs
│   ├── exchange.rs    # Exchange configs
│   ├── database.rs    # Database configs
│   ├── engine.rs      # Engine configs
│   ├── risk.rs        # Risk configs
│   └── logging.rs     # Logging configs
├── errors.rs          # Error types
└── lib.rs
```

**Design Decisions**:
- **No dependencies on other Velora crates**: Prevents circular dependencies
- **Minimal external dependencies**: Only serde, chrono, uuid, rust_decimal
- **Configuration**: Multi-source loading (TOML → ENV → CLI) using `gonfig`

**Types**:
```rust
// Core trading primitives
pub struct Symbol(String);
pub struct Price(Decimal);  // Use Decimal for financial precision
pub struct Quantity(Decimal);
pub type OrderId = Uuid;

// Market data
pub struct Candle {
    pub open: Price,
    pub high: Price,
    pub low: Price,
    pub close: Price,
    pub volume: Quantity,
    pub timestamp: DateTime<Utc>,
}

pub struct Trade {
    pub price: Price,
    pub quantity: Quantity,
    pub side: Side,  // Buy or Sell
    pub timestamp: DateTime<Utc>,
}

pub struct OrderBook {
    pub bids: Vec<PriceLevel>,
    pub asks: Vec<PriceLevel>,
    pub timestamp: DateTime<Utc>,
}

// Trading
pub struct Order {
    pub id: OrderId,
    pub symbol: Symbol,
    pub side: Side,
    pub order_type: OrderType,  // Market, Limit
    pub quantity: Quantity,
    pub price: Option<Price>,  // None for market orders
    pub status: OrderStatus,
}

pub enum OrderStatus {
    Created,
    Submitted,
    PartiallyFilled,
    Filled,
    Cancelled,
    Rejected,
}

pub struct Position {
    pub symbol: Symbol,
    pub side: PositionSide,  // Long, Short, Flat
    pub quantity: Quantity,
    pub entry_price: Price,
    pub current_price: Price,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
}
```

### velora-data

**Purpose**: Historical and real-time data management.

**Key Components**:
```
velora-data/
├── sources/
│   ├── csv.rs         # CSV data source
│   ├── parquet.rs     # Parquet data source
│   └── database.rs    # PostgreSQL/TimescaleDB
├── stream.rs          # Real-time data streaming
├── cache.rs           # In-memory data cache
├── schema.rs          # Data schemas and validation
└── lib.rs
```

**Data Flow**:
```
Historical:  CSV/Parquet → DataFrame → Iterator<Candle>
Real-time:   Exchange → WebSocket → Stream<Candle>
Storage:     Candle → Database → TimescaleDB
```

**Performance Optimizations**:
- **Lazy loading**: Only load data when needed
- **Memory mapping**: Use mmap for large CSV files
- **Batch processing**: Process data in chunks to reduce allocations
- **Compression**: Use Parquet for efficient storage

### velora-ta

**Purpose**: Technical analysis indicators and chart patterns.

**Key Components**:
```
velora-ta/
├── trend/             # Trend indicators (SMA, EMA, etc.)
├── momentum/          # Momentum indicators (RSI, MACD, etc.)
├── volatility/        # Volatility indicators (Bollinger Bands, ATR)
├── volume/            # Volume indicators (OBV, VWAP)
├── patterns/          # Candlestick patterns
│   ├── single.rs      # Single candle patterns (Doji, Hammer)
│   ├── double.rs      # Double candle patterns (Engulfing)
│   └── triple.rs      # Triple candle patterns (Morning Star)
├── types.rs
└── lib.rs
```

**Design**:
- **Pure functions**: No state, easy to test and parallelize
- **Generic over input**: Accept `&[f64]`, `Vec<f64>`, or any `AsRef<[f64]>`
- **SIMD**: Use portable SIMD for vectorized operations
- **Numerical stability**: Carefully handle edge cases (division by zero, NaN)

**Example Implementation**:
```rust
/// Calculate Simple Moving Average
pub fn sma<T: AsRef<[f64]>>(values: T, period: usize) -> Vec<f64> {
    let values = values.as_ref();
    let mut result = Vec::with_capacity(values.len());

    if values.len() < period {
        return result;
    }

    // First value: simple average
    let mut sum: f64 = values[..period].iter().sum();
    result.push(sum / period as f64);

    // Sliding window
    for i in period..values.len() {
        sum += values[i] - values[i - period];
        result.push(sum / period as f64);
    }

    result
}

/// Calculate Exponential Moving Average
pub fn ema<T: AsRef<[f64]>>(values: T, period: usize) -> Vec<f64> {
    let values = values.as_ref();
    let mut result = Vec::with_capacity(values.len());

    if values.is_empty() {
        return result;
    }

    let alpha = 2.0 / (period as f64 + 1.0);

    // First value: simple average
    let sum: f64 = values[..period.min(values.len())].iter().sum();
    let mut ema = sum / period.min(values.len()) as f64;
    result.push(ema);

    // Exponential smoothing
    for &value in &values[1..] {
        ema = alpha * value + (1.0 - alpha) * ema;
        result.push(ema);
    }

    result
}
```

### velora-strategy

**Purpose**: Strategy abstraction and common strategy patterns.

**Key Components**:
```
velora-strategy/
├── traits.rs          # Strategy trait definition
├── context.rs         # Strategy context (market data, positions)
├── state.rs           # Strategy state management
├── signals.rs         # Signal types (Entry, Exit, Adjust)
└── lib.rs
```

**Strategy Trait**:
```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Called when a new candle is received
    async fn on_candle(&mut self, candle: &Candle, ctx: &StrategyContext);

    /// Called when a new trade is received
    async fn on_trade(&mut self, trade: &Trade, ctx: &StrategyContext) {
        // Default implementation: do nothing
    }

    /// Called when an order is filled
    async fn on_fill(&mut self, fill: &Fill, ctx: &StrategyContext) {
        // Default implementation: do nothing
    }

    /// Called periodically (e.g., every second)
    async fn on_timer(&mut self, ctx: &StrategyContext) {
        // Default implementation: do nothing
    }

    /// Initialize strategy (load state, warm up indicators)
    async fn initialize(&mut self, ctx: &StrategyContext) -> Result<()> {
        Ok(())
    }

    /// Cleanup before shutdown
    async fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
```

**StrategyContext**:
```rust
pub struct StrategyContext {
    /// Get current positions
    pub fn get_positions(&self) -> &HashMap<Symbol, Position>;

    /// Get current portfolio value
    pub fn get_portfolio_value(&self) -> f64;

    /// Get available capital
    pub fn get_available_capital(&self) -> f64;

    /// Submit an order
    pub async fn submit_order(&self, order: NewOrder) -> Result<OrderId>;

    /// Cancel an order
    pub async fn cancel_order(&self, order_id: OrderId) -> Result<()>;

    /// Get historical candles
    pub async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        limit: usize,
    ) -> Result<Vec<Candle>>;

    /// Get current orderbook
    pub async fn get_orderbook(&self, symbol: &Symbol) -> Result<OrderBook>;
}
```

**Example Strategy**:
```rust
pub struct SMACrossover {
    fast_period: usize,
    slow_period: usize,
    fast_sma: Vec<f64>,
    slow_sma: Vec<f64>,
    prices: Vec<f64>,
}

#[async_trait]
impl Strategy for SMACrossover {
    async fn on_candle(&mut self, candle: &Candle, ctx: &StrategyContext) {
        // Update price history
        self.prices.push(candle.close.as_f64());

        // Calculate indicators
        self.fast_sma = sma(&self.prices, self.fast_period);
        self.slow_sma = sma(&self.prices, self.slow_period);

        if self.fast_sma.len() < 2 || self.slow_sma.len() < 2 {
            return;  // Not enough data
        }

        let fast_prev = self.fast_sma[self.fast_sma.len() - 2];
        let fast_curr = self.fast_sma[self.fast_sma.len() - 1];
        let slow_prev = self.slow_sma[self.slow_sma.len() - 2];
        let slow_curr = self.slow_sma[self.slow_sma.len() - 1];

        // Golden cross: fast crosses above slow → BUY
        if fast_prev <= slow_prev && fast_curr > slow_curr {
            let order = NewOrder {
                symbol: candle.symbol.clone(),
                side: Side::Buy,
                order_type: OrderType::Market,
                quantity: Quantity::from(1.0),
                price: None,
            };
            ctx.submit_order(order).await.ok();
        }

        // Death cross: fast crosses below slow → SELL
        if fast_prev >= slow_prev && fast_curr < slow_curr {
            let order = NewOrder {
                symbol: candle.symbol.clone(),
                side: Side::Sell,
                order_type: OrderType::Market,
                quantity: Quantity::from(1.0),
                price: None,
            };
            ctx.submit_order(order).await.ok();
        }
    }
}
```

### velora-backtest

**Purpose**: Historical backtesting engine with realistic simulation.

**Key Components**:
```
velora-backtest/
├── engine.rs          # Backtest engine
├── fills.rs           # Fill simulation (market impact, slippage)
├── metrics.rs         # Performance metrics
├── config.rs          # Backtest configuration
└── lib.rs
```

**Architecture**:
```
┌────────────────────────────────────────────────┐
│           Backtest Engine                      │
│  ┌──────────────────────────────────────────┐  │
│  │  1. Load Historical Data                 │  │
│  │     CSV/Parquet → Iterator<Candle>       │  │
│  └──────────────────────────────────────────┘  │
│                    │                            │
│  ┌──────────────────────────────────────────┐  │
│  │  2. Initialize Strategy                  │  │
│  │     strategy.initialize(ctx)             │  │
│  └──────────────────────────────────────────┘  │
│                    │                            │
│  ┌──────────────────────────────────────────┐  │
│  │  3. Event Loop                           │  │
│  │     for candle in data:                  │  │
│  │       strategy.on_candle(candle, ctx)    │  │
│  │       process_orders()                   │  │
│  │       simulate_fills()                   │  │
│  │       update_positions()                 │  │
│  └──────────────────────────────────────────┘  │
│                    │                            │
│  ┌──────────────────────────────────────────┐  │
│  │  4. Calculate Metrics                    │  │
│  │     Sharpe, Drawdown, Win Rate, etc.     │  │
│  └──────────────────────────────────────────┘  │
└────────────────────────────────────────────────┘
```

**Fill Simulation**:
```rust
pub trait FillModel: Send + Sync {
    fn simulate_fill(
        &self,
        order: &Order,
        candle: &Candle,
        orderbook: Option<&OrderBook>,
    ) -> Option<Fill>;
}

pub struct SimpleFillModel {
    slippage: f64,  // e.g., 0.001 = 0.1%
}

impl FillModel for SimpleFillModel {
    fn simulate_fill(
        &self,
        order: &Order,
        candle: &Candle,
        _orderbook: Option<&OrderBook>,
    ) -> Option<Fill> {
        match order.order_type {
            OrderType::Market => {
                // Fill at close price + slippage
                let fill_price = match order.side {
                    Side::Buy => candle.close * (1.0 + self.slippage),
                    Side::Sell => candle.close * (1.0 - self.slippage),
                };

                Some(Fill {
                    order_id: order.id,
                    price: fill_price,
                    quantity: order.quantity,
                    timestamp: candle.timestamp,
                })
            }
            OrderType::Limit => {
                // Check if limit price was touched
                let limit_price = order.price.unwrap();
                let filled = match order.side {
                    Side::Buy => candle.low <= limit_price,
                    Side::Sell => candle.high >= limit_price,
                };

                if filled {
                    Some(Fill {
                        order_id: order.id,
                        price: limit_price,
                        quantity: order.quantity,
                        timestamp: candle.timestamp,
                    })
                } else {
                    None
                }
            }
        }
    }
}
```

**Metrics Calculation**:
```rust
pub struct BacktestMetrics {
    pub total_return: f64,
    pub annualized_return: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_drawdown: f64,
    pub win_rate: f64,
    pub profit_factor: f64,
    pub total_trades: usize,
    pub winning_trades: usize,
    pub losing_trades: usize,
}

impl BacktestMetrics {
    pub fn calculate(trades: &[Trade], equity_curve: &[f64]) -> Self {
        let returns = Self::calculate_returns(equity_curve);

        Self {
            total_return: Self::total_return(equity_curve),
            annualized_return: Self::annualized_return(&returns),
            sharpe_ratio: Self::sharpe_ratio(&returns),
            sortino_ratio: Self::sortino_ratio(&returns),
            max_drawdown: Self::max_drawdown(equity_curve),
            win_rate: Self::win_rate(trades),
            profit_factor: Self::profit_factor(trades),
            total_trades: trades.len(),
            winning_trades: trades.iter().filter(|t| t.pnl > 0.0).count(),
            losing_trades: trades.iter().filter(|t| t.pnl <= 0.0).count(),
        }
    }

    fn sharpe_ratio(returns: &[f64]) -> f64 {
        let mean = returns.iter().sum::<f64>() / returns.len() as f64;
        let variance = returns.iter()
            .map(|r| (r - mean).powi(2))
            .sum::<f64>() / returns.len() as f64;
        let std_dev = variance.sqrt();

        if std_dev == 0.0 {
            0.0
        } else {
            mean / std_dev * (252.0_f64).sqrt()  // Annualized
        }
    }
}
```

### velora-engine

**Purpose**: Live trading orchestration and dry-run mode.

**Key Components**:
```
velora-engine/
├── engine.rs          # Live engine orchestration
├── order_manager.rs   # Order lifecycle management
├── position_tracker.rs# Position and P&L tracking
├── events.rs          # Event types
├── errors.rs          # Engine-specific errors
└── lib.rs
```

**Engine Architecture**:
```
┌────────────────────────────────────────────────┐
│              Live Engine                       │
│                                                │
│  ┌──────────────────────────────────────────┐  │
│  │     Market Data Subscriber               │  │
│  │  (WebSocket from Exchange)               │  │
│  └──────────────┬───────────────────────────┘  │
│                 │                               │
│                 ▼                               │
│  ┌──────────────────────────────────────────┐  │
│  │     Event Queue (MPSC Channel)           │  │
│  └──────────────┬───────────────────────────┘  │
│                 │                               │
│                 ▼                               │
│  ┌──────────────────────────────────────────┐  │
│  │     Strategy Execution                   │  │
│  │  strategy.on_candle(candle, ctx)         │  │
│  └──────────────┬───────────────────────────┘  │
│                 │                               │
│                 ▼                               │
│  ┌──────────────────────────────────────────┐  │
│  │     Order Manager                        │  │
│  │  - Validate orders                       │  │
│  │  - Submit to exchange                    │  │
│  │  - Track status                          │  │
│  └──────────────┬───────────────────────────┘  │
│                 │                               │
│                 ▼                               │
│  ┌──────────────────────────────────────────┐  │
│  │     Position Tracker                     │  │
│  │  - Update positions on fills             │  │
│  │  - Calculate P&L                         │  │
│  │  - Risk checks                           │  │
│  └──────────────────────────────────────────┘  │
└────────────────────────────────────────────────┘
```

**Event Loop**:
```rust
pub struct LiveEngine {
    strategy: Box<dyn Strategy>,
    exchange: Arc<dyn Exchange>,
    order_manager: OrderManager,
    position_tracker: PositionTracker,
    event_tx: mpsc::Sender<Event>,
    event_rx: mpsc::Receiver<Event>,
}

impl LiveEngine {
    pub async fn run(&mut self) -> Result<()> {
        // Initialize strategy
        let ctx = self.create_context();
        self.strategy.initialize(&ctx).await?;

        // Start market data subscription
        let mut candle_stream = self.exchange.subscribe_candles(&self.symbol).await?;

        // Main event loop
        loop {
            tokio::select! {
                // Market data event
                Some(candle) = candle_stream.next() => {
                    let ctx = self.create_context();
                    self.strategy.on_candle(&candle, &ctx).await;
                    self.process_pending_orders().await?;
                }

                // Internal event (order status, fill, etc.)
                Some(event) = self.event_rx.recv() => {
                    self.handle_event(event).await?;
                }

                // Shutdown signal
                _ = tokio::signal::ctrl_c() => {
                    break;
                }
            }
        }

        // Cleanup
        self.strategy.shutdown().await?;
        Ok(())
    }

    async fn process_pending_orders(&mut self) -> Result<()> {
        for order in self.order_manager.get_pending_orders() {
            // Submit to exchange
            let exchange_order = self.exchange.place_order(order).await?;

            // Track order
            self.order_manager.track_order(exchange_order);
        }
        Ok(())
    }
}
```

### velora-exchange

**Purpose**: Exchange connectivity (REST API + WebSocket).

**Key Components**:
```
velora-exchange/
├── traits.rs          # Exchange trait definitions
├── common/
│   ├── rest.rs        # HTTP client wrapper
│   ├── websocket.rs   # WebSocket client wrapper
│   └── auth.rs        # Authentication (HMAC, wallet)
├── exchanges/
│   ├── binance/       # Binance implementation
│   ├── lighter/       # Lighter (Arbitrum L2)
│   └── paradex/       # Paradex (Starknet L2)
└── lib.rs
```

**Exchange Trait**:
```rust
#[async_trait]
pub trait Exchange: Send + Sync {
    /// Get exchange name
    fn name(&self) -> &str;

    /// Market data interface
    fn market_data(&self) -> Arc<dyn MarketData>;

    /// Trading interface
    fn trading(&self) -> Arc<dyn Trading>;

    /// Account interface
    fn account(&self) -> Arc<dyn Account>;

    /// Streaming interface
    fn streaming(&self) -> Arc<dyn Streaming>;
}

#[async_trait]
pub trait MarketData: Send + Sync {
    async fn get_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<Candle>>;

    async fn get_orderbook(&self, symbol: &Symbol) -> Result<OrderBook>;

    async fn get_recent_trades(
        &self,
        symbol: &Symbol,
        limit: usize,
    ) -> Result<Vec<Trade>>;
}

#[async_trait]
pub trait Trading: Send + Sync {
    async fn place_order(&self, order: NewOrder) -> Result<Order>;
    async fn cancel_order(&self, order_id: &OrderId) -> Result<()>;
    async fn modify_order(&self, modifications: OrderModification) -> Result<Order>;
    async fn get_order(&self, order_id: &OrderId) -> Result<Order>;
    async fn get_open_orders(&self, symbol: Option<&Symbol>) -> Result<Vec<Order>>;
}

#[async_trait]
pub trait Streaming: Send + Sync {
    async fn subscribe_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
    ) -> Result<Pin<Box<dyn Stream<Item = Candle> + Send>>>;

    async fn subscribe_trades(
        &self,
        symbol: &Symbol,
    ) -> Result<Pin<Box<dyn Stream<Item = Trade> + Send>>>;

    async fn subscribe_orderbook(
        &self,
        symbol: &Symbol,
    ) -> Result<Pin<Box<dyn Stream<Item = OrderBook> + Send>>>;
}
```

**WebSocket Implementation Pattern**:
```rust
pub struct BinanceStreaming {
    ws_client: Arc<RwLock<WebSocketClient>>,
    subscriptions: Arc<DashMap<String, mpsc::Sender<String>>>,
}

impl BinanceStreaming {
    async fn subscribe_candles(
        &self,
        symbol: &Symbol,
        timeframe: Timeframe,
    ) -> Result<Pin<Box<dyn Stream<Item = Candle> + Send>>> {
        let stream_name = format!("{}@kline_{}", symbol.as_str(), timeframe);

        // Create channel for this subscription
        let (tx, rx) = mpsc::channel(100);
        self.subscriptions.insert(stream_name.clone(), tx);

        // Subscribe on WebSocket
        self.ws_client.write().await.subscribe(&stream_name).await?;

        // Transform JSON messages to Candle
        let stream = ReceiverStream::new(rx).map(|msg| {
            serde_json::from_str::<BinanceCandle>(&msg)
                .map(|bc| bc.into())  // Convert to Velora Candle
        });

        Ok(Box::pin(stream))
    }
}
```

### velora-risk

**Purpose**: Risk management and position sizing.

**Key Components**:
```
velora-risk/
├── traits.rs          # RiskManager trait
├── position_sizing.rs # Kelly, Fixed Fractional, etc.
├── limits.rs          # Risk limits (max position, max drawdown)
└── lib.rs
```

**Risk Manager Trait**:
```rust
#[async_trait]
pub trait RiskManager: Send + Sync {
    /// Validate an order before submission
    async fn validate_order(
        &self,
        order: &NewOrder,
        ctx: &RiskContext,
    ) -> Result<()>;

    /// Calculate position size based on risk parameters
    fn calculate_position_size(
        &self,
        symbol: &Symbol,
        signal_strength: f64,
        ctx: &RiskContext,
    ) -> Quantity;

    /// Check if should close positions due to risk breach
    fn should_close_positions(&self, ctx: &RiskContext) -> bool;
}

pub struct RiskContext {
    pub portfolio_value: f64,
    pub available_capital: f64,
    pub positions: HashMap<Symbol, Position>,
    pub daily_pnl: f64,
    pub max_drawdown: f64,
}
```

---

## Data Flow

### Backtesting Flow

```
CSV File → DataLoader → Candle Stream
                             │
                             ▼
                    Strategy.on_candle()
                             │
                             ▼
                    Generate Signals/Orders
                             │
                             ▼
                      FillSimulator
                             │
                             ▼
                    PositionTracker.update()
                             │
                             ▼
                     MetricsCalculator
                             │
                             ▼
                      BacktestReport
```

### Live Trading Flow

```
Exchange WebSocket → Candle/Trade Event
                             │
                             ▼
                       Event Queue (MPSC)
                             │
                             ▼
                    Strategy.on_candle()
                             │
                             ▼
                    Generate Signals/Orders
                             │
                             ▼
                      Risk Validation
                             │
                             ▼
                      OrderManager.submit()
                             │
                             ▼
                    Exchange REST API
                             │
                             ▼
                    Order Status Updates
                             │
                             ▼
                    PositionTracker.update()
```

---

## Performance Considerations

### Memory Management

1. **Arena Allocation**: Use arena allocators for short-lived objects in hot paths
2. **Object Pooling**: Reuse Order/Trade objects instead of allocating new ones
3. **Copy-on-Write**: Use `Arc<T>` for shared immutable data
4. **Zero-Copy Parsing**: Deserialize directly into final types without intermediate buffers

### CPU Optimization

1. **SIMD**: Use portable SIMD for TA calculations
   ```rust
   #[cfg(target_feature = "avx2")]
   fn sma_simd(values: &[f64], period: usize) -> Vec<f64> {
       // AVX2 vectorized implementation
   }
   ```

2. **Inline Functions**: Aggressive inlining for hot paths
   ```rust
   #[inline(always)]
   fn calculate_pnl(entry: f64, exit: f64, quantity: f64) -> f64 {
       (exit - entry) * quantity
   }
   ```

3. **Branch Prediction**: Minimize branches in loops
   ```rust
   // Bad: unpredictable branch
   for value in values {
       if value > threshold {
           count += 1;
       }
   }

   // Good: branchless
   for value in values {
       count += (value > threshold) as usize;
   }
   ```

### Network Optimization

1. **Connection Pooling**: Reuse HTTP connections
2. **Batching**: Batch API requests where possible
3. **Compression**: Enable gzip for REST API
4. **WebSocket Ping/Pong**: Keep connections alive

### Concurrency

1. **Async/Await**: Use Tokio for async I/O
2. **Lock-Free Data Structures**: Use `DashMap` instead of `Mutex<HashMap>`
3. **Channel Selection**: Use MPSC for single consumer, broadcast for multiple
4. **Task Spawning**: Spawn tasks for independent work

---

## Testing Strategy

### Unit Tests

Test individual functions in isolation:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sma_calculation() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let result = sma(&values, 3);
        assert_eq!(result, vec![2.0, 3.0, 4.0]);
    }
}
```

### Integration Tests

Test interactions between components:
```rust
#[tokio::test]
async fn test_backtest_end_to_end() {
    let strategy = Box::new(SMACrossover::new(10, 20));
    let data = load_test_data("BTCUSDT_1h.csv");
    let engine = BacktestEngine::new(strategy, data);

    let result = engine.run().await.unwrap();

    assert!(result.total_trades > 0);
    assert!(result.sharpe_ratio > 1.0);
}
```

### Property-Based Tests

Test invariants with random inputs:
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_position_pnl_invariant(
        entry_price in 1.0..1000.0,
        quantity in 0.1..100.0,
        exit_price in 1.0..1000.0,
    ) {
        let pnl = calculate_pnl(entry_price, exit_price, quantity);

        // Invariant: buying low and selling high = profit
        if exit_price > entry_price {
            prop_assert!(pnl > 0.0);
        } else if exit_price < entry_price {
            prop_assert!(pnl < 0.0);
        }
    }
}
```

### Benchmark Tests

Measure performance:
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_sma(c: &mut Criterion) {
    let values: Vec<f64> = (0..10000).map(|i| i as f64).collect();

    c.bench_function("sma_10000_values", |b| {
        b.iter(|| {
            sma(black_box(&values), black_box(20))
        });
    });
}

criterion_group!(benches, bench_sma);
criterion_main!(benches);
```

---

## Deployment Architecture

### Single-Instance Deployment

```
┌─────────────────────────────────────────┐
│         Virtual Machine / Container     │
│                                         │
│  ┌───────────────────────────────────┐  │
│  │      Velora Live Engine           │  │
│  │  - Strategy Execution             │  │
│  │  - Order Management               │  │
│  │  - Position Tracking              │  │
│  └───────────────────────────────────┘  │
│                  │                       │
│  ┌───────────────────────────────────┐  │
│  │      PostgreSQL Database          │  │
│  │  - Trade History                  │  │
│  │  - Position State                 │  │
│  └───────────────────────────────────┘  │
│                  │                       │
│  ┌───────────────────────────────────┐  │
│  │   Prometheus + Grafana            │  │
│  │  - Metrics Collection             │  │
│  │  - Dashboards                     │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
                  │
                  ▼
         Exchange APIs (REST + WS)
```

### High-Availability Deployment

```
┌────────────────────────────────────────────────────┐
│                 Load Balancer                      │
└────────────┬───────────────────────┬────────────────┘
             │                       │
  ┌──────────▼─────────┐  ┌─────────▼────────┐
  │   Instance 1       │  │   Instance 2     │
  │  (Active)          │  │  (Standby)       │
  │  - Live Engine     │  │  - Live Engine   │
  │  - Order Manager   │  │  - Order Manager │
  └──────────┬─────────┘  └─────────┬────────┘
             │                       │
             └───────────┬───────────┘
                         │
              ┌──────────▼──────────┐
              │  Shared Database    │
              │  (PostgreSQL HA)    │
              └─────────────────────┘
```

---

## Security Considerations

1. **API Keys**: Store in environment variables or secrets manager (never in code)
2. **Rate Limiting**: Respect exchange limits to avoid IP bans
3. **Audit Logging**: Log all orders and trades for compliance
4. **Network Security**: Use HTTPS/WSS only, validate TLS certificates
5. **Input Validation**: Sanitize all user inputs and config values

---

## Future Enhancements

See [ROADMAP.md](ROADMAP.md) for detailed plans, but key architectural changes include:

1. **gRPC Services**: Decompose into microservices for scale
2. **Event Sourcing**: Immutable event log for complete replay
3. **WASM Strategies**: Sandboxed strategy execution
4. **GPU Acceleration**: CUDA kernels for ML inference
5. **FPGA Integration**: Ultra-low-latency market data processing

---

**Last Updated**: 2025-10-20
**Version**: v0.0.1
