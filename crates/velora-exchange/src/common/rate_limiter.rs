//! Rate limiting utilities for exchange API requests

use governor::{
    clock::DefaultClock,
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use std::num::NonZeroU32;
use std::time::Duration;

/// Rate limiter for exchange API requests
///
/// Prevents exceeding exchange rate limits by throttling outbound requests.
pub struct RateLimiter {
    /// Internal governor-based rate limiter
    limiter: GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,

    /// Maximum requests per period
    max_requests: u32,

    /// Time period for rate limit
    period: Duration,
}

impl RateLimiter {
    /// Create a new rate limiter
    ///
    /// # Arguments
    /// * `max_requests` - Maximum number of requests allowed
    /// * `period` - Time period for the limit (e.g., Duration::from_secs(60) for per-minute)
    ///
    /// # Example
    /// ```rust
    /// use velora_exchange::RateLimiter;
    /// use std::time::Duration;
    ///
    /// // Allow 1200 requests per minute
    /// let limiter = RateLimiter::new(1200, Duration::from_secs(60));
    /// ```
    pub fn new(max_requests: u32, period: Duration) -> Self {
        let quota = Quota::with_period(period)
            .expect("Valid period")
            .allow_burst(NonZeroU32::new(max_requests).expect("Non-zero max_requests"));

        Self {
            limiter: GovernorRateLimiter::direct(quota),
            max_requests,
            period,
        }
    }

    /// Create a rate limiter for Binance (1200 requests/min for general endpoints)
    pub fn binance() -> Self {
        Self::new(1200, Duration::from_secs(60))
    }

    /// Create a rate limiter for Binance order endpoints (more restrictive: 100 orders/10s)
    pub fn binance_orders() -> Self {
        Self::new(100, Duration::from_secs(10))
    }

    /// Create a rate limiter for Lighter
    pub fn lighter() -> Self {
        // Lighter specific rate limits - adjust as needed
        Self::new(100, Duration::from_secs(1))
    }

    /// Create a rate limiter for Paradex
    pub fn paradex() -> Self {
        // Paradex specific rate limits - adjust as needed
        Self::new(50, Duration::from_secs(1))
    }

    /// Wait until a request can be made (async)
    ///
    /// This will block until the rate limiter allows a request to proceed.
    pub async fn wait(&self) {
        self.limiter.until_ready().await;
    }

    /// Check if a request can be made immediately (non-blocking)
    ///
    /// Returns `true` if the request can proceed, `false` otherwise.
    pub fn check(&self) -> bool {
        self.limiter.check().is_ok()
    }

    /// Get the maximum requests per period
    pub fn max_requests(&self) -> u32 {
        self.max_requests
    }

    /// Get the rate limit period
    pub fn period(&self) -> Duration {
        self.period
    }
}

impl std::fmt::Debug for RateLimiter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RateLimiter")
            .field("max_requests", &self.max_requests)
            .field("period", &self.period)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = RateLimiter::new(10, Duration::from_secs(1));
        assert_eq!(limiter.max_requests(), 10);
        assert_eq!(limiter.period(), Duration::from_secs(1));
    }

    #[test]
    fn test_binance_limiter() {
        let limiter = RateLimiter::binance();
        assert_eq!(limiter.max_requests(), 1200);
        assert_eq!(limiter.period(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let limiter = RateLimiter::new(2, Duration::from_secs(1));

        // First two requests should succeed immediately
        assert!(limiter.check());
        limiter.wait().await;
        assert!(limiter.check());
        limiter.wait().await;

        // Third request should be rate limited
        assert!(!limiter.check());
    }
}
