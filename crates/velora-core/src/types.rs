//! Core data types for the Velora trading platform.
//!
//! This module provides fundamental types used throughout the Velora ecosystem.

use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Type alias for price values with total ordering.
///
/// Uses `OrderedFloat<f64>` to ensure prices can be compared, sorted, and used as map keys.
/// This prevents NaN-related bugs in trading logic.
pub type Price = OrderedFloat<f64>;

/// Type alias for volume/quantity values with total ordering.
///
/// Uses `OrderedFloat<f64>` to ensure volumes can be compared and sorted correctly.
pub type Volume = OrderedFloat<f64>;

/// Represents a trading pair symbol (e.g., "BTC/USD", "ETH/USDT").
///
/// # Examples
///
/// ```
/// use velora_core::Symbol;
///
/// let symbol = Symbol::new("BTC/USD");
/// assert_eq!(symbol.as_str(), "BTC/USD");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Symbol(pub String);

impl Symbol {
    /// Creates a new symbol from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Symbol(s.into())
    }

    /// Returns the symbol as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Symbol {
    fn from(s: &str) -> Self {
        Symbol(s.to_string())
    }
}

impl From<String> for Symbol {
    fn from(s: String) -> Self {
        Symbol(s)
    }
}

impl std::fmt::Display for Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Order side - Buy or Sell.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Side {
    /// Buy order (long position)
    Buy,
    /// Sell order (short position)
    Sell,
}

impl Side {
    /// Returns the opposite side.
    pub fn opposite(&self) -> Side {
        match self {
            Side::Buy => Side::Sell,
            Side::Sell => Side::Buy,
        }
    }
}

/// Order type enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderType {
    /// Market order - execute immediately at current market price
    Market,
    /// Limit order - execute only at specified price or better
    Limit,
    /// Stop-limit order - becomes a limit order when stop price is reached
    StopLimit,
    /// Stop-market order - becomes a market order when stop price is reached
    StopMarket,
}

/// Order status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrderStatus {
    /// Order created but not yet sent to exchange
    Pending,
    /// Order sent and active on the exchange
    Open,
    /// Order partially filled
    PartiallyFilled,
    /// Order completely filled
    Filled,
    /// Order cancelled
    Cancelled,
    /// Order rejected by exchange
    Rejected,
    /// Order expired
    Expired,
}

impl OrderStatus {
    /// Returns true if the order is in an active state.
    pub fn is_active(&self) -> bool {
        matches!(
            self,
            OrderStatus::Pending | OrderStatus::Open | OrderStatus::PartiallyFilled
        )
    }

    /// Returns true if the order is in a terminal state.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            OrderStatus::Filled
                | OrderStatus::Cancelled
                | OrderStatus::Rejected
                | OrderStatus::Expired
        )
    }
}

/// Represents a trading order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Order {
    /// Unique order identifier
    pub id: Uuid,
    /// Trading symbol
    pub symbol: Symbol,
    /// Order side (Buy/Sell)
    pub side: Side,
    /// Order type
    pub order_type: OrderType,
    /// Limit price (None for market orders)
    pub price: Option<Price>,
    /// Order quantity
    pub quantity: Volume,
    /// Filled quantity
    pub filled_quantity: Volume,
    /// Order status
    pub status: OrderStatus,
    /// Order creation timestamp
    pub timestamp: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: DateTime<Utc>,
}

impl Order {
    /// Creates a new market order.
    pub fn new_market(symbol: Symbol, side: Side, quantity: Volume) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Market,
            price: None,
            quantity,
            filled_quantity: OrderedFloat(0.0),
            status: OrderStatus::Pending,
            timestamp: now,
            updated_at: now,
        }
    }

    /// Creates a new limit order.
    pub fn new_limit(symbol: Symbol, side: Side, price: Price, quantity: Volume) -> Self {
        let now = Utc::now();
        Order {
            id: Uuid::new_v4(),
            symbol,
            side,
            order_type: OrderType::Limit,
            price: Some(price),
            quantity,
            filled_quantity: OrderedFloat(0.0),
            status: OrderStatus::Pending,
            timestamp: now,
            updated_at: now,
        }
    }

    /// Returns true if the order is fully filled.
    pub fn is_filled(&self) -> bool {
        self.status == OrderStatus::Filled
    }

    /// Returns true if the order is active.
    pub fn is_active(&self) -> bool {
        self.status.is_active()
    }

    /// Returns the remaining unfilled quantity.
    pub fn remaining_quantity(&self) -> Volume {
        self.quantity - self.filled_quantity
    }

    /// Returns the fill percentage (0.0 to 1.0).
    pub fn fill_percentage(&self) -> f64 {
        if self.quantity == OrderedFloat(0.0) {
            0.0
        } else {
            (self.filled_quantity / self.quantity).into()
        }
    }
}

