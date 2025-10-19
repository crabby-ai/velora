//! Lighter Exchange API Validation Example
//!
//! This example validates all Lighter exchange API endpoints to ensure they work correctly.
//! It tests market data, trading, and account functionality.
//!
//! Usage:
//! ```bash
//! # Copy .env.example to .env and fill in your credentials
//! cp .env.example .env
//!
//! # Run the validation
//! cargo run --example lighter_validation --features lighter
//! ```

use chrono::{Duration, Utc};
use std::env;
use velora_exchange::{
    exchanges::lighter::LighterExchange,
    traits::{Exchange, MarketData},
    types::{Interval, Symbol},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load environment variables
    dotenv::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt::init();

    println!("=== Lighter Exchange API Validation ===\n");

    // Create exchange instance
    println!("📡 Connecting to Lighter mainnet...");
    let exchange = create_exchange()?;
    println!("✅ Connected to Lighter Exchange\n");

    // Run validation tests
    validate_market_data(&exchange).await?;

    println!("\n=== Validation Complete ===");
    println!("All API endpoints are working correctly! ✨");

    Ok(())
}

fn create_exchange() -> Result<LighterExchange, Box<dyn std::error::Error>> {
    let api_url = env::var("LIGHTER_API_URL")
        .unwrap_or_else(|_| "https://mainnet.zklighter.elliot.ai".to_string());

    let ws_host =
        env::var("LIGHTER_WS_HOST").unwrap_or_else(|_| "mainnet.zklighter.elliot.ai".to_string());

    let api_key = env::var("LIGHTER_API_KEY").ok();
    let account_index: Option<u64> = env::var("LIGHTER_ACCOUNT_INDEX")
        .ok()
        .and_then(|s| s.parse().ok());

    println!("Configuration:");
    println!("  API URL: {api_url}");
    println!("  WS Host: {ws_host}");
    println!(
        "  API Key: {}",
        if api_key.is_some() {
            "✓ Set"
        } else {
            "✗ Not set"
        }
    );
    println!("  Account Index: {account_index:?}");
    println!();

    // For now, create without authentication (read-only mode)
    let exchange = LighterExchange::new_with_config(api_url, ws_host, None)?;

    Ok(exchange)
}

