#!/bin/bash

# Script to test category filtering on API 8004.dev

set -e

API_PASSWORD="42zyw7pqmXDStKsLEs3OkY57TVf8Pf7JTg9OvXBh8YwKL0fEur1KKjITrLuk+WEH"
API_URL="https://api-8004-dev.fly.dev"
CHAIN_ID=11155111

echo "🔐 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

if [ -z "$TOKEN" ]; then
    echo "❌ Login failed!"
    exit 1
fi

echo "✅ Login successful!"
echo ""

# Test 1: All events (no category filter)
echo "📊 Test 1: GET /events?chain_id=$CHAIN_ID&limit=3 (no category)"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&limit=3" | jq '{count, total, stats}'
echo ""

# Test 2: Agents category
echo "📊 Test 2: GET /events?chain_id=$CHAIN_ID&category=agents&limit=3"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=agents&limit=3" | jq '{count, total, stats, events: [.events[0].event_type]}'
echo ""

# Test 3: Metadata category
echo "📊 Test 3: GET /events?chain_id=$CHAIN_ID&category=metadata&limit=3"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=metadata&limit=3" | jq '{count, total, stats, events: [.events[0].event_type]}'
echo ""

# Test 4: Validation category
echo "📊 Test 4: GET /events?chain_id=$CHAIN_ID&category=validation&limit=3"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=validation&limit=3" | jq '{count, total, stats, events: [.events[0].event_type]}'
echo ""

# Test 5: Feedback category
echo "📊 Test 5: GET /events?chain_id=$CHAIN_ID&category=feedback&limit=3"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=feedback&limit=3" | jq '{count, total, stats, events: [.events[0].event_type]}'
echo ""

# Test 6: Category "all"
echo "📊 Test 6: GET /events?chain_id=$CHAIN_ID&category=all&limit=3"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=all&limit=3" | jq '{count, total, stats}'
echo ""

echo "✨ All tests completed!"