/// Represents a completed trade (execution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Trade {
    /// Unique trade identifier
    pub id: Uuid,
    /// Associated order ID
    pub order_id: Uuid,
    /// Trading symbol
    pub symbol: Symbol,
    /// Trade side
    pub side: Side,
    /// Execution price
    pub price: Price,
    /// Execution quantity
    pub quantity: Volume,
    /// Trading fee
    pub fee: Price,
    /// Trade execution timestamp
    pub timestamp: DateTime<Utc>,
}

/// Tick data - represents a single price update.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tick {
    /// Trading symbol
    pub symbol: Symbol,
    /// Current price
    pub price: Price,
    /// Volume at this price
    pub volume: Volume,
    /// Timestamp of the tick
    pub timestamp: DateTime<Utc>,
}

/// OHLCV candlestick data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Candle {
    /// Trading symbol
    pub symbol: Symbol,
    /// Opening price
    pub open: Price,
    /// Highest price
    pub high: Price,
    /// Lowest price
    pub low: Price,
    /// Closing price
    pub close: Price,
    /// Total volume
    pub volume: Volume,
    /// Candle timestamp (start of period)
    pub timestamp: DateTime<Utc>,
}

impl Candle {
    /// Returns true if this is a bullish (green) candle.
    pub fn is_bullish(&self) -> bool {
        self.close > self.open
    }

    /// Returns true if this is a bearish (red) candle.
    pub fn is_bearish(&self) -> bool {
        self.close < self.open
    }

    /// Returns the price range (high - low).
    pub fn range(&self) -> Price {
        self.high - self.low
    }

    /// Returns the body size (abs(close - open)).
    pub fn body(&self) -> Price {
        OrderedFloat((self.close - self.open).abs())
    }
}

/// Order book price level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookLevel {
    /// Price level
    pub price: Price,
    /// Total quantity at this price
    pub quantity: Volume,
}

/// Order book snapshot containing bids and asks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBook {
    /// Trading symbol
    pub symbol: Symbol,
    /// Bid levels (sorted highest to lowest)
    pub bids: Vec<BookLevel>,
    /// Ask levels (sorted lowest to highest)
    pub asks: Vec<BookLevel>,
    /// Snapshot timestamp
    pub timestamp: DateTime<Utc>,
}

impl OrderBook {
    /// Returns the best (highest) bid.
    pub fn best_bid(&self) -> Option<&BookLevel> {
        self.bids.first()
    }

    /// Returns the best (lowest) ask.
    pub fn best_ask(&self) -> Option<&BookLevel> {
        self.asks.first()
    }

    /// Returns the bid-ask spread.
    pub fn spread(&self) -> Option<Price> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some(ask.price - bid.price),
            _ => None,
        }
    }

    /// Returns the mid price (average of best bid and ask).
    pub fn mid_price(&self) -> Option<Price> {
        match (self.best_ask(), self.best_bid()) {
            (Some(ask), Some(bid)) => Some((ask.price + bid.price) / 2.0),
            _ => None,
        }
    }
}

/// Trading position with P&L tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Trading symbol
    pub symbol: Symbol,
    /// Position quantity (positive = long, negative = short)
    pub quantity: Volume,
    /// Average entry price
    pub entry_price: Price,
    /// Current market price
    pub current_price: Price,
    /// Unrealized profit/loss
    pub unrealized_pnl: Price,
    /// Realized profit/loss
    pub realized_pnl: Price,
}

impl Position {
    /// Creates a new position.
    pub fn new(symbol: Symbol, quantity: Volume, entry_price: Price) -> Self {
        Position {
            symbol,
            quantity,
            entry_price,
            current_price: entry_price,
            unrealized_pnl: OrderedFloat(0.0),
            realized_pnl: OrderedFloat(0.0),
        }
    }

    /// Updates the current price and recalculates unrealized P&L.
    pub fn update_price(&mut self, price: Price) {
        self.current_price = price;
        self.unrealized_pnl = (price - self.entry_price) * self.quantity.abs();
    }

    /// Returns true if this is a long position.
    pub fn is_long(&self) -> bool {
        self.quantity > OrderedFloat(0.0)
    }

    /// Returns true if this is a short position.
    pub fn is_short(&self) -> bool {
        self.quantity < OrderedFloat(0.0)
    }

    /// Returns true if the position is flat (no position).
    pub fn is_flat(&self) -> bool {
        self.quantity == OrderedFloat(0.0)
    }

    /// Returns the total P&L (unrealized + realized).
    pub fn total_pnl(&self) -> Price {
        self.unrealized_pnl + self.realized_pnl
    }
}

