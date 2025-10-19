//! Candlestick pattern recognition.

#![allow(missing_docs)]

pub mod detector;
pub mod double;
pub mod single;
pub mod triple;

pub use detector::{PatternDetector, PatternSignal};
pub use double::*;
pub use single::*;
pub use triple::*;
