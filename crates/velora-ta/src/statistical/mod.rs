//! Statistical indicators

#![allow(missing_docs)]

pub mod correlation;
pub mod linear_regression;
pub mod zscore;

pub use correlation::Correlation;
pub use linear_regression::LinearRegression;
pub use zscore::ZScore;
