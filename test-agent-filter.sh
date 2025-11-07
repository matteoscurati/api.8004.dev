#!/bin/bash
set -e

# Test script for agent_id filtering
# Usage: ./test-agent-filter.sh [agent_id]

# Load environment variables if .env.test exists
if [ -f .env.test ]; then
    source .env.test
fi

# Check for required environment variables
if [ -z "$API_URL" ]; then
    echo "❌ API_URL not set. Please set it in .env.test or export it"
    exit 1
fi

if [ -z "$API_USERNAME" ] || [ -z "$API_PASSWORD" ]; then
    echo "❌ API_USERNAME and API_PASSWORD must be set in .env.test or exported"
    exit 1
fi

AGENT_ID="${1}"

if [ -z "$AGENT_ID" ]; then
    echo "Usage: $0 <agent_id>"
    echo ""
    echo "Example: $0 3"
    echo "Example: $0 4"
    exit 1
fi

echo "🔐 Logging in..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])" 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo "❌ Login failed!"
    echo "$LOGIN_RESPONSE"
    exit 1
fi

echo "✅ Login successful"
echo ""
echo "🔍 Fetching events for agent_id: $AGENT_ID"
echo ""

# Query events for specific agent_id
RESPONSE=$(curl -s "$API_URL/events?agent_id=$AGENT_ID&limit=100" \
    -H "Authorization: Bearer $TOKEN")

# Pretty print the response
echo "$RESPONSE" | python3 -c "import sys, json; print(json.dumps(json.load(sys.stdin), indent=2))"

# Count the events
COUNT=$(echo "$RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null)

echo ""
echo "📊 Found $COUNT events for agent_id: $AGENT_ID"

# Verify all events have the correct agent_id
echo ""
echo "🔍 Verifying all events have agent_id=$AGENT_ID..."
MISMATCH=$(echo "$RESPONSE" | python3 -c "
import sys, json
data = json.load(sys.stdin)
events = data.get('events', [])
mismatches = []
for i, event in enumerate(events):
    event_data = event.get('event_data', {})
    agent_id = event_data.get('agent_id', '')
    if agent_id != '$AGENT_ID':
        mismatches.append(f'Event {i}: agent_id={agent_id}')
if mismatches:
    print('\\n'.join(mismatches))
else:
    print('OK')
")

if [ "$MISMATCH" = "OK" ]; then
    echo "✅ All events have correct agent_id!"
else
    echo "❌ Found mismatched agent_ids:"
    echo "$MISMATCH"
    exit 1
fi
