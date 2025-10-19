//! Core Exchange trait.

use crate::types::*;
use async_trait::async_trait;

/// Core exchange abstraction.
///
/// This trait provides a unified interface for all exchanges,
/// regardless of whether they are CEX or DEX, and regardless
/// of the instrument type (spot, perpetuals, futures, options).
#[async_trait]
pub trait Exchange: Send + Sync {
    // === Metadata ===

    /// Get exchange name (e.g., "binance", "lighter", "paradex")
    fn name(&self) -> &str;

    /// Get exchange type (CEX, DEX_ZK, DEX_L2, etc.)
    fn exchange_type(&self) -> ExchangeType;

    /// Get supported instrument types
    fn supported_instruments(&self) -> &[InstrumentType];

    // === Connection Management ===

    /// Connect to the exchange
    async fn connect(&mut self) -> Result<()>;

    /// Disconnect from the exchange
    async fn disconnect(&mut self) -> Result<()>;

    /// Check if connected
    fn is_connected(&self) -> bool;

    // === Component Access ===

    /// Get market data interface
    fn market_data(&self) -> &dyn super::MarketData;

    /// Get trading interface
    fn trading(&self) -> &dyn super::Trading;

    /// Get account interface
    fn account(&self) -> &dyn super::Account;

    /// Get streaming interface
    fn streaming(&self) -> &dyn super::Streaming;
}
