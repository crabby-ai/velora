//! Paradex market data implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use std::str::FromStr;
use std::sync::Arc;
use velora_core::Price;

use crate::{
    common::{RateLimiter, RestClient},
    traits::MarketData,
    types::{
        Candle, ExchangeError, FundingRate, InstrumentType, Interval, Market, MarketStatus,
        OrderBook, PriceLevel, Result, Side, Symbol, Ticker, Trade,
    },
};

use super::{endpoints, types::*};

/// Paradex market data component
pub struct ParadexMarketData {
    rest_client: Arc<RestClient>,
    rate_limiter: Arc<RateLimiter>,
}

impl ParadexMarketData {
    pub fn new(rest_client: Arc<RestClient>, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            rest_client,
            rate_limiter,
        }
    }

    /// Helper to parse price from string
    fn parse_price(s: &str) -> Result<Price> {
        let val = f64::from_str(s)
            .map_err(|e| ExchangeError::ParseError(format!("Invalid price {s}: {e}")))?;
        Ok(Price::from(val))
    }

    /// Helper to parse decimal from string
    fn parse_decimal(s: &str) -> Result<Decimal> {
        Decimal::from_str(s)
            .map_err(|e| ExchangeError::ParseError(format!("Invalid decimal {s}: {e}")))
    }
}

#[async_trait]
impl MarketData for ParadexMarketData {
    async fn get_markets(&self) -> Result<Vec<Market>> {
        self.rate_limiter.wait().await;

        let response: ParadexApiResponse<Vec<ParadexMarket>> =
            self.rest_client.get(endpoints::MARKETS).await?;

        let mut markets = Vec::new();
        for market_info in response.results {
            // Determine instrument type from asset_kind
            let instrument_type = match market_info.asset_kind.as_str() {
                "PERP" => InstrumentType::Perpetual,
                "PERP_OPTION" => InstrumentType::Options,
                _ => InstrumentType::Perpetual, // default
            };

            markets.push(Market {
                symbol: Symbol::from(market_info.symbol.as_str()),
                base_asset: market_info.base_currency,
                quote_asset: market_info.quote_currency,
                instrument_type,
                status: MarketStatus::Trading, // Paradex doesn't provide status in markets endpoint
                min_quantity: Self::parse_decimal(&market_info.order_size_increment)?,
                max_quantity: Self::parse_decimal(&market_info.max_order_size)
                    .unwrap_or(Decimal::MAX),
                step_size: Self::parse_decimal(&market_info.order_size_increment)?,
                tick_size: Self::parse_decimal(&market_info.price_tick_size)?,
                min_notional: Self::parse_decimal(&market_info.min_notional)?,
                instrument_info: None,
            });
        }

        Ok(markets)
    }