/// Account balance for a specific currency/asset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Currency or asset symbol
    pub currency: String,
    /// Total balance
    pub total: Price,
    /// Available balance (not locked in orders)
    pub available: Price,
    /// Locked balance (in open orders)
    pub locked: Price,
}

impl Balance {
    /// Creates a new balance.
    pub fn new(currency: String, total: Price) -> Self {
        Balance {
            currency,
            total,
            available: total,
            locked: OrderedFloat(0.0),
        }
    }
}

/// Time interval for candlestick data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Interval {
    /// 1 second
    Second1,
    /// 1 minute
    Minute1,
    /// 5 minutes
    Minute5,
    /// 15 minutes
    Minute15,
    /// 30 minutes
    Minute30,
    /// 1 hour
    Hour1,
    /// 4 hours
    Hour4,
    /// 1 day
    Day1,
    /// 1 week
    Week1,
}

impl Interval {
    /// Returns the interval duration in seconds.
    pub fn to_seconds(&self) -> i64 {
        match self {
            Interval::Second1 => 1,
            Interval::Minute1 => 60,
            Interval::Minute5 => 300,
            Interval::Minute15 => 900,
            Interval::Minute30 => 1800,
            Interval::Hour1 => 3600,
            Interval::Hour4 => 14400,
            Interval::Day1 => 86400,
            Interval::Week1 => 604800,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_symbol_creation() {
        let symbol = Symbol::new("BTC/USD");
        assert_eq!(symbol.as_str(), "BTC/USD");
    }

    #[test]
    fn test_side_opposite() {
        assert_eq!(Side::Buy.opposite(), Side::Sell);
        assert_eq!(Side::Sell.opposite(), Side::Buy);
    }

    #[test]
    fn test_order_status() {
        assert!(OrderStatus::Open.is_active());
        assert!(OrderStatus::Filled.is_terminal());
        assert!(!OrderStatus::Filled.is_active());
    }

    #[test]
    fn test_market_order_creation() {
        let order = Order::new_market(Symbol::new("BTC/USD"), Side::Buy, OrderedFloat(0.1));
        assert_eq!(order.order_type, OrderType::Market);
        assert_eq!(order.price, None);
        assert!(order.is_active());
    }

    #[test]
    fn test_limit_order_creation() {
        let order = Order::new_limit(
            Symbol::new("BTC/USD"),
            Side::Buy,
            OrderedFloat(50000.0),
            OrderedFloat(0.1),
        );
        assert_eq!(order.order_type, OrderType::Limit);
        assert_eq!(order.price, Some(OrderedFloat(50000.0)));
    }

    #[test]
    fn test_order_remaining_quantity() {
        let mut order = Order::new_limit(
            Symbol::new("BTC/USD"),
            Side::Buy,
            OrderedFloat(50000.0),
            OrderedFloat(1.0),
        );
        order.filled_quantity = OrderedFloat(0.3);
        assert_eq!(order.remaining_quantity(), OrderedFloat(0.7));
    }

    #[test]
    fn test_candle_properties() {
        let candle = Candle {
            symbol: Symbol::new("BTC/USD"),
            open: OrderedFloat(50000.0),
            high: OrderedFloat(51000.0),
            low: OrderedFloat(49000.0),
            close: OrderedFloat(50500.0),
            volume: OrderedFloat(100.0),
            timestamp: Utc::now(),
        };

        assert!(candle.is_bullish());
        assert!(!candle.is_bearish());
        assert_eq!(candle.range(), OrderedFloat(2000.0));
    }

    #[test]
    fn test_orderbook_mid_price() {
        let orderbook = OrderBook {
            symbol: Symbol::new("BTC/USD"),
            bids: vec![BookLevel {
                price: OrderedFloat(50000.0),
                quantity: OrderedFloat(10.0),
            }],
            asks: vec![BookLevel {
                price: OrderedFloat(50100.0),
                quantity: OrderedFloat(5.0),
            }],
            timestamp: Utc::now(),
        };

        assert_eq!(orderbook.mid_price(), Some(OrderedFloat(50050.0)));
        assert_eq!(orderbook.spread(), Some(OrderedFloat(100.0)));
    }

    #[test]
    fn test_position_pnl() {
        let mut position = Position::new(
            Symbol::new("BTC/USD"),
            OrderedFloat(1.0),
            OrderedFloat(50000.0),
        );

        position.update_price(OrderedFloat(51000.0));
        assert_eq!(position.unrealized_pnl, OrderedFloat(1000.0));
        assert!(position.is_long());
    }

    #[test]
    fn test_interval_to_seconds() {
        assert_eq!(Interval::Minute1.to_seconds(), 60);
        assert_eq!(Interval::Hour1.to_seconds(), 3600);
        assert_eq!(Interval::Day1.to_seconds(), 86400);
    }
}
