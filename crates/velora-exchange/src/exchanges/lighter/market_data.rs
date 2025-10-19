//! Lighter market data implementation

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use rust_decimal::prelude::ToPrimitive;
use rust_decimal::Decimal;
use std::{str::FromStr, sync::Arc};

use crate::{
    common::{RateLimiter, RestClient},
    traits::MarketData,
    types::{
        error::ExchangeError, Candle, FundingRate, InstrumentType, Interval, Market, MarketStatus,
        OrderBook, Price, PriceLevel, Result, Side, Symbol, Ticker, Trade,
    },
};

use super::{endpoints, types::*};

/// Lighter market data component
pub struct LighterMarketData {
    rest_client: Arc<RestClient>,
    rate_limiter: Arc<RateLimiter>,
}

impl LighterMarketData {
    pub fn new(rest_client: Arc<RestClient>, rate_limiter: Arc<RateLimiter>) -> Self {
        Self {
            rest_client,
            rate_limiter,
        }
    }

    /// Get market_id for a given symbol by fetching all markets
    async fn get_market_id(&self, symbol: &Symbol) -> Result<u64> {
        self.rate_limiter.wait().await;
        let response: LighterApiResponse<OrderBooksData> =
            self.rest_client.get(endpoints::ORDERBOOKS).await?;

        response
            .data
            .order_books
            .into_iter()
            .find(|m| m.symbol == symbol.as_str())
            .map(|m| m.market_id)
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))
    }

    /// Helper to parse string to Price
    #[doc(hidden)]
    pub fn parse_price(s: &str) -> Result<Price> {
        let decimal = Decimal::from_str(s)
            .map_err(|e| ExchangeError::ParseError(format!("Failed to parse price '{s}': {e}")))?;
        Ok(Price::from(decimal.to_f64().unwrap_or(0.0)))
    }

    /// Helper to parse optional string to optional Price
    #[doc(hidden)]
    pub fn parse_price_opt(s: Option<String>) -> Option<Price> {
        s.and_then(|s| {
            Decimal::from_str(&s)
                .ok()
                .map(|d| Price::from(d.to_f64().unwrap_or(0.0)))
        })
    }

    /// Convert Lighter interval to our Interval type
    #[doc(hidden)]
    pub fn interval_to_string(interval: Interval) -> &'static str {
        match interval {
            Interval::Second1 => "1s",
            Interval::Minute1 => "1m",
            Interval::Minute5 => "5m",
            Interval::Minute15 => "15m",
            Interval::Minute30 => "30m",
            Interval::Hour1 => "1h",
            Interval::Hour4 => "4h",
            Interval::Day1 => "1d",
            Interval::Week1 => "1w",
        }
    }

    /// Convert Lighter candlestick to our Candle type
    fn convert_candle(
        candle: LighterCandlestick,
        symbol: Symbol,
        interval: Interval,
    ) -> Result<Candle> {
        // Calculate approximate interval duration for close_time
        let duration = match interval {
            Interval::Second1 => chrono::Duration::seconds(1),
            Interval::Minute1 => chrono::Duration::minutes(1),
            Interval::Minute5 => chrono::Duration::minutes(5),
            Interval::Minute15 => chrono::Duration::minutes(15),
            Interval::Minute30 => chrono::Duration::minutes(30),
            Interval::Hour1 => chrono::Duration::hours(1),
            Interval::Hour4 => chrono::Duration::hours(4),
            Interval::Day1 => chrono::Duration::days(1),
            Interval::Week1 => chrono::Duration::weeks(1),
        };

        Ok(Candle {
            symbol,
            interval,
            open: Price::from(candle.open),
            high: Price::from(candle.high),
            low: Price::from(candle.low),
            close: Price::from(candle.close),
            volume: Decimal::try_from(candle.volume0).unwrap_or(Decimal::ZERO),
            open_time: candle.timestamp,
            close_time: candle.timestamp + duration,
            trade_count: Some(candle.last_trade_id),
            quote_volume: Some(Decimal::try_from(candle.volume1).unwrap_or(Decimal::ZERO)),
        })
    }

    /// Convert Lighter trade to our Trade type
    fn convert_trade(trade: LighterTrade, symbol: Symbol) -> Result<Trade> {
        Ok(Trade {
            trade_id: trade.trade_id.to_string(),
            symbol,
            price: Self::parse_price(&trade.price)?,
            quantity: Decimal::from_str(&trade.size).map_err(|e| {
                ExchangeError::ParseError(format!("Failed to parse trade size: {e}"))
            })?,
            side: if trade.is_maker_ask {
                Side::Buy // Taker bought (maker was ask)
            } else {
                Side::Sell // Taker sold (maker was bid)
            },
            timestamp: trade.timestamp,
            buyer_maker: Some(!trade.is_maker_ask),
        })
    }

    /// Convert Lighter orderbook to our OrderBook type
    fn convert_orderbook(data: OrderBookOrdersData, symbol: Symbol) -> Result<OrderBook> {
        let mut bids = Vec::new();
        for order in data.bids {
            bids.push(PriceLevel {
                price: Self::parse_price(&order.price)?,
                quantity: Decimal::from_str(&order.remaining_base_amount).map_err(|e| {
                    ExchangeError::ParseError(format!("Failed to parse bid size: {e}"))
                })?,
            });
        }

        let mut asks = Vec::new();
        for order in data.asks {
            asks.push(PriceLevel {
                price: Self::parse_price(&order.price)?,
                quantity: Decimal::from_str(&order.remaining_base_amount).map_err(|e| {
                    ExchangeError::ParseError(format!("Failed to parse ask size: {e}"))
                })?,
            });
        }

        Ok(OrderBook {
            symbol,
            bids,
            asks,
            timestamp: Utc::now(),
            last_update_id: None,
        })
    }
}

