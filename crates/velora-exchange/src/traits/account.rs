//! Account trait.

use crate::types::*;
use async_trait::async_trait;

/// Account interface for balances and positions.
#[async_trait]
pub trait Account: Send + Sync {
    // === Account Information ===

    /// Get account information
    async fn get_account_info(&self) -> Result<AccountInfo>;

    // === Balances (Spot & Perpetuals) ===

    /// Get all balances
    async fn get_balances(&self) -> Result<Vec<Balance>>;

    /// Get balance for specific asset
    async fn get_balance(&self, asset: &str) -> Result<Balance>;

    // === Positions (Perpetuals/Futures only) ===

    /// Get all positions (returns empty vec for spot trading)
    async fn get_positions(&self) -> Result<Vec<Position>>;

    /// Get position for specific symbol (returns None for spot or if no position)
    async fn get_position(&self, symbol: &Symbol) -> Result<Option<Position>>;

    // === Trade History ===

    /// Get trade execution history
    async fn get_trade_history(
        &self,
        symbol: Option<&Symbol>,
        limit: Option<usize>,
    ) -> Result<Vec<TradeExecution>>;
}
