//! Common utilities for exchange implementations
//!
//! This module provides shared components used across different exchange integrations:
//! - Rate limiting
//! - HTTP client wrapper
//! - WebSocket connection management
//! - Reconnection logic
//! - Exchange factory

pub mod factory;
pub mod rate_limiter;
pub mod rest_client;
pub mod websocket;

pub use factory::ExchangeFactory;
pub use rate_limiter::RateLimiter;
pub use rest_client::RestClient;
pub use websocket::{WebSocketClient, WebSocketMessage};
