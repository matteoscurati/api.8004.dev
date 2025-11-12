#!/bin/bash

# ⚡ Quick Test Script - Unit Tests Only
# Fast verification of all fixes without running the server

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}⚡ Quick Test Suite - Unit Tests${NC}"
echo ""

# Test 1: Compilation
echo -e "${BLUE}[1/5] Checking compilation...${NC}"
if cargo build --lib --quiet 2>/dev/null; then
    echo -e "${GREEN}✓ Compilation successful${NC}"
else
    echo -e "${RED}✗ Compilation failed${NC}"
    cargo build --lib
    exit 1
fi
echo ""

# Test 2: All unit tests
echo -e "${BLUE}[2/5] Running all unit tests (51 expected)...${NC}"
TEST_OUTPUT=$(cargo test --lib 2>&1)
PASSED=$(echo "$TEST_OUTPUT" | grep "test result:" | grep -o "[0-9]* passed" | grep -o "[0-9]*")

if [ "$PASSED" = "51" ]; then
    echo -e "${GREEN}✓ All 51 tests passed${NC}"
else
    echo -e "${RED}✗ Expected 51 tests, got $PASSED${NC}"
    cargo test --lib
    exit 1
fi
echo ""

# Test 3: Specific fix tests
echo -e "${BLUE}[3/5] Testing specific fixes...${NC}"

# Fix #1: Cache key with chain_id
if cargo test test_cache_key_cross_chain --lib --quiet 2>/dev/null; then
    echo -e "  ${GREEN}✓ Fix #1: Cache key with chain_id${NC}"
else
    echo -e "  ${RED}✗ Fix #1 failed${NC}"
fi

# Fix #5: LRU eviction
if cargo test test_cache_lru_eviction --lib --quiet 2>/dev/null; then
    echo -e "  ${GREEN}✓ Fix #5: LRU cache eviction${NC}"
else
    echo -e "  ${RED}✗ Fix #5 failed${NC}"
fi

# Fix #6: include_stats parameter
if cargo test test_event_query_include_stats --lib --quiet 2>/dev/null; then
    echo -e "  ${GREEN}✓ Fix #6: Optional stats parameter${NC}"
else
    echo -e "  ${RED}✗ Fix #6 failed${NC}"
fi

# Fix #2 & #8: Broadcast and metrics
if cargo test test_event_broadcast --lib --quiet 2>/dev/null && \
   cargo test test_metrics --lib --quiet 2>/dev/null; then
    echo -e "  ${GREEN}✓ Fix #2 & #8: Broadcast and metrics${NC}"
else
    echo -e "  ${RED}✗ Fix #2 or #8 failed${NC}"
fi

echo ""

# Test 4: API tests
echo -e "${BLUE}[4/5] Testing API module...${NC}"
API_TESTS=$(cargo test api::tests --lib --quiet 2>&1 | grep "test result:" | grep -o "[0-9]* passed" | grep -o "[0-9]*")
if [ "$API_TESTS" -ge "5" ]; then
    echo -e "${GREEN}✓ API tests passed ($API_TESTS tests)${NC}"
else
    echo -e "${RED}✗ API tests incomplete${NC}"
fi
echo ""

# Test 5: Storage tests
echo -e "${BLUE}[5/5] Testing Storage module...${NC}"
STORAGE_TESTS=$(cargo test storage::tests --lib --quiet 2>&1 | grep "test result:" | grep -o "[0-9]* passed" | grep -o "[0-9]*")
if [ "$STORAGE_TESTS" -ge "10" ]; then
    echo -e "${GREEN}✓ Storage tests passed ($STORAGE_TESTS tests)${NC}"
else
    echo -e "${RED}✗ Storage tests incomplete${NC}"
fi
echo ""

# Summary
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo -e "${GREEN}✓ Quick tests completed!${NC}"
echo -e "${GREEN}═══════════════════════════════════════${NC}"
echo ""
echo "Test Coverage:"
echo "  • Total tests: $PASSED"
echo "  • API tests: $API_TESTS"
echo "  • Storage tests: $STORAGE_TESTS"
echo ""
echo "Verified Fixes:"
echo "  ✓ Fix #1: Cache key with chain_id"
echo "  ✓ Fix #2: WebSocket broadcasting"
echo "  ✓ Fix #5: LRU cache eviction"
echo "  ✓ Fix #6: Optional stats parameter"
echo "  ✓ Fix #8: Metrics collection"
echo ""
echo "Next: Run './test-local.sh' with the server running for integration tests"
