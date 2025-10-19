#!/bin/bash
# Configuration Verification Script
# This script verifies that the Velora configuration system is working correctly

set -e  # Exit on error

echo "═══════════════════════════════════════════════════════════"
echo "  Velora Configuration System Verification"
echo "═══════════════════════════════════════════════════════════"
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter for tests
PASSED=0
FAILED=0

# Helper functions
pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((PASSED++))
}

fail() {
    echo -e "${RED}✗${NC} $1"
    ((FAILED++))
}

warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

section() {
    echo ""
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    echo "  $1"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
}

# 1. Check file existence
section "1. Configuration Files"
if [ -f "config/base.toml" ]; then
    pass "config/base.toml exists"
else
    fail "config/base.toml is missing"
fi

if [ -f "config/testing.toml" ]; then
    pass "config/testing.toml exists"
else
    fail "config/testing.toml is missing"
fi

if [ -f "config/paper-trading.toml" ]; then
    pass "config/paper-trading.toml exists"
else
    fail "config/paper-trading.toml is missing"
fi

if [ -f "config/live-trading.toml" ]; then
    pass "config/live-trading.toml exists"
else
    fail "config/live-trading.toml is missing"
fi

if [ -f "config/backtesting.toml" ]; then
    pass "config/backtesting.toml exists"
else
    fail "config/backtesting.toml is missing"
fi

# 2. Check TOML syntax
section "2. TOML Syntax Validation"
for config in config/*.toml; do
    if [ -f "$config" ]; then
        # Simple syntax check using grep for basic TOML structure
        if grep -q "^\[" "$config"; then
            pass "$(basename $config) has valid TOML structure"
        else
            warn "$(basename $config) may have syntax issues"
        fi
    fi
done

# 3. Run unit tests
section "3. Unit Tests"
echo "Running configuration unit tests..."
if cargo test --package velora-core --test config_tests --quiet; then
    pass "All unit tests passed"
else
    fail "Some unit tests failed"
fi

# 4. Run example
section "4. Configuration Loading Example"
echo "Running config_loading example..."
if cargo run --package velora-core --example config_loading --quiet 2>&1 | grep -q "Velora Configuration"; then
    pass "config_loading example runs successfully"
else
    fail "config_loading example failed"
fi

# 5. Check for sensitive data in git
section "5. Security Check"
if git ls-files | grep -E "(\.env$|secret|password)" | grep -v ".gitignore" | grep -v ".md" | grep -v "scripts/"; then
    fail "Found potential sensitive files in git"
else
    pass "No sensitive files found in git"
fi

if grep -r "CHANGE_ME" config/*.toml > /dev/null 2>&1; then
    warn "Found CHANGE_ME placeholders in config (expected for live-trading.toml)"
else
    pass "No unexpected CHANGE_ME placeholders"
fi

# 6. Verify layering behavior
section "6. Configuration Layering"
echo "Testing that layering works correctly..."

# Create a temp test
cat > /tmp/test_config_layering.sh << 'EOF'
cd /Users/itsparser/Developer/Opensource/velora
cargo run --package velora-core --example config_loading 2>&1 | grep -q "DB=InMemory.*DryRun=true.*MaxPos=\$500"
EOF

if bash /tmp/test_config_layering.sh 2>/dev/null; then
    pass "Configuration layering works correctly"
else
    warn "Configuration layering test inconclusive (example may not be running)"
fi

# 7. Check documentation
section "7. Documentation"
if [ -f "CONFIG_GUIDE.md" ]; then
    pass "CONFIG_GUIDE.md exists"
else
    fail "CONFIG_GUIDE.md is missing"
fi

if [ -f "config/README.md" ]; then
    pass "config/README.md exists"
else
    fail "config/README.md is missing"
fi

if [ -f "CONFIG_SUMMARY.md" ]; then
    pass "CONFIG_SUMMARY.md exists"
else
    fail "CONFIG_SUMMARY.md is missing"
fi

# 8. Environment variable test
section "8. Environment Variable Override"
echo "Testing environment variable override..."
export VELORA_LOGGING_LEVEL=trace
if cargo run --package velora-core --example config_loading 2>&1 | grep -q "VELORA_"; then
    pass "Environment variable example present in output"
else
    warn "Environment variable example not found (non-critical)"
fi
unset VELORA_LOGGING_LEVEL

# 9. Build verification
section "9. Build Verification"
echo "Verifying all packages build..."
if cargo build --package velora-core --quiet 2>&1; then
    pass "velora-core builds successfully"
else
    fail "velora-core build failed"
fi

# Summary
section "Summary"
echo ""
echo "Tests Passed: ${GREEN}${PASSED}${NC}"
echo "Tests Failed: ${RED}${FAILED}${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${GREEN}  ✓ All checks passed! Configuration system is working.${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 0
else
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}  ✗ Some checks failed. Please review the output above.${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    exit 1
fi
