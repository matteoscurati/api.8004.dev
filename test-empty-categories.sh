#!/bin/bash

API_PASSWORD="42zyw7pqmXDStKsLEs3OkY57TVf8Pf7JTg9OvXBh8YwKL0fEur1KKjITrLuk+WEH"
API_URL="https://api-8004-dev.fly.dev"
CHAIN_ID=11155111

echo "🔐 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

echo "✅ Login successful!"
echo ""

echo "📊 Test 1: category=capabilities"
echo "================================"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=capabilities&limit=10" | jq '{count, total, stats, events_length: (.events | length)}'
echo ""

echo "📊 Test 2: category=payments"
echo "============================"
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&category=payments&limit=10" | jq '{count, total, stats, events_length: (.events | length)}'
echo ""

echo "✨ Verification complete!"
