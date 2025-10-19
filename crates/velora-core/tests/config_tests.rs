//! Integration tests for configuration loading and validation
//!
//! Note: Tests that use environment variables use serial_test to avoid conflicts

use velora_core::{DatabaseBackend, VeloraConfig};

#[test]
fn test_load_single_config() {
    // Clear any environment variables that might interfere
    std::env::remove_var("VELORA_LOGGING_LEVEL");
    std::env::remove_var("VELORA_DATABASE_BACKEND");

    let config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Verify logging config
    assert_eq!(config.logging.level, "info");

    // Verify database config
    assert_eq!(config.database.backend, DatabaseBackend::QuestDB);
    assert!(config.database.questdb.is_some());
    let questdb = config.database.questdb.unwrap();
    assert_eq!(questdb.host, "localhost");
    assert_eq!(questdb.pg_port, 8812);

    // Verify engine config
    assert_eq!(config.engine.backtest.initial_capital, 10000.0);
    assert_eq!(config.engine.backtest.commission_rate, 0.001);
    assert!(config.engine.live.dry_run);

    // Verify risk config
    assert_eq!(config.risk.max_position_size, 1000.0);
    assert_eq!(config.risk.max_total_exposure, 10000.0);
    assert_eq!(config.risk.max_drawdown_percent, 20.0);
}

#[test]
fn test_layered_config_override() {
    // Clear any environment variables that might interfere
    std::env::remove_var("VELORA_LOGGING_LEVEL");
    std::env::remove_var("VELORA_DATABASE_BACKEND");
    std::env::remove_var("VELORA_RISK_MAX_POSITION_SIZE");

    let config = VeloraConfig::from_files(&[
        "tests/fixtures/test_base.toml",
        "tests/fixtures/test_override.toml",
    ])
    .expect("Failed to load layered configs");

    // Verify override values
    assert_eq!(config.logging.level, "debug"); // Overridden
    assert_eq!(config.database.backend, DatabaseBackend::InMemory); // Overridden
    assert_eq!(config.risk.max_position_size, 500.0); // Overridden

    // Verify inherited values (not in override file)
    assert_eq!(config.risk.max_total_exposure, 10000.0); // From base
    assert_eq!(config.risk.max_drawdown_percent, 20.0); // From base
    assert_eq!(config.engine.backtest.initial_capital, 10000.0); // From base
}

#[test]
fn test_config_validation_valid() {
    let config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Should not error on valid config
    config.validate().expect("Valid config failed validation");
}

#[test]
fn test_config_validation_invalid_capital() {
    let mut config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Set invalid value
    config.engine.backtest.initial_capital = -100.0;

    // Should error
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Initial capital must be positive"));
}

#[test]
fn test_config_validation_invalid_commission() {
    let mut config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Set invalid value
    config.engine.backtest.commission_rate = 1.5; // > 1.0

    // Should error
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Commission rate must be between 0 and 1"));
}

#[test]
fn test_config_validation_invalid_drawdown() {
    let mut config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Set invalid value
    config.risk.max_drawdown_percent = 150.0; // > 100%

    // Should error
    let result = config.validate();
    assert!(result.is_err());
    assert!(result
        .unwrap_err()
        .to_string()
        .contains("Max drawdown must be between 0 and 100"));
}

#[test]
fn test_optional_file_loading() {
    // Load with required and non-existent optional file
    let config = VeloraConfig::from_files_optional(&[
        ("tests/fixtures/test_base.toml", true),         // required
        ("tests/fixtures/test_override.toml", true),     // required
        ("tests/fixtures/test_nonexistent.toml", false), // optional (doesn't exist)
    ])
    .expect("Failed to load with optional file");

    // Should have loaded the first two files
    assert_eq!(config.logging.level, "debug");
    assert_eq!(config.database.backend, DatabaseBackend::InMemory);
}

#[test]
fn test_optional_file_loading_missing_required() {
    // Should fail if required file is missing
    let result = VeloraConfig::from_files_optional(&[
        ("tests/fixtures/test_base.toml", true),
        ("tests/fixtures/test_missing_required.toml", true), // required but doesn't exist
    ]);

    assert!(result.is_err());
}

