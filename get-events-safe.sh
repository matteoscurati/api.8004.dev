#!/bin/bash

# Safe script that reads credentials from environment variables
# Usage:
#   1. Create .env.test file (copy from .env.test.example)
#   2. source .env.test
#   3. ./get-events-safe.sh

set -e

# Check if environment variables are set
if [ -z "$API_PASSWORD" ]; then
    echo "❌ Error: API_PASSWORD not set!"
    echo ""
    echo "Please set environment variables first:"
    echo ""
    echo "  1. Copy .env.test.example to .env.test:"
    echo "     cp .env.test.example .env.test"
    echo ""
    echo "  2. Edit .env.test with your credentials"
    echo ""
    echo "  3. Load the environment variables:"
    echo "     source .env.test"
    echo ""
    echo "  4. Run this script again:"
    echo "     ./get-events-safe.sh"
    echo ""
    exit 1
fi

# Use environment variables with defaults
API_URL="${API_URL:-https://api-8004-dev.fly.dev}"
USERNAME="${API_USERNAME:-admin}"
PASSWORD="$API_PASSWORD"
LIMIT="${EVENT_LIMIT:-10000}"

echo "🔐 Logging in as '$USERNAME'..."

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
  echo "$LOGIN_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$LOGIN_RESPONSE"
  exit 1
fi

echo ""
echo "📡 Fetching events (limit: $LIMIT)..."

# Fetch events
EVENTS_RESPONSE=$(curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?limit=$LIMIT")

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
