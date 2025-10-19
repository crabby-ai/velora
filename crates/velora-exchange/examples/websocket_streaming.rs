//! WebSocket Streaming Example
//!
//! This example demonstrates real-time WebSocket streaming for both
//! Lighter and Paradex exchanges.
//!
//! Usage:
//!   cargo run --example websocket_streaming -- paradex trades BTC-USD-PERP
//!   cargo run --example websocket_streaming -- paradex orderbook BTC-USD-PERP
//!   cargo run --example websocket_streaming -- lighter orderbook BTC

use futures::StreamExt;
use std::env;
use velora_core::types::Symbol;
use velora_exchange::{
    auth::AuthConfig,
    exchanges::{lighter::LighterExchange, paradex::ParadexExchange},
    traits::Exchange,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 4 {
        eprintln!("Usage: {} <exchange> <channel> <symbol>", args[0]);
        eprintln!("\nExamples:");
        eprintln!("  {} paradex trades BTC-USD-PERP", args[0]);
        eprintln!("  {} paradex orderbook BTC-USD-PERP", args[0]);
        eprintln!("  {} lighter orderbook BTC", args[0]);
        std::process::exit(1);
    }

    let exchange_name = &args[1];
    let channel = &args[2];
    let symbol_str = &args[3];

    match exchange_name.to_lowercase().as_str() {
        "paradex" => test_paradex_streaming(channel, symbol_str).await?,
        "lighter" => test_lighter_streaming(channel, symbol_str).await?,
        _ => {
            eprintln!("Unknown exchange: {exchange_name}");
            eprintln!("Supported: paradex, lighter");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn test_paradex_streaming(
    channel: &str,
    symbol_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Paradex WebSocket Streaming");
    println!("Channel: {channel}");
    println!("Symbol: {symbol_str}");
    println!("{}", "=".repeat(80));

    // Create exchange instance
    let exchange = ParadexExchange::new(AuthConfig::None).await?;
    let symbol = Symbol::from(symbol_str);

    // Get streaming component
    let streaming = exchange.streaming();

    match channel.to_lowercase().as_str() {
        "trades" => {
            println!("📊 Subscribing to trades for {symbol}...\n");
            let mut stream = streaming.subscribe_trades(&symbol).await?;

            // Receive and display trades
            let mut count = 0;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(trade) => {
                        count += 1;
                        println!(
                            "[{}] Trade: {} {} @ ${} (ID: {})",
                            count,
                            match trade.side {
                                velora_core::types::Side::Buy => "BUY ",
                                velora_core::types::Side::Sell => "SELL",
                            },
                            trade.quantity,
                            trade.price,
                            trade.trade_id
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
        }
        "orderbook" => {
            println!("📖 Subscribing to orderbook for {symbol}...\n");
            let mut stream = streaming.subscribe_orderbook(&symbol, Some(5)).await?;

            // Receive and display orderbook updates
            let mut count = 0;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(update) => {
                        count += 1;
                        println!("\n[{count}] OrderBook Update:");
                        if let Some(seq) = update.final_update_id {
                            println!("  Sequence: {seq}");
                        }
                        println!("  Bids: {} levels", update.bids.len());
                        if let Some(best_bid) = update.bids.first() {
                            println!("    Best Bid: ${} ({})", best_bid.price, best_bid.quantity);
                        }
                        println!("  Asks: {} levels", update.asks.len());
                        if let Some(best_ask) = update.asks.first() {
                            println!("    Best Ask: ${} ({})", best_ask.price, best_ask.quantity);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
        }
        _ => {
            eprintln!("Unknown channel: {channel}");
            eprintln!("Supported channels: trades, orderbook");
            std::process::exit(1);
        }
    }

    Ok(())
}

async fn test_lighter_streaming(
    channel: &str,
    symbol_str: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Lighter WebSocket Streaming");
    println!("Channel: {channel}");
    println!("Symbol: {symbol_str}");
    println!("{}", "=".repeat(80));

    // Create exchange instance
    let exchange = LighterExchange::new(AuthConfig::None).await?;
    let symbol = Symbol::from(symbol_str);

    // Get streaming component
    let streaming = exchange.streaming();

    match channel.to_lowercase().as_str() {
        "trades" => {
            println!("📊 Subscribing to trades for {symbol}...\n");
            let mut stream = streaming.subscribe_trades(&symbol).await?;

            // Receive and display trades
            let mut count = 0;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(trade) => {
                        count += 1;
                        println!(
                            "[{}] Trade: {} {} @ ${} (ID: {})",
                            count,
                            match trade.side {
                                velora_core::types::Side::Buy => "BUY ",
                                velora_core::types::Side::Sell => "SELL",
                            },
                            trade.quantity,
                            trade.price,
                            trade.trade_id
                        );
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
        }
        "orderbook" => {
            println!("📖 Subscribing to orderbook for {symbol}...\n");
            let mut stream = streaming.subscribe_orderbook(&symbol, Some(5)).await?;

            // Receive and display orderbook updates
            let mut count = 0;
            while let Some(result) = stream.next().await {
                match result {
                    Ok(update) => {
                        count += 1;
                        println!("\n[{count}] OrderBook Update:");
                        if let Some(seq) = update.final_update_id {
                            println!("  Sequence: {seq}");
                        }
                        println!("  Bids: {} levels", update.bids.len());
                        if let Some(best_bid) = update.bids.first() {
                            println!("    Best Bid: ${} ({})", best_bid.price, best_bid.quantity);
                        }
                        println!("  Asks: {} levels", update.asks.len());
                        if let Some(best_ask) = update.asks.first() {
                            println!("    Best Ask: ${} ({})", best_ask.price, best_ask.quantity);
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {e}");
                        break;
                    }
                }
            }
        }
        _ => {
            eprintln!("Unknown channel: {channel}");
            eprintln!("Supported channels: trades, orderbook");
            std::process::exit(1);
        }
    }

    Ok(())
}
