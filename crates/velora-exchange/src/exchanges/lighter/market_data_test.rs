//! Tests for Lighter market data implementation

#[cfg(test)]
mod tests {
    use super::super::market_data::*;

    use crate::common::{RateLimiter, RestClient};
    use crate::traits::MarketData;
    use crate::types::{Interval, Symbol};

    use mockito::{Matcher, Server};
    use std::sync::Arc;

    /// Helper to create a test market data instance with mocked server
    async fn setup_test(server: &Server) -> LighterMarketData {
        let rest_client =
            Arc::new(RestClient::new(server.url(), std::time::Duration::from_secs(30)).unwrap());
        let rate_limiter = Arc::new(RateLimiter::new(100, std::time::Duration::from_secs(1)));

        LighterMarketData::new(rest_client, rate_limiter)
    }

    #[tokio::test]
    async fn test_get_markets_success() {
        let mut server = Server::new_async().await;

        // Mock response for /orderbooks endpoint
        let mock_response = r#"[
            {
                "order_book_id": "BTC-USDT-PERP",
                "symbol": "BTC-USDT-PERP",
                "base_currency": "BTC",
                "quote_currency": "USDT",
                "price_precision": 2,
                "size_precision": 4,
                "min_order_size": "0.0001",
                "max_order_size": "1000",
                "last_price": "50000.00",
                "best_bid": "49999.50",
                "best_ask": "50000.50",
                "high_24h": "51000.00",
                "low_24h": "49000.00",
                "volume_24h": "1234567.89",
                "price_change_24h": "500.00",
                "price_change_percent_24h": 1.01
            },
            {
                "order_book_id": "ETH-USDT-PERP",
                "symbol": "ETH-USDT-PERP",
                "base_currency": "ETH",
                "quote_currency": "USDT",
                "price_precision": 2,
                "size_precision": 3,
                "min_order_size": "0.001",
                "max_order_size": "10000",
                "last_price": "3000.00",
                "best_bid": "2999.50",
                "best_ask": "3000.50",
                "high_24h": "3100.00",
                "low_24h": "2900.00",
                "volume_24h": "987654.32",
                "price_change_24h": "50.00",
                "price_change_percent_24h": 1.69
            }
        ]"#;

        let _m = server
            .mock("GET", "/orderbooks")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let markets = market_data.get_markets().await.unwrap();

