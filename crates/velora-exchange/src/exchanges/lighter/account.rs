//! Lighter account implementation

use async_trait::async_trait;
use std::sync::Arc;

use crate::{
    auth::EvmWalletAuth,
    common::{RateLimiter, RestClient},
    traits::Account,
    types::{AccountInfo, Balance, Position, Result, Symbol, TradeExecution},
};

/// Lighter account component
pub struct LighterAccount {
    rest_client: Arc<RestClient>,
    rate_limiter: Arc<RateLimiter>,
    auth: Option<EvmWalletAuth>,
}

impl LighterAccount {
    pub fn new(
        rest_client: Arc<RestClient>,
        rate_limiter: Arc<RateLimiter>,
        auth: Option<EvmWalletAuth>,
    ) -> Self {
        Self {
            rest_client,
            rate_limiter,
            auth,
        }
    }
}

#[async_trait]
impl Account for LighterAccount {
    async fn get_account_info(&self) -> Result<AccountInfo> {
        self.rate_limiter.wait().await;

        // TODO: Implement account info retrieval
        todo!("Implement Lighter get_account_info")
    }

    async fn get_balances(&self) -> Result<Vec<Balance>> {
        self.rate_limiter.wait().await;

        // TODO: Implement balances retrieval
        todo!("Implement Lighter get_balances")
    }

    async fn get_balance(&self, asset: &str) -> Result<Balance> {
        self.rate_limiter.wait().await;

        // TODO: Implement single balance retrieval
        todo!("Implement Lighter get_balance for {}", asset)
    }

    async fn get_positions(&self) -> Result<Vec<Position>> {
        self.rate_limiter.wait().await;

        // TODO: Implement positions retrieval
        todo!("Implement Lighter get_positions")
    }

    async fn get_position(&self, symbol: &Symbol) -> Result<Option<Position>> {
        self.rate_limiter.wait().await;

        // TODO: Implement single position retrieval
        todo!("Implement Lighter get_position for {}", symbol)
    }

    async fn get_trade_history(
        &self,
        symbol: Option<&Symbol>,
        limit: Option<usize>,
    ) -> Result<Vec<TradeExecution>> {
        self.rate_limiter.wait().await;

        // TODO: Implement trade history retrieval
        todo!("Implement Lighter get_trade_history")
    }
}
