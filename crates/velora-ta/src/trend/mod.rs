//! Trend indicators for identifying market direction.
//!
//! Trend indicators help identify the direction and strength of price movements.

pub mod adx;
pub mod aroon;
pub mod dema;
pub mod ema;
pub mod hma;
pub mod kama;
pub mod parabolic_sar;
pub mod sma;
pub mod smma;
pub mod supertrend;
pub mod tema;
pub mod vortex;
pub mod vwma;
pub mod wma;

pub use adx::ADX;
pub use aroon::Aroon;
pub use dema::DEMA;
pub use ema::EMA;
pub use hma::HMA;
pub use kama::KAMA;
pub use parabolic_sar::ParabolicSAR;
pub use sma::SMA;
pub use smma::SMMA;
pub use supertrend::SuperTrend;
pub use tema::TEMA;
pub use vortex::Vortex;
pub use vwma::VWMA;
pub use wma::WMA;
