#!/bin/bash
# Quick production status check

API_URL="https://api-8004-dev.fly.dev"

echo "🔍 Quick Production Check"
echo "========================="
echo ""

# Health check
echo "1️⃣  Health Status:"
curl -s "$API_URL/health" | jq '.'
echo ""

# Detailed health
echo "2️⃣  Detailed Health:"
curl -s "$API_URL/health/detailed" | jq '{status, checks: {database, cache}}'
echo ""

# Chains list (no auth required)
echo "3️⃣  Chains Status:"
curl -s "$API_URL/chains" | jq '.'
echo ""

# Login and get events count
if [ -f ".env.test" ]; then
    source .env.test

    echo "4️⃣  Events Count (requires auth):"
    LOGIN=$(curl -s -X POST "$API_URL/login" \
      -H "Content-Type: application/json" \
      -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}")

    TOKEN=$(echo "$LOGIN" | jq -r '.token')

    if [ "$TOKEN" != "null" ] && [ -n "$TOKEN" ]; then
        # Ethereum Sepolia
        echo "   • Ethereum Sepolia (11155111):"
        curl -s "$API_URL/events?chain_id=11155111&limit=0" \
          -H "Authorization: Bearer $TOKEN" | jq '{total: .total}'

        # Base Sepolia
        echo "   • Base Sepolia (84532):"
        curl -s "$API_URL/events?chain_id=84532&limit=0" \
          -H "Authorization: Bearer $TOKEN" | jq '{total: .total}'
    else
        echo "   ❌ Login failed"
    fi
else
    echo "4️⃣  Events Count: Skipped (.env.test not found)"
fi

echo ""
echo "✅ Check complete"
