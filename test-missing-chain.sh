#!/bin/bash
set -e

# Test script for missing chain_id parameter
# Should return an error

# Load environment variables
if [ -f .env.test ]; then
    source .env.test
fi

echo "🔐 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])" 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo "❌ Login failed!"
    exit 1
fi

echo "✅ Login successful"
echo ""
echo "🧪 Testing query WITHOUT chain_id (should fail)..."
echo ""

RESPONSE=$(curl -s "$API_URL/events?agent_id=3" \
    -H "Authorization: Bearer $TOKEN")

echo "$RESPONSE"
echo ""

# Check if it's an error response
if echo "$RESPONSE" | grep -q "error\|failed\|missing\|required"; then
    echo "✅ Correctly returned error for missing chain_id"
else
    echo "❌ Should have returned an error but got a successful response"
    exit 1
fi
