//! Paradex Exchange API Validation Example
//!
//! This example tests all Paradex public API endpoints to validate
//! the implementation and document API behavior.
//!
//! Run with: cargo run --example paradex_validation

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use velora_core::types::Interval;
use velora_exchange::{auth::AuthConfig, exchanges::paradex::ParadexExchange, traits::Exchange};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Paradex Exchange API Validation\n");
    println!("{}", "=".repeat(60));

    // Initialize Paradex exchange with no authentication (public endpoints only)
    let exchange = ParadexExchange::new(AuthConfig::None).await?;

    println!("✅ Paradex exchange initialized successfully\n");

    // Validate market data endpoints
    validate_market_data(&exchange).await?;

    println!("\n{}", "=".repeat(60));
    println!("✅ Validation Complete!\n");

    Ok(())
}

async fn validate_market_data(
    exchange: &ParadexExchange,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🔍 Validating Market Data Endpoints\n");
    let market_data = exchange.market_data();

    // Test 1: Get all markets
    println!("1️⃣  Testing get_markets()...");
    match market_data.get_markets().await {
        Ok(markets) => {
            println!("   ✅ Success! Found {} markets", markets.len());
            if !markets.is_empty() {
                let sample = &markets[0];
                println!("   📊 Sample market: {}", sample.symbol);
                println!(
                    "      Base: {}, Quote: {}",
                    sample.base_asset, sample.quote_asset
                );
                println!(
                    "      Type: {:?}, Status: {:?}",
                    sample.instrument_type, sample.status
                );
                println!(
                    "      Tick size: {}, Step size: {}",
                    sample.tick_size, sample.step_size
                );
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 2: Get specific market
    println!("2️⃣  Testing get_market() for BTC-USD-PERP...");
    let test_symbol = velora_core::types::Symbol::from("BTC-USD-PERP");
    match market_data.get_market(&test_symbol).await {
        Ok(market) => {
            println!("   ✅ Success! Market: {}", market.symbol);
            println!(
                "      Base: {}, Quote: {}",
                market.base_asset, market.quote_asset
            );
            println!("      Min quantity: {}", market.min_quantity);
            println!("      Max quantity: {}", market.max_quantity);
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 3: Get ticker
    println!("3️⃣  Testing get_ticker() for BTC-USD-PERP...");
    match market_data.get_ticker(&test_symbol).await {
        Ok(ticker) => {
            println!("   ✅ Success! Ticker for {}", ticker.symbol);
            println!("      Last: ${}", ticker.last_price);
            println!("      Bid: ${}, Ask: ${}", ticker.bid, ticker.ask);
            println!("      24h Volume: {}", ticker.volume_24h);
            println!("      24h Change: {}%", ticker.price_change_percent_24h);
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 4: Get all tickers
    println!("4️⃣  Testing get_tickers()...");
    match market_data.get_tickers().await {
        Ok(tickers) => {
            println!("   ✅ Success! Found {} tickers", tickers.len());
            if !tickers.is_empty() {
                let sample = &tickers[0];
                println!("   📊 Sample: {} @ ${}", sample.symbol, sample.last_price);
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 5: Get orderbook
    println!("5️⃣  Testing get_orderbook() for BTC-USD-PERP with depth 10...");
    match market_data.get_orderbook(&test_symbol, Some(10)).await {
        Ok(orderbook) => {
            println!("   ✅ Success! Orderbook for {}", orderbook.symbol);
            println!("      Bids: {} levels", orderbook.bids.len());
            println!("      Asks: {} levels", orderbook.asks.len());
            if !orderbook.bids.is_empty() {
                println!(
                    "      Best bid: ${} (qty: {})",
                    orderbook.bids[0].price, orderbook.bids[0].quantity
                );
            }
            if !orderbook.asks.is_empty() {
                println!(
                    "      Best ask: ${} (qty: {})",
                    orderbook.asks[0].price, orderbook.asks[0].quantity
                );
            }
            println!("      Last updated: {}", orderbook.timestamp);
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 6: Get recent trades
    println!("6️⃣  Testing get_recent_trades() for BTC-USD-PERP with limit 5...");
    match market_data.get_recent_trades(&test_symbol, Some(5)).await {
        Ok(trades) => {
            println!("   ✅ Success! Found {} trades", trades.len());
            for (i, trade) in trades.iter().enumerate() {
                println!(
                    "      Trade {}: {:?} {} @ ${} at {}",
                    i + 1,
                    trade.side,
                    trade.quantity,
                    trade.price,
                    trade.timestamp
                );
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 7: Get candles (OHLC)
    println!("7️⃣  Testing get_candles() for BTC-USD-PERP (1h interval, last 5)...");
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(5);
    match market_data
        .get_candles(
            &test_symbol,
            Interval::Hour1,
            Some(start_time),
            Some(end_time),
            Some(5),
        )
        .await
    {
        Ok(candles) => {
            println!("   ✅ Success! Found {} candles", candles.len());
            for (i, candle) in candles.iter().enumerate() {
                println!(
                    "      Candle {}: O:{} H:{} L:{} C:{} V:{} @ {}",
                    i + 1,
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close,
                    candle.volume,
                    candle.open_time
                );
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 8: Get current funding rate
    println!("8️⃣  Testing get_funding_rate() for BTC-USD-PERP...");
    match market_data.get_funding_rate(&test_symbol).await {
        Ok(Some(funding)) => {
            println!("   ✅ Success! Funding rate for {}", funding.symbol);
            println!("      Current rate: {}%", funding.rate * Decimal::from(100));
            println!("      Next funding: {}", funding.next_funding_time);
            println!("      Updated: {}", funding.timestamp);
        }
        Ok(None) => println!("   ⚠️  No funding rate available (not a perpetual?)"),
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 9: Get funding rate history
    println!("9️⃣  Testing get_funding_rate_history() for BTC-USD-PERP (last 24h)...");
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(1);
    match market_data
        .get_funding_rate_history(&test_symbol, Some(start_time), Some(end_time), Some(10))
        .await
    {
        Ok(history) => {
            println!(
                "   ✅ Success! Found {} funding rate entries",
                history.len()
            );
            for (i, funding) in history.iter().enumerate() {
                println!(
                    "      Entry {}: Rate: {}% at {} (next: {})",
                    i + 1,
                    funding.rate * Decimal::from(100),
                    funding.timestamp,
                    funding.next_funding_time
                );
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    Ok(())
}