        assert_eq!(markets.len(), 2);
        assert_eq!(markets[0].symbol, Symbol::from("BTC-USDT-PERP"));
        assert_eq!(markets[0].base_asset, "BTC");
        assert_eq!(markets[0].quote_asset, "USDT");
        assert_eq!(markets[1].symbol, Symbol::from("ETH-USDT-PERP"));
    }

    #[tokio::test]
    async fn test_get_market_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"{
            "order_book_id": "BTC-USDT-PERP",
            "symbol": "BTC-USDT-PERP",
            "base_currency": "BTC",
            "quote_currency": "USDT",
            "price_precision": 2,
            "size_precision": 4,
            "min_order_size": "0.0001",
            "max_order_size": "1000",
            "last_price": "50000.00",
            "best_bid": "49999.50",
            "best_ask": "50000.50",
            "high_24h": "51000.00",
            "low_24h": "49000.00",
            "volume_24h": "1234567.89",
            "price_change_24h": "500.00",
            "price_change_percent_24h": 1.01
        }"#;

        let _m = server
            .mock("GET", "/orderbookdetails")
            .match_query(Matcher::UrlEncoded(
                "order_book_id".into(),
                "BTC-USDT-PERP".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let market = market_data
            .get_market(&Symbol::from("BTC-USDT-PERP"))
            .await
            .unwrap();

        assert_eq!(market.symbol, Symbol::from("BTC-USDT-PERP"));
        assert_eq!(market.base_asset, "BTC");
        assert_eq!(market.quote_asset, "USDT");
    }

    #[tokio::test]
    async fn test_get_ticker_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"{
            "order_book_id": "BTC-USDT-PERP",
            "symbol": "BTC-USDT-PERP",
            "base_currency": "BTC",
            "quote_currency": "USDT",
            "price_precision": 2,
            "size_precision": 4,
            "min_order_size": "0.0001",
            "max_order_size": "1000",
            "last_price": "50000.00",
            "best_bid": "49999.50",
            "best_ask": "50000.50",
            "high_24h": "51000.00",
            "low_24h": "49000.00",
            "volume_24h": "1234567.89",
            "price_change_24h": "500.00",
            "price_change_percent_24h": 1.01
        }"#;

        let _m = server
            .mock("GET", "/orderbookdetails")
            .match_query(Matcher::UrlEncoded(
                "order_book_id".into(),
                "BTC-USDT-PERP".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let ticker = market_data
            .get_ticker(&Symbol::from("BTC-USDT-PERP"))
            .await
            .unwrap();

        assert_eq!(ticker.symbol, Symbol::from("BTC-USDT-PERP"));
        assert!(ticker.last_price.into_inner() > 0.0);
        assert!(ticker.volume_24h > rust_decimal::Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_get_orderbook_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"{
            "order_book_id": "BTC-USDT-PERP",
            "symbol": "BTC-USDT-PERP",
            "bids": [
                {"price": "49999.50", "size": "1.5"},
                {"price": "49999.00", "size": "2.3"}
            ],
            "asks": [
                {"price": "50000.50", "size": "1.2"},
                {"price": "50001.00", "size": "3.4"}
            ],
            "timestamp": 1234567890000
        }"#;

        let _m = server
            .mock("GET", "/orderbookdetails")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("order_book_id".into(), "BTC-USDT-PERP".into()),
                Matcher::UrlEncoded("depth".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let orderbook = market_data
            .get_orderbook(&Symbol::from("BTC-USDT-PERP"), Some(10))
            .await
            .unwrap();

        assert_eq!(orderbook.symbol, Symbol::from("BTC-USDT-PERP"));
        assert_eq!(orderbook.bids.len(), 2);
        assert_eq!(orderbook.asks.len(), 2);
        assert!(orderbook.bids[0].price.into_inner() > 0.0);
        assert!(orderbook.bids[0].quantity > rust_decimal::Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_get_recent_trades_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"[
            {
                "id": "trade1",
                "order_book_id": "BTC-USDT-PERP",
                "price": "50000.00",
                "size": "0.5",
                "side": "buy",
                "timestamp": 1234567890000
            },
            {
                "id": "trade2",
                "order_book_id": "BTC-USDT-PERP",
                "price": "50001.00",
                "size": "0.3",
                "side": "sell",
                "timestamp": 1234567891000
            }
        ]"#;

        let _m = server
            .mock("GET", "/recenttrades")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("order_book_id".into(), "BTC-USDT-PERP".into()),
                Matcher::UrlEncoded("limit".into(), "100".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let trades = market_data
            .get_recent_trades(&Symbol::from("BTC-USDT-PERP"), Some(100))
            .await
            .unwrap();

        assert_eq!(trades.len(), 2);
        assert_eq!(trades[0].trade_id, "trade1");
        assert_eq!(trades[0].symbol, Symbol::from("BTC-USDT-PERP"));
        assert!(trades[0].price.into_inner() > 0.0);
    }

    #[tokio::test]
    async fn test_get_candles_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"[
            {
                "order_book_id": "BTC-USDT-PERP",
                "resolution": "1h",
                "open_time": 1234567800000,
                "close_time": 1234571400000,
                "open": "49500.00",
                "high": "50500.00",
                "low": "49000.00",
                "close": "50000.00",
                "volume": "12345.67",
                "trades_count": 1234
            },
            {
                "order_book_id": "BTC-USDT-PERP",
                "resolution": "1h",
                "open_time": 1234571400000,
                "close_time": 1234575000000,
                "open": "50000.00",
                "high": "51000.00",
                "low": "49800.00",
                "close": "50200.00",
                "volume": "23456.78",
                "trades_count": 2345
            }
        ]"#;

        let _m = server
            .mock("GET", "/candlesticks")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("order_book_id".into(), "BTC-USDT-PERP".into()),
                Matcher::UrlEncoded("resolution".into(), "1h".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let candles = market_data
            .get_candles(
                &Symbol::from("BTC-USDT-PERP"),
                Interval::Hour1,
                None,
                None,
                None,
            )
            .await
            .unwrap();

        assert_eq!(candles.len(), 2);
        assert_eq!(candles[0].symbol, Symbol::from("BTC-USDT-PERP"));
        assert_eq!(candles[0].interval, Interval::Hour1);
        assert!(candles[0].open.into_inner() > 0.0);
        assert!(candles[0].volume > rust_decimal::Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_get_funding_rate_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"{
            "order_book_id": "BTC-USDT-PERP",
            "funding_rate": "0.0001",
            "funding_time": 1234567890000,
            "next_funding_time": 1234571490000
        }"#;

        let _m = server
            .mock("GET", "/funding-rates")
            .match_query(Matcher::UrlEncoded(
                "order_book_id".into(),
                "BTC-USDT-PERP".into(),
            ))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let funding_rate = market_data
            .get_funding_rate(&Symbol::from("BTC-USDT-PERP"))
            .await
            .unwrap();

        assert!(funding_rate.is_some());
        let rate = funding_rate.unwrap();
        assert_eq!(rate.symbol, Symbol::from("BTC-USDT-PERP"));
        assert!(rate.rate > rust_decimal::Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_get_funding_rate_not_available() {
        let mut server = Server::new_async().await;

        let _m = server
            .mock("GET", "/funding-rates")
            .match_query(Matcher::UrlEncoded(
                "order_book_id".into(),
                "BTC-USDT-SPOT".into(),
            ))
            .with_status(404)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let funding_rate = market_data
            .get_funding_rate(&Symbol::from("BTC-USDT-SPOT"))
            .await
            .unwrap();

        assert!(funding_rate.is_none());
    }

    #[tokio::test]
    async fn test_get_funding_rate_history_success() {
        let mut server = Server::new_async().await;

        let mock_response = r#"[
            {
                "order_book_id": "BTC-USDT-PERP",
                "funding_rate": "0.0001",
                "funding_time": 1234567890000,
                "next_funding_time": 1234571490000
            },
            {
                "order_book_id": "BTC-USDT-PERP",
                "funding_rate": "0.00015",
                "funding_time": 1234571490000,
                "next_funding_time": 1234575090000
            }
        ]"#;

        let _m = server
            .mock("GET", "/funding-rates")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("order_book_id".into(), "BTC-USDT-PERP".into()),
                Matcher::UrlEncoded("limit".into(), "10".into()),
            ]))
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(mock_response)
            .create_async()
            .await;

        let market_data = setup_test(&server).await;
        let rates = market_data
            .get_funding_rate_history(&Symbol::from("BTC-USDT-PERP"), None, None, Some(10))
            .await
            .unwrap();

        assert_eq!(rates.len(), 2);
        assert_eq!(rates[0].symbol, Symbol::from("BTC-USDT-PERP"));
        assert!(rates[0].rate > rust_decimal::Decimal::ZERO);
    }

    #[tokio::test]
    async fn test_interval_conversion() {
        // Test that all interval types map to correct strings
        let intervals = vec![
            (Interval::Second1, "1s"),
            (Interval::Minute1, "1m"),
            (Interval::Minute5, "5m"),
            (Interval::Minute15, "15m"),
            (Interval::Minute30, "30m"),
            (Interval::Hour1, "1h"),
            (Interval::Hour4, "4h"),
            (Interval::Day1, "1d"),
            (Interval::Week1, "1w"),
        ];

        for (interval, expected) in intervals {
            let result = LighterMarketData::interval_to_string(interval);
            assert_eq!(result, expected);
        }
    }

    #[tokio::test]
    async fn test_parse_price_valid() {
        let price = LighterMarketData::parse_price("12345.67").unwrap();
        assert!((price.into_inner() - 12345.67).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_parse_price_invalid() {
        let result = LighterMarketData::parse_price("invalid");
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_parse_price_opt_some() {
        let price = LighterMarketData::parse_price_opt(Some("12345.67".to_string()));
        assert!(price.is_some());
        assert!((price.unwrap().into_inner() - 12345.67).abs() < 0.01);
    }

    #[tokio::test]
    async fn test_parse_price_opt_none() {
        let price = LighterMarketData::parse_price_opt(None);
        assert!(price.is_none());
    }

    #[tokio::test]
    async fn test_parse_price_opt_invalid() {
        let price = LighterMarketData::parse_price_opt(Some("invalid".to_string()));
        assert!(price.is_none());
    }
}
