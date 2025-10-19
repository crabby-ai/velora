# Velora Exchange Examples

This directory contains example programs demonstrating how to use the Velora exchange integrations.

## Available Examples

### 1. Lighter Exchange Validation (`lighter_validation.rs`)

Validates all Lighter exchange API endpoints to ensure they work correctly with the mainnet API.

**Features:**
- Tests all market data endpoints
- Validates API response formats
- Provides detailed output for debugging
- Works in read-only mode (no authentication required for market data)

**Usage:**

```bash
# 1. Copy the example environment file
cp .env.example .env

# 2. (Optional) Add your Lighter credentials to .env
#    Only needed for trading/account endpoints
nano .env

# 3. Run the validation
cargo run --example lighter_validation --features lighter
```

**What it tests:**

1. ✅ `get_markets()` - Fetch all available trading markets
2. ✅ `get_market()` - Get specific market details
3. ✅ `get_ticker()` - Fetch real-time ticker data
4. ✅ `get_tickers()` - Get tickers for all markets
5. ✅ `get_orderbook()` - Retrieve order book with depth
6. ✅ `get_recent_trades()` - Fetch recent trade history
7. ✅ `get_candles()` - Get candlestick/OHLCV data
8. ✅ `get_funding_rate()` - Current funding rate (perpetuals)
9. ✅ `get_funding_rate_history()` - Historical funding rates

**Expected Output:**

```
=== Lighter Exchange API Validation ===

📡 Connecting to Lighter mainnet...
Configuration:
  API URL: https://mainnet.zklighter.elliot.ai
  WS Host: mainnet.zklighter.elliot.ai
  API Key: ✗ Not set
  Account Index: None

✅ Connected to Lighter Exchange

🔍 Validating Market Data Endpoints

1️⃣  Testing get_markets()...
   ✅ Success! Found 25 markets
   📊 Sample markets:
      - BTC-USDT-PERP (BTC/USDT)
      - ETH-USDT-PERP (ETH/USDT)
      - SOL-USDT-PERP (SOL/USDT)

...

=== Validation Complete ===
All API endpoints are working correctly! ✨
```

## Environment Variables

Create a `.env` file in the project root with the following variables:

```env
# Lighter Exchange Configuration
LIGHTER_API_URL=https://mainnet.zklighter.elliot.ai
LIGHTER_CHAIN_ID=304
LIGHTER_WS_HOST=mainnet.zklighter.elliot.ai

# Optional: For authenticated endpoints (trading/account)
LIGHTER_API_KEY=0x...
LIGHTER_ACCOUNT_INDEX=12345
LIGHTER_API_KEY_INDEX=0
```

## Troubleshooting

### Connection Errors

If you see connection errors:
1. Check your internet connection
2. Verify the API URL is correct
3. Check if Lighter mainnet is operational

### API Errors

If specific endpoints fail:
1. The endpoint might not be available in this API version
2. The symbol format might be incorrect
3. Rate limiting might be in effect

### Missing Dependencies

If you see compilation errors:
```bash
cargo clean
cargo build --example lighter_validation --features lighter
```

## Adding More Examples

To add a new example:

1. Create a new file in this directory: `examples/my_example.rs`
2. Add the example code with proper documentation
3. Test it: `cargo run --example my_example --features lighter`
4. Update this README with the new example

## Next Steps

- [ ] Add trading example (requires authentication)
- [ ] Add account management example
- [ ] Add streaming/WebSocket example
- [ ] Add Paradex exchange examples
- [ ] Add multi-exchange arbitrage example

## Contributing

When contributing examples:
- Include comprehensive error handling
- Add detailed comments explaining the code
- Use the validation pattern to check API responses
- Document all environment variables needed
- Test with mainnet before submitting
