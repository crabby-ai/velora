//! Example: Loading Layered Configuration
//!
//! This example demonstrates how to load configuration from multiple sources
//! using Velora's layered configuration system.
//!
//! Run with:
//! ```bash
//! cargo run --example config_loading
//! ```

use velora_core::VeloraConfig;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Velora Configuration Loading Examples ===\n");

    // Example 1: Load from single file
    println!("1. Loading from single file (base.toml):");
    match VeloraConfig::from_file("config/base.toml") {
        Ok(config) => {
            println!("   ✓ Loaded successfully");
            println!("   Database backend: {:?}", config.database.backend);
            println!("   Logging level: {}", config.logging.level);
            println!();
        }
        Err(e) => {
            println!("   ✗ Failed to load: {e}");
            println!("   (This is expected if config/base.toml doesn't exist)\n");
        }
    }

    // Example 2: Load layered configuration (base + testing)
    println!("2. Loading layered config (base.toml + testing.toml):");
    match VeloraConfig::from_files(&["config/base.toml", "config/testing.toml"]) {
        Ok(config) => {
            println!("   ✓ Loaded successfully");
            println!("   Database backend: {:?}", config.database.backend);
            println!("   Logging level: {}", config.logging.level);
            println!("   Max position size: {}", config.risk.max_position_size);
            println!();
        }
        Err(e) => {
            println!("   ✗ Failed to load: {e}");
            println!("   (This is expected if config files don't exist)\n");
        }
    }

    // Example 3: Load with optional local overrides
    println!("3. Loading with optional local.toml override:");
    match VeloraConfig::from_files_optional(&[
        ("config/base.toml", true),    // required
        ("config/testing.toml", true), // required
        ("config/local.toml", false),  // optional (won't error if missing)
    ]) {
        Ok(config) => {
            println!("   ✓ Loaded successfully");
            println!("   Database backend: {:?}", config.database.backend);
            println!();
        }
        Err(e) => {
            println!("   ✗ Failed to load: {e}");
            println!();
        }
    }

    // Example 4: Environment variable overrides
    println!("4. Environment variable override example:");
    println!("   Set: export VELORA_LOGGING_LEVEL=trace");
    println!("   Then the logging level will override any config file value");
    println!();

    // Example 5: Demonstrate different environments
    println!("5. Different environment configurations:\n");

    let environments = vec![
        ("Testing", vec!["config/base.toml", "config/testing.toml"]),
        (
            "Paper Trading",
            vec!["config/base.toml", "config/paper-trading.toml"],
        ),
        (
            "Live Trading",
            vec!["config/base.toml", "config/live-trading.toml"],
        ),
        (
            "Backtesting",
            vec!["config/base.toml", "config/backtesting.toml"],
        ),
    ];

    for (name, paths) in environments {
        print!("   {name}: ");
        match VeloraConfig::from_files(&paths) {
            Ok(config) => {
                println!(
                    "DB={:?}, DryRun={}, MaxPos=${}",
                    config.database.backend,
                    config.engine.live.dry_run,
                    config.risk.max_position_size
                );
            }
            Err(_) => {
                println!("(config files not found)");
            }
        }
    }

    println!("\n=== Configuration Priority ===");
    println!("Highest → Lowest:");
    println!("1. Environment variables (VELORA_*)");
    println!("2. Last config file in list");
    println!("3. ...");
    println!("4. First config file in list (usually base.toml)");
    println!("5. Default values in code");

    println!("\n=== Usage Examples ===");
    println!("Development:");
    println!("  cargo run -- --config config/base.toml --config config/testing.toml");
    println!();
    println!("Paper Trading:");
    println!("  cargo run -- --config config/base.toml --config config/paper-trading.toml");
    println!();
    println!("Live Trading (⚠️  REAL MONEY):");
    println!("  export VELORA_EXCHANGES_BINANCE_PROD_API_KEY=your_key");
    println!("  export VELORA_EXCHANGES_BINANCE_PROD_API_SECRET=your_secret");
    println!("  cargo run -- --config config/base.toml --config config/live-trading.toml");

    Ok(())
}
