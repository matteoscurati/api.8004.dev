#!/bin/bash

# Test all API endpoints
set -e

API_URL="https://api-8004-dev.fly.dev"
PASSWORD="5tkqFytIWZfiLV33IcHkSz0B7T5Z2kCHzwFVQV9RMDq5VXLae7vzbB9ulRZfK+7/"

echo "========================================="
echo "Testing API Endpoints"
echo "========================================="
echo ""

# Test 1: Health endpoint
echo "✅ Test 1: Health endpoint"
curl -s "$API_URL/health" | python3 -m json.tool
echo ""
echo ""

# Test 2: Detailed health endpoint
echo "✅ Test 2: Detailed health endpoint"
curl -s "$API_URL/health/detailed" | python3 -m json.tool
echo ""
echo ""

# Test 3: Login endpoint
echo "✅ Test 3: Login endpoint"
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"$PASSWORD\"}")
echo "$LOGIN_RESPONSE" | python3 -m json.tool
TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])" 2>/dev/null)
echo ""
echo ""

if [ -z "$TOKEN" ]; then
    echo "❌ Failed to get token, stopping tests"
    exit 1
fi

# Test 4: Stats endpoint (requires auth)
echo "✅ Test 4: Stats endpoint (authenticated)"
curl -s "$API_URL/stats" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
echo ""
echo ""

# Test 5: Events endpoint (requires auth)
echo "✅ Test 5: Events endpoint (authenticated, limit 5)"
curl -s "$API_URL/events?limit=5" \
  -H "Authorization: Bearer $TOKEN" | python3 -m json.tool
echo ""
echo ""

# Test 6: Metrics endpoint (public)
echo "✅ Test 6: Metrics endpoint (first 20 lines)"
curl -s "$API_URL/metrics" | head -20
echo ""
echo "... (truncated)"
echo ""
echo ""

echo "========================================="
echo "✅ All tests completed successfully!"
echo "========================================="
