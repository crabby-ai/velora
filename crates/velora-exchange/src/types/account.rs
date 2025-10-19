//! Account and balance types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Account information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountInfo {
    /// Account type
    pub account_type: AccountType,

    /// Can trade
    pub can_trade: bool,

    /// Can withdraw
    pub can_withdraw: bool,

    /// Can deposit
    pub can_deposit: bool,

    /// Maker commission rate
    pub maker_commission: Decimal,

    /// Taker commission rate
    pub taker_commission: Decimal,

    /// Last update time
    pub update_time: DateTime<Utc>,
}

/// Account type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AccountType {
    /// Spot trading account
    Spot,
    /// Margin trading account
    Margin,
    /// Isolated margin account
    IsolatedMargin,
    /// Futures trading account
    Futures,
    /// Options trading account
    Options,
}

/// Asset balance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    /// Asset symbol (e.g., "BTC", "USDT")
    pub asset: String,

    /// Free balance (available for trading)
    pub free: Decimal,

    /// Locked balance (in orders)
    pub locked: Decimal,
}

impl Balance {
    /// Get total balance (free + locked)
    pub fn total(&self) -> Decimal {
        self.free + self.locked
    }
}

/// Position (for perpetuals/futures)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    /// Symbol
    pub symbol: Symbol,

    /// Position side
    pub side: PositionSide,

    /// Position quantity
    pub quantity: Decimal,

    /// Entry price
    pub entry_price: Price,

    /// Mark price (current market price)
    pub mark_price: Price,

    /// Liquidation price
    #[serde(skip_serializing_if = "Option::is_none")]
    pub liquidation_price: Option<Price>,

    /// Leverage
    pub leverage: u32,

    /// Unrealized PnL
    pub unrealized_pnl: Decimal,

    /// Realized PnL
    pub realized_pnl: Decimal,

    /// Margin used for this position
    pub margin: Decimal,

    /// Margin type
    pub margin_type: MarginType,

    /// Last update time
    pub update_time: DateTime<Utc>,
}

impl Position {
    /// Check if position is long
    pub fn is_long(&self) -> bool {
        matches!(self.side, PositionSide::Long)
    }

    /// Check if position is short
    pub fn is_short(&self) -> bool {
        matches!(self.side, PositionSide::Short)
    }

    /// Get ROI percentage
    pub fn roi_percentage(&self) -> f64 {
        if self.margin.is_zero() {
            0.0
        } else {
            ((self.unrealized_pnl / self.margin) * Decimal::new(100, 0))
                .try_into()
                .unwrap_or(0.0)
        }
    }
}
