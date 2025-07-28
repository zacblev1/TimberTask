#!/bin/bash
# Test runner script for TimberTask

set -e

echo "🧪 Running TimberTask Test Suite"
echo "================================"

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Run formatting check
echo -e "\n${YELLOW}📋 Checking code formatting...${NC}"
if cargo fmt -- --check; then
    echo -e "${GREEN}✓ Code formatting check passed${NC}"
else
    echo -e "${RED}✗ Code formatting issues found. Run 'cargo fmt' to fix.${NC}"
    exit 1
fi

# Run clippy
echo -e "\n${YELLOW}🔍 Running clippy lints...${NC}"
if cargo clippy --all-targets --all-features -- -D warnings; then
    echo -e "${GREEN}✓ Clippy checks passed${NC}"
else
    echo -e "${RED}✗ Clippy found issues${NC}"
    exit 1
fi

# Run unit tests
echo -e "\n${YELLOW}🧪 Running unit tests...${NC}"
cargo test --lib --bins -- --nocapture

# Run integration tests
echo -e "\n${YELLOW}🔗 Running integration tests...${NC}"
cargo test --test integration_tests -- --nocapture

# Run all tests with coverage (if tarpaulin is installed)
if command -v cargo-tarpaulin &> /dev/null; then
    echo -e "\n${YELLOW}📊 Running tests with coverage...${NC}"
    cargo tarpaulin --out Html --output-dir target/coverage
    echo -e "${GREEN}✓ Coverage report generated at target/coverage/tarpaulin-report.html${NC}"
else
    echo -e "\n${YELLOW}ℹ️  Install cargo-tarpaulin for coverage reports:${NC}"
    echo "  cargo install cargo-tarpaulin"
fi

# Run benchmarks (if any)
if ls benches/*.rs 1> /dev/null 2>&1; then
    echo -e "\n${YELLOW}⚡ Running benchmarks...${NC}"
    cargo bench
fi

echo -e "\n${GREEN}✅ All tests passed!${NC}"

# Summary
echo -e "\n${YELLOW}📈 Test Summary:${NC}"
cargo test --lib --bins 2>&1 | grep -E "test result:|running" | tail -2

# Check test count
TEST_COUNT=$(cargo test -- --list 2>&1 | grep -E "test$" | wc -l | tr -d ' ')
echo -e "\nTotal test count: ${GREEN}$TEST_COUNT${NC} tests"

# Suggest next steps
echo -e "\n${YELLOW}💡 Next steps:${NC}"
echo "  - Run specific test: cargo test test_name"
echo "  - Run tests in watch mode: cargo watch -x test"
echo "  - Generate detailed coverage: cargo tarpaulin --out Html"
echo "  - Run tests with output: cargo test -- --nocapture"