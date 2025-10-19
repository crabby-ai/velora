//! Common types used across all exchanges.

use serde::{Deserialize, Serialize};
use std::fmt;

// Re-export from dependencies
pub use rust_decimal::Decimal;

// Re-export from velora-core
pub use velora_core::{Interval, OrderType, Price, Side, Symbol};

/// Exchange type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExchangeType {
    /// Centralized exchange (Binance, Coinbase)
    CEX,
    /// zkRollup DEX (Lighter)
    #[serde(rename = "dex_zk")]
    DexZk,
    /// Layer 2 DEX (Paradex on Starknet)
    #[serde(rename = "dex_l2")]
    DexL2,
    /// Layer 1 DEX (Uniswap - future)
    #[serde(rename = "dex_l1")]
    DexL1,
}

/// Market status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarketStatus {
    /// Market is open for trading
    Trading,
    /// Pre-trading period
    PreTrading,
    /// Post-trading period
    PostTrading,
    /// Trading halted
    Halted,
    /// Market break
    Break,
    /// Market closed
    Closed,
}

/// Time in force for orders
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeInForce {
    /// Good Till Cancel - order remains until filled or cancelled
    #[serde(rename = "GTC")]
    GoodTilCancel,
    /// Immediate Or Cancel - fill immediately or cancel
    #[serde(rename = "IOC")]
    ImmediateOrCancel,
    /// Fill Or Kill - fill completely or cancel
    #[serde(rename = "FOK")]
    FillOrKill,
    /// Good Till Crossing - post-only order
    #[serde(rename = "GTX")]
    GoodTilCrossing,
}

impl fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TimeInForce::GoodTilCancel => write!(f, "GTC"),
            TimeInForce::ImmediateOrCancel => write!(f, "IOC"),
            TimeInForce::FillOrKill => write!(f, "FOK"),
            TimeInForce::GoodTilCrossing => write!(f, "GTX"),
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderStatus {
    /// Order is open and active
    Open,
    /// Order is partially filled
    PartiallyFilled,
    /// Order is completely filled
    Filled,
    /// Order was cancelled
    Cancelled,
    /// Order was rejected
    Rejected,
    /// Order expired
    Expired,
    /// Order is pending
    Pending,
}

/// Execution type for order updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExecutionType {
    /// New order created
    New,
    /// Order cancelled
    Cancelled,
    /// Order replaced/modified
    Replaced,
    /// Order rejected
    Rejected,
    /// Order executed (trade)
    Trade,
    /// Order expired
    Expired,
}

/// Position side for perpetuals/futures
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum PositionSide {
    /// Long position
    Long,
    /// Short position
    Short,
    /// Both (hedge mode)
    Both,
}

/// Margin type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MarginType {
    /// Cross margin
    Cross,
    /// Isolated margin
    Isolated,
}

/// Option type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OptionType {
    /// Call option
    Call,
    /// Put option
    Put,
}
