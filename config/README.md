# Velora Configuration Files

This directory contains configuration files for different environments and use cases.

## Quick Start

### 1. Testing/Development
```bash
cargo run -- --config config/base.toml --config config/testing.toml
```

### 2. Paper Trading (Simulated)
```bash
cargo run -- --config config/base.toml --config config/paper-trading.toml
```

### 3. Backtesting
```bash
cargo run -- --config config/base.toml --config config/backtesting.toml
```

### 4. Live Trading (⚠️ REAL MONEY)
```bash
# Set API credentials first!
export VELORA_EXCHANGES_BINANCE_PROD_API_KEY="your_key"
export VELORA_EXCHANGES_BINANCE_PROD_API_SECRET="your_secret"

cargo run -- --config config/base.toml --config config/live-trading.toml
```

## Configuration Files

| File | Purpose | Database | Trading |
|------|---------|----------|---------|
| `base.toml` | Shared defaults | QuestDB | N/A |
| `testing.toml` | Development & tests | In-memory | Dry-run |
| `paper-trading.toml` | Strategy testing | QuestDB | Simulated |
| `backtesting.toml` | Historical analysis | QuestDB | Historical |
| `live-trading.toml` | Production | TimescaleDB | **REAL** |

## Configuration Inheritance

Files are layered on top of each other:

```
base.toml           (defaults)
    ↓
environment.toml    (overrides)
    ↓
env variables       (final overrides)
```

### Example

**base.toml**:
```toml
[risk]
max_position_size = 1000.0
max_daily_loss = 500.0
```

**testing.toml**:
```toml
[risk]
max_position_size = 100.0  # Override
# max_daily_loss = 500.0   # Inherited from base
```

**Environment Variable**:
```bash
export VELORA_RISK_MAX_POSITION_SIZE=50.0
```

**Result**: `max_position_size = 50.0`, `max_daily_loss = 500.0`

## Security

⚠️ **NEVER commit these to version control**:
- API keys
- API secrets
- Production passwords
- Private keys

Always use environment variables for sensitive data:

```bash
export VELORA_EXCHANGES_BINANCE_API_KEY="your_key"
export VELORA_EXCHANGES_BINANCE_API_SECRET="your_secret"
export VELORA_DATABASE_TIMESCALEDB_PASSWORD="db_password"
```

## See Also

- [CONFIG_GUIDE.md](../CONFIG_GUIDE.md) - Complete configuration documentation
- [DATABASE_INTEGRATION.md](../DATABASE_INTEGRATION.md) - Database setup guide
