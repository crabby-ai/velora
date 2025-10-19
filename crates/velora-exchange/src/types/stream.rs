//! Stream event types.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Order update event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderUpdate {
    /// Order information
    pub order: Order,

    /// Update timestamp
    pub timestamp: DateTime<Utc>,
}

/// Balance update event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdate {
    /// Asset name
    pub asset: String,

    /// Free balance
    pub free: Decimal,

    /// Locked balance
    pub locked: Decimal,

    /// Update timestamp
    pub timestamp: DateTime<Utc>,
}

/// Position update event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionUpdate {
    /// Position information
    pub position: Position,

    /// Update timestamp
    pub timestamp: DateTime<Utc>,
}

/// User data stream events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UserDataEvent {
    /// Order update event
    OrderUpdate(OrderUpdate),

    /// Balance update event
    BalanceUpdate(BalanceUpdate),

    /// Position update event
    PositionUpdate(PositionUpdate),

    /// Account configuration update
    AccountUpdate(AccountInfo),
}
