//! Factory for creating exchange instances

use crate::{
    auth::AuthConfig,
    traits::Exchange,
    types::{ExchangeError, ExchangeType, Result},
};

/// Factory for creating exchange instances
pub struct ExchangeFactory;

impl ExchangeFactory {
    /// Create an exchange instance
    ///
    /// # Arguments
    /// * `exchange_type` - Type of exchange (CEX, DexZk, DexL2, DexL1)
    /// * `name` - Exchange name (e.g., "binance", "lighter", "paradex")
    /// * `auth` - Authentication configuration
    ///
    /// # Returns
    /// A boxed trait object implementing the Exchange trait
    ///
    /// # Example
    /// ```rust,no_run
    /// use velora_exchange::{ExchangeFactory, ExchangeType, AuthConfig};
    ///
    /// #[tokio::main]
    /// async fn main() -> Result<(), Box<dyn std::error::Error>> {
    ///     let auth = AuthConfig::api_key("my_key", "my_secret");
    ///     let exchange = ExchangeFactory::create(
    ///         ExchangeType::CEX,
    ///         "binance",
    ///         auth,
    ///     ).await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn create(
        exchange_type: ExchangeType,
        name: &str,
        auth: AuthConfig,
    ) -> Result<Box<dyn Exchange>> {
        match (exchange_type, name.to_lowercase().as_str()) {
            #[cfg(feature = "lighter")]
            (ExchangeType::DexZk, "lighter") => {
                use crate::exchanges::lighter::LighterExchange;
                let exchange = LighterExchange::new(auth).await?;
                Ok(Box::new(exchange))
            }

            #[cfg(feature = "paradex")]
            (ExchangeType::DexL2, "paradex") => {
                use crate::exchanges::paradex::ParadexExchange;
                let exchange = ParadexExchange::new(auth).await?;
                Ok(Box::new(exchange))
            }

            _ => Err(ExchangeError::UnsupportedExchange(format!(
                "Exchange '{name}' of type '{exchange_type:?}' is not supported or not enabled. \
                     Check that the appropriate feature flag is enabled."
            ))),
        }
    }

    /// List all available exchanges based on enabled features
    pub fn list_available() -> Vec<(&'static str, ExchangeType)> {
        let mut exchanges = Vec::new();

        #[cfg(feature = "binance")]
        exchanges.push(("binance", ExchangeType::CEX));

        #[cfg(feature = "lighter")]
        exchanges.push(("lighter", ExchangeType::DexZk));

        #[cfg(feature = "paradex")]
        exchanges.push(("paradex", ExchangeType::DexL2));

        exchanges
    }

    /// Check if an exchange is available
    pub fn is_available(name: &str) -> bool {
        Self::list_available()
            .iter()
            .any(|(n, _)| n.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_available() {
        let available = ExchangeFactory::list_available();

        // At minimum, lighter and paradex should be available (default features)
        #[cfg(feature = "lighter")]
        assert!(available.iter().any(|(name, _)| *name == "lighter"));

        #[cfg(feature = "paradex")]
        assert!(available.iter().any(|(name, _)| *name == "paradex"));
    }

    #[test]
    fn test_is_available() {
        #[cfg(feature = "lighter")]
        assert!(ExchangeFactory::is_available("lighter"));

        #[cfg(feature = "paradex")]
        assert!(ExchangeFactory::is_available("paradex"));

        assert!(!ExchangeFactory::is_available("nonexistent_exchange"));
    }
}
