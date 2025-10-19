//! Paradex Exchange client implementation

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::info;

use crate::{
    auth::{AuthConfig, StarknetWalletAuth},
    common::{RateLimiter, RestClient, WebSocketClient},
    traits::{Account, Exchange, MarketData, Streaming, Trading},
    types::{ExchangeError, ExchangeType, InstrumentType, Result},
};

use super::{
    account::ParadexAccount, endpoints, market_data::ParadexMarketData,
    streaming::ParadexStreaming, trading::ParadexTrading, SUPPORTED_INSTRUMENTS,
};

/// Paradex Exchange implementation
pub struct ParadexExchange {
    /// REST API client
    rest_client: Arc<RestClient>,

    /// WebSocket client
    ws_client: Arc<RwLock<WebSocketClient>>,

    /// Rate limiter
    rate_limiter: Arc<RateLimiter>,

    /// Authentication
    auth: Option<StarknetWalletAuth>,

    /// Connection status
    connected: bool,

    /// Market data component
    market_data: ParadexMarketData,

    /// Trading component
    trading: ParadexTrading,

    /// Account component
    account: ParadexAccount,

    /// Streaming component
    streaming: ParadexStreaming,
}

impl ParadexExchange {
    /// Create a new Paradex exchange instance
    pub async fn new(auth: AuthConfig) -> Result<Self> {
        let rest_client = Arc::new(RestClient::new(
            endpoints::REST_BASE_URL,
            Duration::from_secs(30),
        )?);

        let ws_client = Arc::new(RwLock::new(WebSocketClient::new(endpoints::WS_BASE_URL)));

        let rate_limiter = Arc::new(RateLimiter::paradex());

        // Extract Starknet wallet auth
        let wallet_auth = match auth {
            AuthConfig::StarknetWallet(wallet) => Some(wallet),
            AuthConfig::None => None,
            _ => {
                return Err(ExchangeError::Authentication(
                    "Paradex requires Starknet wallet authentication".to_string(),
                ));
            }
        };

        let market_data =
            ParadexMarketData::new(Arc::clone(&rest_client), Arc::clone(&rate_limiter));

        let trading = ParadexTrading::new(
            Arc::clone(&rest_client),
            Arc::clone(&rate_limiter),
            wallet_auth.clone(),
        );

        let account = ParadexAccount::new(
            Arc::clone(&rest_client),
            Arc::clone(&rate_limiter),
            wallet_auth.clone(),
        );

        let streaming = ParadexStreaming::new(Arc::clone(&ws_client), wallet_auth.clone());

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
impl Exchange for ParadexExchange {
    fn name(&self) -> &str {
        "paradex"
    }

    fn exchange_type(&self) -> ExchangeType {
        ExchangeType::DexL2
    }

    fn supported_instruments(&self) -> &[InstrumentType] {
        SUPPORTED_INSTRUMENTS
    }

    async fn connect(&mut self) -> Result<()> {
        info!("Connecting to Paradex exchange");

        // Connect WebSocket
        let mut ws = self.ws_client.write().await;
        ws.connect().await?;
        drop(ws);

        self.connected = true;
        info!("Successfully connected to Paradex");

        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        info!("Disconnecting from Paradex exchange");

        let mut ws = self.ws_client.write().await;
        ws.close().await?;
        drop(ws);

        self.connected = false;
        info!("Disconnected from Paradex");

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