#[async_trait]
impl MarketData for LighterMarketData {
    async fn get_markets(&self) -> Result<Vec<Market>> {
        self.rate_limiter.wait().await;

        // Fetch all orderbooks to get available markets
        let response: LighterApiResponse<OrderBooksData> =
            self.rest_client.get(endpoints::ORDERBOOKS).await?;

        let mut markets = Vec::new();
        for info in response.data.order_books {
            // Parse market status
            let status = match info.status.as_str() {
                "active" => MarketStatus::Trading,
                "inactive" => MarketStatus::Halted,
                _ => MarketStatus::Trading, // Default to trading if unknown
            };

            // For Lighter, all markets are USDT-margined perpetuals
            // Symbol is just the base asset (e.g., "BTC" for BTC-USDT perpetual)
            markets.push(Market {
                symbol: Symbol::from(info.symbol.as_str()),
                base_asset: info.symbol.clone(),
                quote_asset: "USDT".to_string(),
                instrument_type: InstrumentType::Perpetual,
                status,
                min_quantity: Decimal::from_str(&info.min_base_amount).unwrap_or(Decimal::ZERO),
                max_quantity: Decimal::MAX,
                step_size: Decimal::new(1, info.supported_size_decimals),
                tick_size: Decimal::new(1, info.supported_price_decimals),
                min_notional: Decimal::from_str(&info.min_quote_amount).unwrap_or(Decimal::ZERO),
                instrument_info: None,
            });
        }

        Ok(markets)
    }

