#!/bin/bash

API_PASSWORD="42zyw7pqmXDStKsLEs3OkY57TVf8Pf7JTg9OvXBh8YwKL0fEur1KKjITrLuk+WEH"
API_URL="https://api-8004-dev.fly.dev"

echo "🔐 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"admin\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)

echo "✅ Login successful!"
echo ""

echo "📊 Getting all events and checking event types..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=11155111&limit=5000" | \
  jq -r '.events[].event_type.type' | sort | uniq -c | sort -rn

echo ""
echo "✨ Done!"