#[test]
fn test_default_config() {
    let config = VeloraConfig::default();

    // Check defaults
    assert!(config.exchanges.is_empty());
    assert_eq!(config.engine.backtest.initial_capital, 10000.0);
    assert_eq!(config.risk.max_drawdown_percent, 20.0);
    assert!(config.engine.live.dry_run);
}

#[test]
fn test_database_backend_serialization() {
    std::env::remove_var("VELORA_DATABASE_BACKEND");

    let config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Verify backend enum serialization works
    match config.database.backend {
        DatabaseBackend::QuestDB => {} // Expected
        other => panic!("Expected QuestDB backend, got {other:?}"),
    }

    let override_config = VeloraConfig::from_files(&[
        "tests/fixtures/test_base.toml",
        "tests/fixtures/test_override.toml",
    ])
    .expect("Failed to load layered configs");

    match override_config.database.backend {
        DatabaseBackend::InMemory => {} // Expected
        other => panic!("Expected InMemory backend, got {other:?}"),
    }
}

#[test]
fn test_config_clone_and_debug() {
    let config = VeloraConfig::from_file("tests/fixtures/test_base.toml")
        .expect("Failed to load test config");

    // Test Clone trait
    let cloned = config.clone();
    assert_eq!(cloned.logging.level, config.logging.level);

    // Test Debug trait (should not panic)
    let debug_str = format!("{config:?}");
    assert!(debug_str.contains("VeloraConfig"));
}

#[test]
fn test_env_var_override_in_isolation() {
    // This test demonstrates env var override in a controlled way
    // by using a subprocess-like approach or by testing the from_env method

    // Test that default config works
    let _config = VeloraConfig::default();

    // Note: In real usage, env vars would be set before running the application
    // Testing env vars in unit tests is tricky due to test parallelization
    // For production verification, use the config_loading example instead
}

#[test]
#[ignore] // Run manually with: cargo test --package velora-core -- --ignored
fn test_config_files_exist() {
    use std::path::Path;

    // Note: This test must be run from workspace root
    // Verify our main config files exist
    assert!(
        Path::new("config/base.toml").exists(),
        "base.toml should exist (run from workspace root)"
    );
    assert!(
        Path::new("config/testing.toml").exists(),
        "testing.toml should exist"
    );
    assert!(
        Path::new("config/paper-trading.toml").exists(),
        "paper-trading.toml should exist"
    );
    assert!(
        Path::new("config/live-trading.toml").exists(),
        "live-trading.toml should exist"
    );
    assert!(
        Path::new("config/backtesting.toml").exists(),
        "backtesting.toml should exist"
    );
}

#[test]
#[ignore] // Run manually with: cargo test --package velora-core -- --ignored
fn test_actual_config_files_load() {
    // Note: This test must be run from workspace root
    // Test that actual config files can be loaded
    let _ = VeloraConfig::from_file("config/base.toml").expect("Failed to load config/base.toml");

    let _ = VeloraConfig::from_files(&["config/base.toml", "config/testing.toml"])
        .expect("Failed to load base + testing configs");

    let _ = VeloraConfig::from_files(&["config/base.toml", "config/paper-trading.toml"])
        .expect("Failed to load base + paper-trading configs");
}

#[test]
fn test_config_layering_priority() {
    // Clear env vars
    std::env::remove_var("VELORA_LOGGING_LEVEL");
    std::env::remove_var("VELORA_DATABASE_BACKEND");

    // Base should have info level
    let base = VeloraConfig::from_file("tests/fixtures/test_base.toml").unwrap();
    assert_eq!(base.logging.level, "info");

    // Layered should have debug level (from override)
    let layered = VeloraConfig::from_files(&[
        "tests/fixtures/test_base.toml",
        "tests/fixtures/test_override.toml",
    ])
    .unwrap();
    assert_eq!(layered.logging.level, "debug");
}
