//! Base strategy trait and implementations.

use crate::context::StrategyContext;
use crate::errors::StrategyResult;
use crate::types::{Signal, StrategyConfig, StrategyState};
use async_trait::async_trait;
use velora_core::types::{Candle, Tick, Trade};

/// Base trait for all trading strategies
#[async_trait]
pub trait Strategy: Send + Sync {
    /// Get the strategy name
    fn name(&self) -> &str;

    /// Get the strategy version
    fn version(&self) -> &str {
        "0.1.0"
    }

    /// Get the strategy configuration
    fn config(&self) -> &StrategyConfig;

    /// Get the current strategy state
    fn state(&self) -> StrategyState;

    /// Initialize the strategy with historical data
    ///
    /// This is called once before the strategy starts running.
    /// Use this to load historical data, warm up indicators, etc.
    async fn initialize(&mut self, ctx: &StrategyContext) -> StrategyResult<()> {
        let _ = ctx;
        Ok(())
    }

    /// Called when a new candle is received
    ///
    /// This is the primary method for strategies that operate on candle timeframes.
    async fn on_candle(
        &mut self,
        candle: &Candle,
        ctx: &StrategyContext,
    ) -> StrategyResult<Signal> {
        let _ = (candle, ctx);
        Ok(Signal::Hold)
    }

    /// Called when a new trade is received
    ///
    /// Use this for strategies that need tick-by-tick trade data.
    async fn on_trade(&mut self, trade: &Trade, ctx: &StrategyContext) -> StrategyResult<Signal> {
        let _ = (trade, ctx);
        Ok(Signal::Hold)
    }

    /// Called when a new tick (quote) is received
    ///
    /// Use this for market-making or strategies that react to order book changes.
    async fn on_tick(&mut self, tick: &Tick, ctx: &StrategyContext) -> StrategyResult<Signal> {
        let _ = (tick, ctx);
        Ok(Signal::Hold)
    }

    /// Called when an order is filled
    ///
    /// Use this to track position changes and update strategy state.
    /// Note: Fill notifications are typically handled by the execution engine,
    /// which updates positions automatically.
    async fn on_order_update(&mut self, ctx: &StrategyContext) -> StrategyResult<Signal> {
        let _ = ctx;
        Ok(Signal::Hold)
    }

    /// Called periodically (e.g., every second)
    ///
    /// Use this for time-based logic like trailing stops, position monitoring, etc.
    async fn on_timer(&mut self, ctx: &StrategyContext) -> StrategyResult<Signal> {
        let _ = ctx;
        Ok(Signal::Hold)
    }

    /// Reset the strategy to initial state
    ///
    /// Clears all indicators and internal state.
    fn reset(&mut self);

    /// Shutdown the strategy gracefully
    ///
    /// Called when the strategy is being stopped. Use this to close positions,
    /// save state, clean up resources, etc.
    async fn shutdown(&mut self, ctx: &StrategyContext) -> StrategyResult<()> {
        let _ = ctx;
        Ok(())
    }

    /// Validate a signal before execution
    ///
    /// Override this to implement custom risk checks, position limits, etc.
    fn validate_signal(&self, signal: &Signal, ctx: &StrategyContext) -> StrategyResult<bool> {
        let _ = (signal, ctx);
        Ok(true)
    }
}

/// Strategy metadata
pub struct StrategyMetadata {
    /// Strategy name
    pub name: String,
    /// Strategy version
    pub version: String,
    /// Strategy description
    pub description: String,
    /// Strategy author
    pub author: Option<String>,
    /// Strategy parameters
    pub parameters: Vec<ParameterInfo>,
}

/// Parameter information for strategy configuration
pub struct ParameterInfo {
    /// Parameter name
    pub name: String,
    /// Parameter description
    pub description: String,
    /// Parameter type (e.g., "number", "string", "boolean")
    pub param_type: String,
    /// Default value (as JSON)
    pub default: Option<serde_json::Value>,
    /// Minimum value (for numbers)
    pub min: Option<f64>,
    /// Maximum value (for numbers)
    pub max: Option<f64>,
    /// Required parameter
    pub required: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::StrategyConfig;

    struct DummyStrategy {
        config: StrategyConfig,
        state: StrategyState,
    }

    impl DummyStrategy {
        fn new() -> Self {
            Self {
                config: StrategyConfig::new("Dummy"),
                state: StrategyState::Initializing,
            }
        }
    }

    #[async_trait]
    impl Strategy for DummyStrategy {
        fn name(&self) -> &str {
            &self.config.name
        }

        fn config(&self) -> &StrategyConfig {
            &self.config
        }

        fn state(&self) -> StrategyState {
            self.state
        }

        fn reset(&mut self) {
            self.state = StrategyState::Initializing;
        }
    }

    #[tokio::test]
    async fn test_dummy_strategy() {
        let mut strategy = DummyStrategy::new();
        assert_eq!(strategy.name(), "Dummy");
        assert_eq!(strategy.state(), StrategyState::Initializing);

        let ctx = StrategyContext::new(10_000.0);
        let result = strategy.initialize(&ctx).await;
        assert!(result.is_ok());
    }
}
