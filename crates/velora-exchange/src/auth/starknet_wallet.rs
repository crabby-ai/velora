//! Starknet wallet authentication for L2 DEX (Paradex)

use serde::{Deserialize, Serialize};

#[cfg(feature = "paradex")]
use starknet::signers::SigningKey as StarknetSigningKey;

#[cfg(feature = "paradex")]
use starknet_crypto::Felt as FieldElement;

/// Starknet wallet authentication for L2 exchanges like Paradex
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StarknetWalletAuth {
    /// Private key in hex format (without 0x prefix)
    #[serde(skip_serializing)]
    private_key: String,
}

impl StarknetWalletAuth {
    /// Create new Starknet wallet authentication
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

    /// Get the signing key (only available with paradex feature)
    #[cfg(feature = "paradex")]
    pub fn get_signing_key(&self) -> Result<StarknetSigningKey, Box<dyn std::error::Error>> {
        let key_bytes = hex::decode(&self.private_key)?;
        let key_array: [u8; 32] = key_bytes.try_into().map_err(|_| "Invalid key length")?;
        let field_element = FieldElement::from_bytes_be(&key_array);
        Ok(StarknetSigningKey::from_secret_scalar(field_element))
    }

    /// Get the public key
    #[cfg(feature = "paradex")]
    pub fn public_key(&self) -> Result<FieldElement, Box<dyn std::error::Error>> {
        let signing_key = self.get_signing_key()?;
        Ok(signing_key.verifying_key().scalar())
    }

    /// Get the Starknet address
    #[cfg(feature = "paradex")]
    pub fn address(&self) -> Result<FieldElement, Box<dyn std::error::Error>> {
        // Address derivation for Starknet
        // This is simplified - actual implementation may need account contract deployment
        self.public_key()
    }

    /// Sign a message hash
    #[cfg(feature = "paradex")]
    pub fn sign_hash(
        &self,
        message_hash: FieldElement,
    ) -> Result<(FieldElement, FieldElement), Box<dyn std::error::Error>> {
        let signing_key = self.get_signing_key()?;
        let signature = signing_key.sign(&message_hash)?;
        Ok((signature.r, signature.s))
    }

    /// Sign a message (will be hashed first)
    #[cfg(feature = "paradex")]
    pub fn sign_message(
        &self,
        message: &[u8],
    ) -> Result<(FieldElement, FieldElement), Box<dyn std::error::Error>> {
        // Convert message to field element (simplified)
        // In production, you'd use proper message hashing with poseidon or pedersen
        let mut padded = [0u8; 32];
        let len = message.len().min(32);
        padded[32 - len..].copy_from_slice(&message[..len]);
        let message_hash = FieldElement::from_bytes_be(&padded);
        self.sign_hash(message_hash)
    }

    /// Get the private key (use with extreme caution)
    pub fn private_key(&self) -> &str {
        &self.private_key
    }
}

// Stub implementation when paradex feature is not enabled
#[cfg(not(feature = "paradex"))]
impl StarknetWalletAuth {
    /// Sign a message (stub - requires paradex feature)
    pub fn sign_message(
        &self,
        _message: &[u8],
    ) -> Result<(String, String), Box<dyn std::error::Error>> {
        Err("Starknet wallet signing requires the 'paradex' feature".into())
    }
}

#[cfg(all(test, feature = "paradex"))]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        // Test with 0x prefix
        let auth1 = StarknetWalletAuth::new(
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        );
        assert_eq!(
            auth1.private_key(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );

        // Test without 0x prefix
        let auth2 = StarknetWalletAuth::new(
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
        );
        assert_eq!(
            auth2.private_key(),
            "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
        );
    }
}