    async fn get_market(&self, symbol: &Symbol) -> Result<Market> {
        self.rate_limiter.wait().await;

        // Get detailed orderbook info for specific market
        // Need to find the market_id first by searching through all markets
        let markets = self.get_markets().await?;
        markets
            .into_iter()
            .find(|m| m.symbol == *symbol)
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))
    }

    async fn get_ticker(&self, symbol: &Symbol) -> Result<Ticker> {
        self.rate_limiter.wait().await;

        // Get all markets and find the one we need for its details
        let response: LighterApiResponse<OrderBookDetailsData> = self
            .rest_client
            .get(&format!(
                "{}?symbol={}",
                endpoints::ORDERBOOK_DETAILS,
                symbol
            ))
            .await?;

        let details = response
            .data
            .order_book_details
            .into_iter()
            .next()
            .ok_or_else(|| ExchangeError::MarketNotFound(symbol.to_string()))?;

        Ok(Ticker {
            symbol: Symbol::from(details.symbol.as_str()),
            last_price: details
                .last_trade_price
                .map(Price::from)
                .unwrap_or(Price::from(0.0)),
            bid: Price::from(0.0), // Not available in this endpoint
            bid_size: Decimal::ZERO,
            ask: Price::from(0.0), // Not available in this endpoint
            ask_size: Decimal::ZERO,
            volume_24h: details
                .daily_base_token_volume
                .map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO))
                .unwrap_or(Decimal::ZERO),
            high_24h: details
                .daily_price_high
                .map(Price::from)
                .unwrap_or(Price::from(0.0)),
            low_24h: details
                .daily_price_low
                .map(Price::from)
                .unwrap_or(Price::from(0.0)),
            price_change_24h: details
                .daily_price_change
                .map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO))
                .unwrap_or(Decimal::ZERO),
            price_change_percent_24h: details
                .daily_price_change
                .map(|v| Decimal::try_from(v * 100.0).unwrap_or(Decimal::ZERO))
                .unwrap_or(Decimal::ZERO),
            timestamp: Utc::now(),
        })
    }

    async fn get_tickers(&self) -> Result<Vec<Ticker>> {
        self.rate_limiter.wait().await;

        // Get all orderbook details
        let response: LighterApiResponse<OrderBookDetailsData> =
            self.rest_client.get(endpoints::ORDERBOOK_DETAILS).await?;

        let mut tickers = Vec::new();
        for details in response.data.order_book_details {
            tickers.push(Ticker {
                symbol: Symbol::from(details.symbol.as_str()),
                last_price: details
                    .last_trade_price
                    .map(Price::from)
                    .unwrap_or(Price::from(0.0)),
                bid: Price::from(0.0),
                bid_size: Decimal::ZERO,
                ask: Price::from(0.0),
                ask_size: Decimal::ZERO,
                volume_24h: details
                    .daily_base_token_volume
                    .map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO))
                    .unwrap_or(Decimal::ZERO),
                high_24h: details
                    .daily_price_high
                    .map(Price::from)
                    .unwrap_or(Price::from(0.0)),
                low_24h: details
                    .daily_price_low
                    .map(Price::from)
                    .unwrap_or(Price::from(0.0)),
                price_change_24h: details
                    .daily_price_change
                    .map(|v| Decimal::try_from(v).unwrap_or(Decimal::ZERO))
                    .unwrap_or(Decimal::ZERO),
                price_change_percent_24h: details
                    .daily_price_change
                    .map(|v| Decimal::try_from(v * 100.0).unwrap_or(Decimal::ZERO))
                    .unwrap_or(Decimal::ZERO),
                timestamp: Utc::now(),
            });
        }

        Ok(tickers)
    }

    async fn get_orderbook(&self, symbol: &Symbol, depth: Option<usize>) -> Result<OrderBook> {
        self.rate_limiter.wait().await;

        // Get market_id for the symbol
        let market_id = self.get_market_id(symbol).await?;

        // Get orderbook snapshot using orderBookOrders endpoint
        let mut endpoint = format!("{}?market_id={}", endpoints::ORDERBOOK_ORDERS, market_id);
        if let Some(d) = depth {
            endpoint.push_str(&format!("&limit={d}"));
        }

        let response: LighterApiResponse<OrderBookOrdersData> =
            self.rest_client.get(&endpoint).await?;

        Self::convert_orderbook(response.data, symbol.clone())
    }

    async fn get_recent_trades(&self, symbol: &Symbol, limit: Option<usize>) -> Result<Vec<Trade>> {
        self.rate_limiter.wait().await;

        // Get market_id for the symbol
        let market_id = self.get_market_id(symbol).await?;

        let mut endpoint = format!("{}?market_id={}", endpoints::RECENT_TRADES, market_id);
        if let Some(l) = limit {
            endpoint.push_str(&format!("&limit={l}"));
        }

        let response: LighterApiResponse<TradesData> = self.rest_client.get(&endpoint).await?;

        response
            .data
            .trades
            .into_iter()
            .map(|t| Self::convert_trade(t, symbol.clone()))
            .collect()
    }

    async fn get_candles(
        &self,
        symbol: &Symbol,
        interval: Interval,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<Candle>> {
        self.rate_limiter.wait().await;

        // Get market_id for the symbol
        let market_id = self.get_market_id(symbol).await?;

        // Lighter requires start_timestamp, end_timestamp, and count_back
        // Use defaults if not provided
        let end = end_time.unwrap_or_else(Utc::now);
        let start = start_time.unwrap_or_else(|| end - chrono::Duration::days(1));
        let count = limit.unwrap_or(100);

        // Build candlesticks query
        let endpoint = format!(
            "{}?market_id={}&resolution={}&start_timestamp={}&end_timestamp={}&count_back={}",
            endpoints::CANDLESTICKS,
            market_id,
            Self::interval_to_string(interval),
            start.timestamp(),
            end.timestamp(),
            count
        );

        let response: LighterApiResponse<CandlesticksData> =
            self.rest_client.get(&endpoint).await?;

        response
            .data
            .candlesticks
            .into_iter()
            .map(|c| Self::convert_candle(c, symbol.clone(), interval))
            .collect()
    }

    async fn get_funding_rate(&self, symbol: &Symbol) -> Result<Option<FundingRate>> {
        self.rate_limiter.wait().await;

        // Get current funding rate
        let endpoint = format!("{}?order_book_id={}", endpoints::FUNDING_RATES, symbol);

        match self.rest_client.get::<LighterFundingRate>(&endpoint).await {
            Ok(rate) => {
                let funding_rate = Decimal::from_str(&rate.funding_rate).map_err(|e| {
                    ExchangeError::ParseError(format!("Failed to parse funding rate: {e}"))
                })?;

                Ok(Some(FundingRate {
                    symbol: Symbol::from(rate.order_book_id.as_str()),
                    rate: funding_rate,
                    next_funding_time: rate.next_funding_time,
                    timestamp: rate.funding_time,
                }))
            }
            Err(_) => Ok(None), // Funding rate not available for this market (likely spot)
        }
    }

    async fn get_funding_rate_history(
        &self,
        symbol: &Symbol,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<usize>,
    ) -> Result<Vec<FundingRate>> {
        self.rate_limiter.wait().await;

        // Get market_id for the symbol
        let market_id = self.get_market_id(symbol).await?;

        // Use fundings endpoint which requires same params as candlesticks
        let end = end_time.unwrap_or_else(Utc::now);
        let start = start_time.unwrap_or_else(|| end - chrono::Duration::days(1));
        let count = limit.unwrap_or(100);

        // Build fundings query (uses 1h resolution for funding rates)
        let endpoint = format!(
            "{}?market_id={}&resolution=1h&start_timestamp={}&end_timestamp={}&count_back={}",
            endpoints::FUNDINGS,
            market_id,
            start.timestamp(),
            end.timestamp(),
            count
        );

        let response: LighterApiResponse<FundingsData> = self.rest_client.get(&endpoint).await?;

        response
            .data
            .fundings
            .into_iter()
            .map(|funding| {
                let rate = Decimal::from_str(&funding.rate).map_err(|e| {
                    ExchangeError::ParseError(format!("Failed to parse funding rate: {e}"))
                })?;

                let timestamp =
                    DateTime::from_timestamp(funding.timestamp, 0).unwrap_or_else(Utc::now);

                Ok(FundingRate {
                    symbol: symbol.clone(),
                    rate,
                    next_funding_time: timestamp + chrono::Duration::hours(1), // Funding every hour
                    timestamp,
                })
            })
            .collect()
    }
}
