#!/bin/bash
set -e

# Test script for pagination
# Usage: ./test-pagination.sh

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

CHAIN_ID=11155111

echo "📄 Page 1 (offset=0, limit=3):"
echo "================================"
RESPONSE1=$(curl -s "$API_URL/events?chain_id=$CHAIN_ID&limit=3&offset=0" \
    -H "Authorization: Bearer $TOKEN")

echo "$RESPONSE1" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(f\"Count: {data['count']}\")
print(f\"Total: {data['total']}\")
print(f\"Pagination: {data['pagination']}\")
print(f\"Event IDs: {[e['id'] for e in data['events']]}\")
"

echo ""
echo "📄 Page 2 (offset=3, limit=3):"
echo "================================"
RESPONSE2=$(curl -s "$API_URL/events?chain_id=$CHAIN_ID&limit=3&offset=3" \
    -H "Authorization: Bearer $TOKEN")

echo "$RESPONSE2" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(f\"Count: {data['count']}\")
print(f\"Total: {data['total']}\")
print(f\"Pagination: {data['pagination']}\")
print(f\"Event IDs: {[e['id'] for e in data['events']]}\")
"

echo ""
echo "📄 Page 3 (offset=6, limit=3):"
echo "================================"
RESPONSE3=$(curl -s "$API_URL/events?chain_id=$CHAIN_ID&limit=3&offset=6" \
    -H "Authorization: Bearer $TOKEN")

echo "$RESPONSE3" | python3 -c "
import sys, json
data = json.load(sys.stdin)
print(f\"Count: {data['count']}\")
print(f\"Total: {data['total']}\")
print(f\"Pagination: {data['pagination']}\")
print(f\"Event IDs: {[e['id'] for e in data['events']]}\")
"

echo ""
echo "✅ Pagination test completed!"
