//! Instrument type definitions.

use super::*;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Type of financial instrument
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum InstrumentType {
    /// Spot trading
    Spot,
    /// Perpetual futures (no expiry)
    Perpetual,
    /// Delivery futures (with expiry)
    Futures,
    /// Options contracts
    Options,
}

/// Instrument-specific information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum InstrumentInfo {
    /// Spot trading information
    Spot(SpotInfo),
    /// Perpetual futures information
    Perpetual(PerpetualInfo),
    /// Delivery futures information
    Futures(FuturesInfo),
    /// Options information
    Options(OptionsInfo),
}

/// Spot trading information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotInfo {
    /// Maker fee rate
    pub maker_fee: Decimal,
    /// Taker fee rate
    pub taker_fee: Decimal,
}

/// Perpetual futures information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerpetualInfo {
    /// Funding interval (e.g., 8 hours)
    #[serde(with = "duration_serde")]
    pub funding_interval: Duration,
    /// Maximum leverage allowed
    pub max_leverage: u32,
    /// Maker fee rate
    pub maker_fee: Decimal,
    /// Taker fee rate
    pub taker_fee: Decimal,
    /// Initial margin requirement
    pub initial_margin: Option<Decimal>,
    /// Maintenance margin requirement
    pub maintenance_margin: Option<Decimal>,
}

/// Delivery futures information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuturesInfo {
    /// Contract delivery/expiry date
    pub delivery_date: DateTime<Utc>,
    /// Maximum leverage allowed
    pub max_leverage: u32,
    /// Maker fee rate
    pub maker_fee: Decimal,
    /// Taker fee rate
    pub taker_fee: Decimal,
}

/// Options contract information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptionsInfo {
    /// Option expiry date
    pub expiry_date: DateTime<Utc>,
    /// Strike price
    pub strike_price: Price,
    /// Option type (Call/Put)
    pub option_type: OptionType,
    /// Underlying asset
    pub underlying: String,
}

/// Funding rate information for perpetuals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    /// Symbol
    pub symbol: Symbol,
    /// Current funding rate
    pub rate: Decimal,
    /// Next funding time
    pub next_funding_time: DateTime<Utc>,
    /// Time of this funding rate
    pub timestamp: DateTime<Utc>,
}

// Helper module for Duration serialization
mod duration_serde {
    use serde::{Deserialize, Deserializer, Serializer};
    use std::time::Duration;

    pub fn serialize<S>(duration: &Duration, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_u64(duration.as_secs())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(Duration::from_secs(secs))
    }
}