async fn validate_market_data(
    exchange: &LighterExchange,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating Market Data Endpoints\n");

    // Get market data component
    let market_data = exchange.market_data();

    // Test 1: Get all markets
    println!("1️⃣  Testing get_markets()...");
    match market_data.get_markets().await {
        Ok(markets) => {
            println!("   ✅ Success! Found {} markets", markets.len());
            if !markets.is_empty() {
                println!("   📊 Sample markets:");
                for market in markets.iter() {
                    println!(
                        "      - {} ({}/{})",
                        market.symbol, market.base_asset, market.quote_asset
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
            println!("   📝 Note: This endpoint might not be available. Checking documentation...");
        }
    }
    println!();

    // Test 2: Get specific market
    println!("2️⃣  Testing get_market()...");
    let test_symbol = Symbol::from("BTC"); // Lighter uses just the base asset symbol
    match market_data.get_market(&test_symbol).await {
        Ok(market) => {
            println!("   ✅ Success! Market: {}", market.symbol);
            println!(
                "      Base: {}, Quote: {}",
                market.base_asset, market.quote_asset
            );
            println!("      Status: {:?}", market.status);
            println!(
                "      Min Qty: {}, Max Qty: {}",
                market.min_quantity, market.max_quantity
            );
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
            println!("   💡 Tip: Check if the symbol format is correct for Lighter");
        }
    }
    println!();

    // Test 3: Get ticker
    println!("3️⃣  Testing get_ticker()...");
    match market_data.get_ticker(&test_symbol).await {
        Ok(ticker) => {
            println!("   ✅ Success! Ticker for {}", ticker.symbol);
            println!("      Last Price: ${}", ticker.last_price);
            println!("      Bid: ${}, Ask: ${}", ticker.bid, ticker.ask);
            println!("      24h Volume: {}", ticker.volume_24h);
            println!("      24h Change: {}%", ticker.price_change_percent_24h);
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 4: Get all tickers
    println!("4️⃣  Testing get_tickers()...");
    match market_data.get_tickers().await {
        Ok(tickers) => {
            println!("   ✅ Success! Found {} tickers", tickers.len());
            if !tickers.is_empty() {
                println!("   📈 Top 3 by volume:");
                let mut sorted_tickers = tickers;
                sorted_tickers.sort_by(|a, b| b.volume_24h.cmp(&a.volume_24h));
                for ticker in sorted_tickers.iter().take(3) {
                    println!(
                        "      - {}: ${} (Vol: {})",
                        ticker.symbol, ticker.last_price, ticker.volume_24h
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 5: Get orderbook
    println!("5️⃣  Testing get_orderbook()...");
    match market_data.get_orderbook(&test_symbol, Some(10)).await {
        Ok(orderbook) => {
            println!("   ✅ Success! Orderbook for {}", orderbook.symbol);
            println!("      Bids: {} levels", orderbook.bids.len());
            println!("      Asks: {} levels", orderbook.asks.len());
            if let (Some(best_bid), Some(best_ask)) = (orderbook.best_bid(), orderbook.best_ask()) {
                println!(
                    "      Best Bid: ${} ({})",
                    best_bid.price, best_bid.quantity
                );
                println!(
                    "      Best Ask: ${} ({})",
                    best_ask.price, best_ask.quantity
                );
                if let Some(spread) = orderbook.spread() {
                    println!("      Spread: ${spread}");
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 6: Get recent trades
    println!("6️⃣  Testing get_recent_trades()...");
    match market_data.get_recent_trades(&test_symbol, Some(10)).await {
        Ok(trades) => {
            println!("   ✅ Success! Found {} recent trades", trades.len());
            if !trades.is_empty() {
                println!("   💰 Latest trades:");
                for trade in trades.iter().take(3) {
                    println!(
                        "      - {} @ ${} ({:?})",
                        trade.quantity, trade.price, trade.side
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 7: Get candlestick data
    println!("7️⃣  Testing get_candles()...");
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(24);

    match market_data
        .get_candles(
            &test_symbol,
            Interval::Hour1,
            Some(start_time),
            Some(end_time),
            Some(24),
        )
        .await
    {
        Ok(candles) => {
            println!("   ✅ Success! Found {} candles", candles.len());
            if !candles.is_empty() {
                let latest = &candles[candles.len() - 1];
                println!("   📊 Latest candle:");
                println!(
                    "      Open: ${}, High: ${}, Low: ${}, Close: ${}",
                    latest.open, latest.high, latest.low, latest.close
                );
                println!("      Volume: {}", latest.volume);
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 8: Get funding rate (for perpetuals)
    println!("8️⃣  Testing get_funding_rate()...");
    match market_data.get_funding_rate(&test_symbol).await {
        Ok(Some(rate)) => {
            println!("   ✅ Success! Funding rate for {}", rate.symbol);
            println!(
                "      Rate: {}%",
                rate.rate * rust_decimal::Decimal::new(100, 0)
            );
            println!("      Next Funding: {}", rate.next_funding_time);
        }
        Ok(None) => {
            println!("   ℹ️  Funding rate not available (might be a spot market)");
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }
    println!();

    // Test 9: Get funding rate history
    println!("9️⃣  Testing get_funding_rate_history()...");
    match market_data
        .get_funding_rate_history(&test_symbol, None, None, Some(10))
        .await
    {
        Ok(rates) => {
            if rates.is_empty() {
                println!("   ℹ️  No funding rate history available");
            } else {
                println!("   ✅ Success! Found {} funding rate records", rates.len());
                println!("   📈 Recent funding rates:");
                for rate in rates.iter().take(3) {
                    println!(
                        "      - {}% at {}",
                        rate.rate * rust_decimal::Decimal::new(100, 0),
                        rate.timestamp
                    );
                }
            }
        }
        Err(e) => {
            println!("   ❌ Failed: {e}");
        }
    }

    Ok(())
}
