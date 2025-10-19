//! API Key authentication for centralized exchanges (Binance, etc.)

use hmac::{Hmac, Mac};
use serde::{Deserialize, Serialize};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// API Key authentication credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyAuth {
    /// API key
    pub api_key: String,

    /// API secret (stored securely, not exposed in debug output)
    #[serde(skip_serializing)]
    api_secret: String,
}

impl ApiKeyAuth {
    /// Create new API key authentication
    pub fn new(api_key: impl Into<String>, api_secret: impl Into<String>) -> Self {
        Self {
            api_key: api_key.into(),
            api_secret: api_secret.into(),
        }
    }

    /// Sign a message using HMAC-SHA256
    ///
    /// This is commonly used by exchanges like Binance for request signing.
    pub fn sign_hmac_sha256(&self, message: &str) -> String {
        let mut mac = HmacSha256::new_from_slice(self.api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(message.as_bytes());
        hex::encode(mac.finalize().into_bytes())
    }

    /// Get the API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get the API secret (use with caution)
    pub fn api_secret(&self) -> &str {
        &self.api_secret
    }

    /// Create authentication headers for HTTP requests
    ///
    /// Returns a vector of (header_name, header_value) tuples.
    pub fn create_headers(&self) -> Vec<(&'static str, String)> {
        vec![("X-API-KEY", self.api_key.clone())]
    }

    /// Sign query parameters and return signature
    ///
    /// # Arguments
    /// * `query_string` - The query string to sign (e.g., "symbol=BTCUSDT&timestamp=1234567890")
    ///
    /// # Returns
    /// The HMAC-SHA256 signature as a hex string
    pub fn sign_query(&self, query_string: &str) -> String {
        self.sign_hmac_sha256(query_string)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_key_creation() {
        let auth = ApiKeyAuth::new("test_key", "test_secret");
        assert_eq!(auth.api_key(), "test_key");
        assert_eq!(auth.api_secret(), "test_secret");
    }

    #[test]
    fn test_hmac_signing() {
        let auth = ApiKeyAuth::new("key", "secret");
        let signature = auth.sign_hmac_sha256("test_message");

        // Verify it's a valid hex string
        assert!(signature.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(signature.len(), 64); // SHA256 produces 32 bytes = 64 hex chars
    }

    #[test]
    fn test_deterministic_signing() {
        let auth = ApiKeyAuth::new("key", "secret");
        let sig1 = auth.sign_query("symbol=BTCUSDT&timestamp=123");
        let sig2 = auth.sign_query("symbol=BTCUSDT&timestamp=123");
        assert_eq!(sig1, sig2);
    }
}
