//! Authentication strategies for different exchange types
//!
//! This module provides authentication mechanisms for:
//! - Centralized Exchanges (CEX): API Key authentication
//! - zkRollup DEX (Lighter): EVM wallet signing
//! - Starknet L2 DEX (Paradex): Starknet wallet signing

pub mod api_key;
pub mod evm_wallet;
pub mod starknet_wallet;

pub use api_key::ApiKeyAuth;
pub use evm_wallet::EvmWalletAuth;
pub use starknet_wallet::StarknetWalletAuth;

use serde::{Deserialize, Serialize};

/// Authentication configuration enum supporting different auth strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuthConfig {
    /// API Key authentication for centralized exchanges
    ApiKey(ApiKeyAuth),

    /// EVM wallet authentication for zkRollup DEX (Lighter)
    EvmWallet(EvmWalletAuth),

    /// Starknet wallet authentication for L2 DEX (Paradex)
    StarknetWallet(StarknetWalletAuth),

    /// No authentication (for public endpoints only)
    None,
}

impl AuthConfig {
    /// Create API key auth config
    pub fn api_key(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self::ApiKey(ApiKeyAuth::new(api_key, api_secret))
    }

    /// Create EVM wallet auth config
    #[cfg(feature = "lighter")]
    pub fn evm_wallet(private_key: impl Into<String>) -> Self {
        Self::EvmWallet(EvmWalletAuth::new(private_key))
    }

    /// Create Starknet wallet auth config
    #[cfg(feature = "paradex")]
    pub fn starknet_wallet(private_key: impl Into<String>) -> Self {
        Self::StarknetWallet(StarknetWalletAuth::new(private_key))
    }

    /// Check if authentication is configured
    pub fn is_authenticated(&self) -> bool {
        !matches!(self, Self::None)
    }
}
