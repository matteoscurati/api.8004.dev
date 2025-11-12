#!/bin/bash
# Test RPC endpoint connectivity for all configured chains

set -e

echo "=========================================="
echo "RPC Endpoint Connectivity Test"
echo "=========================================="
echo ""

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counters
TOTAL=0
SUCCESS=0
FAILED=0

# Function to test an RPC endpoint
test_rpc() {
    local chain_name="$1"
    local rpc_url="$2"
    local provider_index="$3"

    TOTAL=$((TOTAL + 1))

    echo -n "  [$provider_index] Testing: ${rpc_url:0:60}..."

    # Make eth_blockNumber call with timeout and measure latency
    START_TIME=$(python3 -c 'import time; print(int(time.time() * 1000))' 2>/dev/null || echo "0")
    RESPONSE=$(curl -s -m 10 -X POST "$rpc_url" \
        -H "Content-Type: application/json" \
        -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}' 2>&1 || echo '{"error":"curl failed"}')
    END_TIME=$(python3 -c 'import time; print(int(time.time() * 1000))' 2>/dev/null || echo "$START_TIME")
    LATENCY=$((END_TIME - START_TIME))

    # Check if response contains a result
    if echo "$RESPONSE" | grep -q '"result"'; then
        BLOCK_NUMBER=$(echo "$RESPONSE" | grep -o '"result":"[^"]*"' | cut -d'"' -f4)
        # Convert hex to decimal safely
        if [[ "$BLOCK_NUMBER" =~ ^0x[0-9a-fA-F]+$ ]]; then
            BLOCK_DEC=$((16#${BLOCK_NUMBER#0x}))
            echo -e " ${GREEN}✓${NC} (${LATENCY}ms, block: $BLOCK_DEC)"
            SUCCESS=$((SUCCESS + 1))
            return 0
        else
            echo -e " ${RED}✗${NC} Invalid block number format"
            FAILED=$((FAILED + 1))
            return 1
        fi
    else
        echo -e " ${RED}✗${NC} (${LATENCY}ms)"
        ERROR_MSG=$(echo "$RESPONSE" | head -c 80)
        echo "    Error: $ERROR_MSG"
        FAILED=$((FAILED + 1))
        return 0  # Return 0 to continue testing other endpoints
    fi
}

# Parse chains.yaml using Ruby
ruby << 'RUBY_SCRIPT'
require 'yaml'

config = YAML.load_file('chains.yaml')
chains = config['chains'] || []

chains.each do |chain|
  next unless chain['enabled']

  puts ""
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  puts "Chain: #{chain['name']} (ID: #{chain['chain_id']})"
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  # Handle both old (rpc_url) and new (rpc_providers) formats
  if chain['rpc_providers'] && chain['rpc_providers'].length > 0
    chain['rpc_providers'].each_with_index do |provider, index|
      puts "#{chain['name']}\t#{provider['url']}\t#{index + 1}"
    end
  elsif chain['rpc_url']
    puts "#{chain['name']}\t#{chain['rpc_url']}\t1"
  end
end
RUBY_SCRIPT

# Read the Ruby output and test each endpoint
while IFS=$'\t' read -r chain_name rpc_url provider_index; do
    if [ -n "$chain_name" ] && [ -n "$rpc_url" ]; then
        # Skip header lines
        if [[ "$chain_name" =~ ^━ ]] || [[ "$chain_name" == "Chain:"* ]]; then
            echo "$chain_name"
            continue
        fi

        test_rpc "$chain_name" "$rpc_url" "$provider_index"
    fi
done < <(ruby << 'RUBY_SCRIPT'
require 'yaml'

config = YAML.load_file('chains.yaml')
chains = config['chains'] || []

chains.each do |chain|
  next unless chain['enabled']

  puts ""
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
  puts "Chain: #{chain['name']} (ID: #{chain['chain_id']})"
  puts "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

  # Handle both old (rpc_url) and new (rpc_providers) formats
  if chain['rpc_providers'] && chain['rpc_providers'].length > 0
    chain['rpc_providers'].each_with_index do |provider, index|
      puts "#{chain['name']}\t#{provider['url']}\t#{index + 1}"
    end
  elsif chain['rpc_url']
    puts "#{chain['name']}\t#{chain['rpc_url']}\t1"
  end
end
RUBY_SCRIPT
)

echo ""
echo "=========================================="
echo "Test Summary"
echo "=========================================="
echo -e "Total endpoints tested: $TOTAL"
echo -e "${GREEN}Successful: $SUCCESS${NC}"
echo -e "${RED}Failed: $FAILED${NC}"

if [ $FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}✓ All RPC endpoints are working!${NC}"
    exit 0
else
    echo ""
    echo -e "${YELLOW}⚠  Some RPC endpoints failed. Check the output above for details.${NC}"
    echo -e "${YELLOW}   Consider switching to working providers in chains.yaml${NC}"
    exit 1
fi
