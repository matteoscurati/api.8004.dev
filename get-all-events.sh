#!/bin/bash

# Script to fetch all events from API 8004.dev
# Usage: ./get-all-events.sh [username] [password] [api-url] [chain-id] [limit]

set -e

# Configuration
USERNAME="${1:-admin}"
PASSWORD="${2}"
API_URL="${3:-https://api-8004-dev.fly.dev}"
CHAIN_ID="${4:-11155111}"
LIMIT="${5:-10000}"

echo "🔐 Logging in as '$USERNAME'..."

# Check if password is provided
if [ -z "$PASSWORD" ]; then
    echo "❌ Error: Password required!"
    echo ""
    echo "Usage: $0 <username> <password> [api-url] [chain-id] [limit]"
    echo ""
    echo "Example:"
    echo "  $0 admin 'your-password' https://api-8004-dev.fly.dev 11155111 10000"
    echo ""
    echo "Default chain-id: 11155111 (Sepolia)"
    echo ""
    exit 1
fi

# Login and get token
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

# Check if login was successful
if echo "$LOGIN_RESPONSE" | grep -q "token"; then
  TOKEN=$(echo "$LOGIN_RESPONSE" | grep -o '"token":"[^"]*' | cut -d'"' -f4)
  echo "✅ Login successful!"
else
  echo "❌ Login failed!"
  echo "$LOGIN_RESPONSE"
  exit 1
fi

echo ""
echo "📡 Fetching events from chain $CHAIN_ID (limit: $LIMIT)..."

# Fetch events
EVENTS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?chain_id=$CHAIN_ID&limit=$LIMIT")

# Check if jq is available for pretty printing
if command -v jq &> /dev/null; then
  echo "$EVENTS_RESPONSE" | jq '.'

  # Extract count
  COUNT=$(echo "$EVENTS_RESPONSE" | jq -r '.count // 0')
  echo ""
  echo "📊 Total events fetched: $COUNT"

  # Save to file
  FILENAME="events_$(date +%Y%m%d_%H%M%S).json"
  echo "$EVENTS_RESPONSE" | jq '.' > "$FILENAME"
  echo "💾 Saved to: $FILENAME"
else
  echo "$EVENTS_RESPONSE"

  # Save to file
  FILENAME="events_$(date +%Y%m%d_%H%M%S).json"
  echo "$EVENTS_RESPONSE" > "$FILENAME"
  echo ""
  echo "💾 Saved to: $FILENAME"
  echo "ℹ️  Install 'jq' for pretty printing: brew install jq"
fi

echo ""
echo "✨ Done!"