    async fn get_market(&self, symbol: &Symbol) -> Result<Market> {
        self.rate_limiter.wait().await;

        // Get all markets and find the requested one
        let markets = self.get_markets().await?;
        markets
            .into_iter()
            .find(|m| m.symbol == *symbol)
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))
    }

    async fn get_ticker(&self, symbol: &Symbol) -> Result<Ticker> {
        self.rate_limiter.wait().await;

        // Get all tickers and find the requested one
        let tickers = self.get_tickers().await?;
        tickers
            .into_iter()
            .find(|t| t.symbol == *symbol)
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))
    }

    async fn get_tickers(&self) -> Result<Vec<Ticker>> {
        self.rate_limiter.wait().await;

        let endpoint = format!("{}?market=ALL", endpoints::MARKETS_SUMMARY);
        let response: ParadexApiResponse<Vec<ParadexMarketSummary>> =
            self.rest_client.get(&endpoint).await?;

        let mut tickers = Vec::new();
        for summary in response.results {
            // Parse prices with defaults for missing fields
            let last_price = Self::parse_price(&summary.last_traded_price)
                .or_else(|_| Self::parse_price(&summary.mark_price))
                .unwrap_or(Price::from(0.0));

            let bid = Self::parse_price(&summary.bid).unwrap_or(Price::from(0.0));
            let ask = Self::parse_price(&summary.ask).unwrap_or(Price::from(0.0));
            let volume_24h = Self::parse_decimal(&summary.volume_24h).unwrap_or(Decimal::ZERO);

            tickers.push(Ticker {
                symbol: Symbol::from(summary.symbol.as_str()),
                last_price,
                bid,
                bid_size: Decimal::ZERO, // Not provided in summary
                ask,
                ask_size: Decimal::ZERO, // Not provided in summary
                volume_24h,
                high_24h: Price::from(0.0),              // Not provided
                low_24h: Price::from(0.0),               // Not provided
                price_change_24h: Decimal::ZERO,         // Not provided
                price_change_percent_24h: Decimal::ZERO, // Not provided
                timestamp: Utc::now(),
            });
        }

        Ok(tickers)
    }

    async fn get_orderbook(&self, symbol: &Symbol, depth: Option<usize>) -> Result<OrderBook> {
        self.rate_limiter.wait().await;

        let endpoint = format!("{}/{}", endpoints::ORDERBOOK, symbol.as_str());
        let response: ParadexOrderBook = self.rest_client.get(&endpoint).await?;

        // Parse bids and asks (they come as [price, size] arrays)
        let mut bids = Vec::new();
        for level in response.bids.iter() {
            if let (Some(price_str), Some(size_str)) = (level.first(), level.get(1)) {
                let price = Self::parse_price(price_str)?;
                let quantity = Self::parse_decimal(size_str)?;
                bids.push(PriceLevel { price, quantity });
            }
        }

        let mut asks = Vec::new();
        for level in response.asks.iter() {
            if let (Some(price_str), Some(size_str)) = (level.first(), level.get(1)) {
                let price = Self::parse_price(price_str)?;
                let quantity = Self::parse_decimal(size_str)?;
                asks.push(PriceLevel { price, quantity });
            }
        }

        // Apply depth limit if specified
        if let Some(limit) = depth {
            bids.truncate(limit);
            asks.truncate(limit);
        }

        let timestamp =
            DateTime::from_timestamp_millis(response.last_updated_at).unwrap_or_else(Utc::now);

        Ok(OrderBook {
            symbol: symbol.clone(),
            bids,
            asks,
            last_update_id: Some(response.seq_no),
            timestamp,
        })
    }

    async fn get_recent_trades(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Trade>> {
        self.rate_limiter.wait().await;

        // Build query parameters
        let page_size = limit.unwrap_or(100).min(100); // Max 100 per request
        let endpoint = format!(
            "{}?market={}&page_size={}",
            endpoints::TRADES,
            symbol.as_str(),
            page_size
        );

        // Use ParadexPaginatedResponse for trades
        let response: ParadexPaginatedResponse<Vec<ParadexTrade>> =
            self.rest_client.get(&endpoint).await?;

        // Convert to common Trade type
        let mut trades = Vec::new();
        for trade_info in response.results {
            let price = Self::parse_price(&trade_info.price)?;
            let quantity = Self::parse_decimal(&trade_info.size)?;

            // Parse side
            let side = match trade_info.side.as_str() {
                "BUY" => Side::Buy,
                "SELL" => Side::Sell,
                _ => Side::Buy, // Default to Buy if unknown
            };

            // Convert timestamp (milliseconds to DateTime)
            let timestamp =
                DateTime::from_timestamp_millis(trade_info.created_at).unwrap_or_else(Utc::now);

            trades.push(Trade {
                trade_id: trade_info.id,
                symbol: symbol.clone(),
                price,
                quantity,
                side,
                timestamp,
                buyer_maker: None, // Paradex doesn't provide this info
            });
        }

        Ok(trades)
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
        _limit: Option<usize>,
    ) -> Result<Vec<Candle>> {
        self.rate_limiter.wait().await;

        // Paradex OHLC/candles endpoint requires Starknet wallet authentication
        Err(ExchangeError::Authentication(format!(
            "Paradex OHLC/candles endpoint requires Starknet wallet authentication. Use ParadexExchange::new(AuthConfig::StarknetWallet(...)) to access candles for {symbol} with interval {interval:?}"
        )))
    }

    async fn get_funding_rate(&self, symbol: &Symbol) -> Result<Option<FundingRate>> {
        self.rate_limiter.wait().await;

        // Get funding rate from markets summary which is public
        let endpoint = format!("{}?market=ALL", endpoints::MARKETS_SUMMARY);
        let response: ParadexApiResponse<Vec<ParadexMarketSummary>> =
            self.rest_client.get(&endpoint).await?;

        // Find the specific symbol
        let summary = response
            .results
            .into_iter()
            .find(|s| s.symbol == symbol.as_str())
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))?;

        // Parse funding rate
        let rate = Self::parse_decimal(&summary.funding_rate).unwrap_or(Decimal::ZERO);

        Ok(Some(FundingRate {
            symbol: symbol.clone(),
            rate,
            next_funding_time: Utc::now() + chrono::Duration::hours(1), // Paradex uses 1h funding
            timestamp: Utc::now(),
        }))
    }

    async fn get_funding_rate_history(
        &self,
        symbol: &Symbol,
        _start_time: Option<DateTime<Utc>>,
        _end_time: Option<DateTime<Utc>>,
        _limit: Option<usize>,
    ) -> Result<Vec<FundingRate>> {
        self.rate_limiter.wait().await;

        // Paradex funding history endpoint requires Starknet wallet authentication
        Err(ExchangeError::Authentication(format!(
            "Paradex funding history endpoint requires Starknet wallet authentication. Use ParadexExchange::new(AuthConfig::StarknetWallet(...)) to access funding history for {symbol}"
        )))
    }
}
