//! Unified Exchange Validation Example
//!
//! This example demonstrates how to use the common Exchange trait to test
//! different exchanges with the same code. It validates all market data
//! functionality that both Lighter and Paradex support.
//!
//! Usage:
//!   cargo run --example exchange_validation -- lighter
//!   cargo run --example exchange_validation -- paradex
//!   cargo run --example exchange_validation -- all

use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use std::env;
use velora_core::types::Interval;
use velora_exchange::{
    auth::AuthConfig,
    exchanges::{lighter::LighterExchange, paradex::ParadexExchange},
    traits::{Exchange, MarketData},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    let exchange_name = args.get(1).map(|s| s.as_str()).unwrap_or("lighter");

    match exchange_name.to_lowercase().as_str() {
        "lighter" => {
            println!("🔧 Testing Lighter Exchange\n");
            test_lighter().await?;
        }
        "paradex" => {
            println!("🔧 Testing Paradex Exchange\n");
            test_paradex().await?;
        }
        "all" => {
            println!("🔧 Testing All Exchanges\n");
            println!("{}\n", "=".repeat(80));
            test_lighter().await?;
            println!("\n{}\n", "=".repeat(80));
            test_paradex().await?;
        }
        _ => {
            eprintln!("❌ Unknown exchange: {exchange_name}");
            eprintln!("\nUsage:");
            eprintln!("  cargo run --example exchange_validation -- lighter");
            eprintln!("  cargo run --example exchange_validation -- paradex");
            eprintln!("  cargo run --example exchange_validation -- all");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn test_lighter() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lighter Exchange Validation");
    println!("{}", "=".repeat(80));

    // Initialize Lighter with no authentication (public endpoints)
    let exchange = LighterExchange::new(AuthConfig::None).await?;
    println!("✅ Connected to Lighter Exchange");
    println!("   Type: {:?}", exchange.exchange_type());
    println!("   Instruments: {:?}\n", exchange.supported_instruments());

    // Run common tests
    let test_symbol = "BTC"; // Lighter uses simple symbols
    test_exchange_common(&exchange, test_symbol, "Lighter").await?;

    Ok(())
}

async fn test_paradex() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Paradex Exchange Validation");
    println!("{}", "=".repeat(80));

    // Initialize Paradex with no authentication (public endpoints)
    let exchange = ParadexExchange::new(AuthConfig::None).await?;
    println!("✅ Connected to Paradex Exchange");
    println!("   Type: {:?}", exchange.exchange_type());
    println!("   Instruments: {:?}\n", exchange.supported_instruments());

    // Run common tests
    let test_symbol = "BTC-USD-PERP"; // Paradex uses full perpetual notation
    test_exchange_common(&exchange, test_symbol, "Paradex").await?;

    Ok(())
}

/// Common test suite that works with any Exchange implementation
async fn test_exchange_common(
    exchange: &dyn Exchange,
    test_symbol: &str,
    exchange_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let market_data = exchange.market_data();
    let symbol = velora_core::types::Symbol::from(test_symbol);

    println!("🔍 Testing Common Market Data Functionality\n");

    // Test 1: Get all markets
    println!("1️⃣  get_markets()");
    match market_data.get_markets().await {
        Ok(markets) => {
            println!("   ✅ Success: {} markets found", markets.len());
            if let Some(market) = markets.first() {
                println!(
                    "   📊 Sample: {} ({:?})",
                    market.symbol, market.instrument_type
                );
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 2: Get specific market
    println!("2️⃣  get_market({test_symbol})");
    match market_data.get_market(&symbol).await {
        Ok(market) => {
            println!("   ✅ Success: {}", market.symbol);
            println!(
                "      Base: {}, Quote: {}",
                market.base_asset, market.quote_asset
            );
            println!(
                "      Type: {:?}, Status: {:?}",
                market.instrument_type, market.status
            );
            println!(
                "      Min Qty: {}, Tick: {}",
                market.min_quantity, market.tick_size
            );
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 3: Get ticker
    println!("3️⃣  get_ticker({test_symbol})");
    match market_data.get_ticker(&symbol).await {
        Ok(ticker) => {
            println!("   ✅ Success: {}", ticker.symbol);
            println!("      Last Price: ${}", ticker.last_price);
            println!("      Bid: ${}, Ask: ${}", ticker.bid, ticker.ask);
            println!("      24h Volume: {}", ticker.volume_24h);
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 4: Get all tickers
    println!("4️⃣  get_tickers()");
    match market_data.get_tickers().await {
        Ok(tickers) => {
            println!("   ✅ Success: {} tickers found", tickers.len());
            if tickers.len() >= 3 {
                println!("   📊 Top 3 by last price:");
                let mut sorted = tickers.clone();
                sorted.sort_by(|a, b| b.last_price.partial_cmp(&a.last_price).unwrap());
                for (i, ticker) in sorted.iter().take(3).enumerate() {
                    println!(
                        "      {}. {} @ ${}",
                        i + 1,
                        ticker.symbol,
                        ticker.last_price
                    );
                }
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 5: Get orderbook
    println!("5️⃣  get_orderbook({test_symbol}, depth=5)");
    match market_data.get_orderbook(&symbol, Some(5)).await {
        Ok(orderbook) => {
            println!(
                "   ✅ Success: {} bids, {} asks",
                orderbook.bids.len(),
                orderbook.asks.len()
            );
            if let Some(best_bid) = orderbook.best_bid() {
                println!(
                    "      Best Bid: ${} (qty: {})",
                    best_bid.price, best_bid.quantity
                );
            }
            if let Some(best_ask) = orderbook.best_ask() {
                println!(
                    "      Best Ask: ${} (qty: {})",
                    best_ask.price, best_ask.quantity
                );
            }
            if let Some(spread) = orderbook.spread() {
                println!("      Spread: ${spread}");
            }
            if let Some(mid) = orderbook.mid_price() {
                println!("      Mid Price: ${mid}");
            }
        }
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 6: Get recent trades
    println!("6️⃣  get_recent_trades({test_symbol}, limit=3)");
    match market_data.get_recent_trades(&symbol, Some(3)).await {
        Ok(trades) => {
            println!("   ✅ Success: {} trades found", trades.len());
            for (i, trade) in trades.iter().enumerate() {
                println!(
                    "      {}. {:?} {} @ ${} at {}",
                    i + 1,
                    trade.side,
                    trade.quantity,
                    trade.price,
                    trade.timestamp
                );
            }
        }
        Err(e) => {
            if e.to_string().contains("authentication") {
                println!("   ⚠️  Requires authentication (expected for {exchange_name})");
            } else {
                println!("   ❌ Failed: {e}");
            }
        }
    }
    println!();

    // Test 7: Get candles
    println!("7️⃣  get_candles({test_symbol}, 1h, last 3)");
    let end_time = Utc::now();
    let start_time = end_time - Duration::hours(3);
    match market_data
        .get_candles(
            &symbol,
            Interval::Hour1,
            Some(start_time),
            Some(end_time),
            Some(3),
        )
        .await
    {
        Ok(candles) => {
            println!("   ✅ Success: {} candles found", candles.len());
            for (i, candle) in candles.iter().enumerate() {
                println!(
                    "      {}. O:{} H:{} L:{} C:{} V:{} [{} - {}]",
                    i + 1,
                    candle.open,
                    candle.high,
                    candle.low,
                    candle.close,
                    candle.volume,
                    candle.open_time.format("%H:%M"),
                    candle.close_time.format("%H:%M")
                );
            }
        }
        Err(e) => {
            if e.to_string().contains("authentication") {
                println!("   ⚠️  Requires authentication (expected for {exchange_name})");
            } else {
                println!("   ❌ Failed: {e}");
            }
        }
    }
    println!();

    // Test 8: Get funding rate
    println!("8️⃣  get_funding_rate({test_symbol})");
    match market_data.get_funding_rate(&symbol).await {
        Ok(Some(funding)) => {
            println!("   ✅ Success: {}", funding.symbol);
            println!(
                "      Current Rate: {}% (annual: {}%)",
                funding.rate * Decimal::from(100),
                funding.rate * Decimal::from(100) * Decimal::from(365 * 3) // 8h funding
            );
            println!("      Next Funding: {}", funding.next_funding_time);
        }
        Ok(None) => println!("   ⚠️  No funding rate (not a perpetual contract)"),
        Err(e) => println!("   ❌ Failed: {e}"),
    }
    println!();

    // Test 9: Get funding rate history
    println!("9️⃣  get_funding_rate_history({test_symbol}, last 24h, limit=5)");
    let end_time = Utc::now();
    let start_time = end_time - Duration::days(1);
    match market_data
        .get_funding_rate_history(&symbol, Some(start_time), Some(end_time), Some(5))
        .await
    {
        Ok(history) => {
            println!("   ✅ Success: {} funding rate entries", history.len());
            for (i, funding) in history.iter().enumerate() {
                println!(
                    "      {}. Rate: {}% at {}",
                    i + 1,
                    funding.rate * Decimal::from(100),
                    funding.timestamp.format("%Y-%m-%d %H:%M")
                );
            }
            if !history.is_empty() {
                let avg_rate =
                    history.iter().map(|f| f.rate).sum::<Decimal>() / Decimal::from(history.len());
                println!("      Average Rate: {}%", avg_rate * Decimal::from(100));
            }
        }
        Err(e) => {
            if e.to_string().contains("authentication") {
                println!("   ⚠️  Requires authentication (expected for {exchange_name})");
            } else {
                println!("   ❌ Failed: {e}");
            }
        }
    }
    println!();

    // Summary
    println!("✅ {exchange_name} Validation Complete");

    Ok(())
}
