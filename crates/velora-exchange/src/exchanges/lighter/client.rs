//! Lighter Exchange client implementation

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    auth::{AuthConfig, EvmWalletAuth},
    common::{RateLimiter, RestClient, WebSocketClient},
    traits::{Account, Exchange, MarketData, Streaming, Trading},
    types::{ExchangeError, ExchangeType, InstrumentType, Result},
};

use super::{
    account::LighterAccount, endpoints, market_data::LighterMarketData,
    streaming::LighterStreaming, trading::LighterTrading, SUPPORTED_INSTRUMENTS,
};

/// Lighter Exchange implementation
pub struct LighterExchange {
    /// REST API client
    rest_client: Arc<RestClient>,

    /// WebSocket client
    ws_client: Arc<RwLock<WebSocketClient>>,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,

    /// Authentication
    auth: Option<EvmWalletAuth>,

    /// Connection status
    connected: bool,

    /// Market data component
    market_data: LighterMarketData,

    /// Trading component
    trading: LighterTrading,

    /// Account component
    account: LighterAccount,

    /// Streaming component
    streaming: LighterStreaming,
}

impl LighterExchange {
    /// Create a new Lighter exchange instance with authentication
    pub async fn new(auth: AuthConfig) -> Result<Self> {
        Self::new_with_urls(
            endpoints::REST_BASE_URL.to_string(),
            endpoints::WS_BASE_URL.to_string(),
            Some(auth),
        )
        .await
    }

    /// Create a new Lighter exchange instance without authentication (read-only mode)
    pub fn new_readonly() -> Result<Self> {
        Self::new_with_urls_sync(
            endpoints::REST_BASE_URL.to_string(),
            endpoints::WS_BASE_URL.to_string(),
            None,
        )
    }

    /// Create a new Lighter exchange instance with custom URLs (for testing/different networks)
    pub fn new_with_config(
        api_url: String,
        ws_host: String,
        auth: Option<AuthConfig>,
    ) -> Result<Self> {
        let ws_url = format!("wss://{ws_host}");
        Self::new_with_urls_sync(api_url, ws_url, auth)
    }

    /// Internal constructor with custom URLs (async version)
    async fn new_with_urls(
        rest_base_url: String,
        ws_base_url: String,
        auth: Option<AuthConfig>,
    ) -> Result<Self> {
        Self::new_with_urls_sync(rest_base_url, ws_base_url, auth)
    }

    /// Internal constructor with custom URLs (sync version)
    fn new_with_urls_sync(
        rest_base_url: String,
        ws_base_url: String,
        auth: Option<AuthConfig>,
    ) -> Result<Self> {
        let rest_client = Arc::new(RestClient::new(rest_base_url, Duration::from_secs(30))?);

        let ws_client = Arc::new(RwLock::new(WebSocketClient::new(ws_base_url)));

        let rate_limiter = Arc::new(RateLimiter::lighter());

        // Extract EVM wallet auth
        let wallet_auth = match auth {
            Some(AuthConfig::EvmWallet(wallet)) => Some(wallet),
            Some(AuthConfig::None) => None, // No authentication (read-only mode)
            None => None,
            Some(_) => {
                return Err(ExchangeError::Authentication(
                    "Lighter requires EVM wallet authentication (got incompatible auth type)"
                        .to_string(),
                ));
            }
        };

        let market_data =
            LighterMarketData::new(Arc::clone(&rest_client), Arc::clone(&rate_limiter));

        let trading = LighterTrading::new(
            Arc::clone(&rest_client),
            Arc::clone(&rate_limiter),
            wallet_auth.clone(),
        );

        let account = LighterAccount::new(
            Arc::clone(&rest_client),
            Arc::clone(&rate_limiter),
            wallet_auth.clone(),
        );

        let streaming = LighterStreaming::new(Arc::clone(&ws_client), wallet_auth.clone());

        Ok(Self {
            rest_client,
            ws_client,
            rate_limiter,
            auth: wallet_auth,
            connected: false,
            market_data,
            trading,
            account,
            streaming,
        })
    }
}

#[async_trait]
impl Exchange for LighterExchange {
    fn name(&self) -> &str {
        "lighter"
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DexZk
    }

    fn supported_instruments(&self) -> &[InstrumentType] {
        SUPPORTED_INSTRUMENTS
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Lighter exchange");

        // Connect WebSocket
        let mut ws = self.ws_client.write().await;
        ws.connect().await?;
        drop(ws);

        self.connected = true;
        info!("Successfully connected to Lighter");

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Lighter exchange");

        let mut ws = self.ws_client.write().await;
        ws.close().await?;
        drop(ws);

        self.connected = false;
        info!("Disconnected from Lighter");

        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    fn market_data(&self) -> &dyn MarketData {
        &self.market_data
    }

    fn trading(&self) -> &dyn Trading {
        &self.trading
    }

    fn account(&self) -> &dyn Account {
        &self.account
    }

    fn streaming(&self) -> &dyn Streaming {
        &self.streaming
    }
}
