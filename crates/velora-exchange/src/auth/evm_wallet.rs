//! EVM wallet authentication for zkRollup DEX (Lighter)

use serde::{Deserialize, Serialize};

#[cfg(feature = "lighter")]
use ethers::{
    signers::{LocalWallet, Signer},
    types::{Address, Signature},
};

/// EVM wallet authentication for zkRollup exchanges like Lighter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvmWalletAuth {
    /// Private key in hex format (without 0x prefix)
    #[serde(skip_serializing)]
    private_key: String,
}

impl EvmWalletAuth {
    /// Create new EVM wallet authentication
    ///
    /// # Arguments
    /// * `private_key` - Private key as hex string (with or without 0x prefix)
    pub fn new(private_key: impl Into<String>) -> Self {
        let mut key = private_key.into();
        // Remove 0x prefix if present
        if key.starts_with("0x") || key.starts_with("0X") {
            key = key[2..].to_string();
        }
        Self { private_key: key }
    }

    /// Get the wallet signer (only available with lighter feature)
    #[cfg(feature = "lighter")]
    pub fn get_wallet(&self) -> Result<LocalWallet, Box<dyn std::error::Error>> {
        let wallet = self.private_key.parse::<LocalWallet>()?;
        Ok(wallet)
    }

    /// Get the Ethereum address
    #[cfg(feature = "lighter")]
    pub fn address(&self) -> Result<Address, Box<dyn std::error::Error>> {
        let wallet = self.get_wallet()?;
        Ok(wallet.address())
    }

    /// Sign a message using EIP-191 personal sign
    #[cfg(feature = "lighter")]
    pub async fn sign_message(
        &self,
        message: &[u8],
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let wallet = self.get_wallet()?;
        let signature = wallet.sign_message(message).await?;
        Ok(signature)
    }

    /// Sign typed data (EIP-712)
    #[cfg(feature = "lighter")]
    pub async fn sign_typed_data(
        &self,
        data: &ethers::types::transaction::eip712::TypedData,
    ) -> Result<Signature, Box<dyn std::error::Error>> {
        let wallet = self.get_wallet()?;
        let signature = wallet.sign_typed_data(data).await?;
        Ok(signature)
    }

    /// Get the private key (use with extreme caution)
    pub fn private_key(&self) -> &str {
        &self.private_key
    }
}

// Stub implementation when lighter feature is not enabled
#[cfg(not(feature = "lighter"))]
impl EvmWalletAuth {
    /// Sign a message (stub - requires lighter feature)
    pub async fn sign_message(
        &self,
        _message: &[u8],
    ) -> Result<String, Box<dyn std::error::Error>> {
        Err("EVM wallet signing requires the 'lighter' feature".into())
    }
}

#[cfg(all(test, feature = "lighter"))]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        // Test with 0x prefix
        let auth1 = EvmWalletAuth::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        );
        assert_eq!(
            auth1.private_key(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );

        // Test without 0x prefix
        let auth2 =
            EvmWalletAuth::new("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef");
        assert_eq!(
            auth2.private_key(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }
}
