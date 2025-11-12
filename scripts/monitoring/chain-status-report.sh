#!/bin/bash

# Chain Status Report Script
# Queries the /chains/status endpoint and displays a formatted report

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
API_USERNAME="${API_USERNAME:-admin}"
API_PASSWORD="${API_PASSWORD:-}"

# Function to get blocks per minute for each chain
# Source: Official chain documentation and block explorers (January 2025)
get_blocks_per_minute() {
    case "$1" in
        11155111) echo "5" ;;   # Ethereum Sepolia: ~12s block time
        84532) echo "30" ;;     # Base Sepolia: ~2s block time
        59141) echo "30" ;;     # Linea Sepolia: ~2s block time
        80002) echo "30" ;;     # Polygon Amoy: ~2s block time
        296) echo "30" ;;       # Hedera Testnet: ~2s block time
        *) echo "N/A" ;;
    esac
}

# Check if required tools are installed
command -v curl >/dev/null 2>&1 || { echo "Error: curl is required but not installed."; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed."; exit 1; }
command -v bc >/dev/null 2>&1 || { echo "Error: bc is required but not installed. Install with: brew install bc (macOS) or apt-get install bc (Linux)"; exit 1; }

# Check if password is set
if [ -z "$API_PASSWORD" ]; then
    echo "Error: API_PASSWORD environment variable must be set"
    exit 1
fi

echo -e "${CYAN}=== ERC-8004 Multi-Chain Indexer Status Report ===${NC}"
echo ""

# Login to get JWT token
echo -e "${BLUE}Authenticating...${NC}"
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | jq -r '.token')

if [ "$TOKEN" = "null" ] || [ -z "$TOKEN" ]; then
    echo -e "${RED}Authentication failed!${NC}"
    echo "$LOGIN_RESPONSE" | jq .
    exit 1
fi

echo -e "${GREEN}Authenticated successfully${NC}"
echo ""

# Get chains status
echo -e "${BLUE}Fetching chains status...${NC}"
STATUS_RESPONSE=$(curl -s "$API_URL/chains/status" \
    -H "Authorization: Bearer $TOKEN")

# Check if request was successful
SUCCESS=$(echo "$STATUS_RESPONSE" | jq -r '.success')

if [ "$SUCCESS" != "true" ]; then
    echo -e "${RED}Failed to fetch chain status!${NC}"
    echo "$STATUS_RESPONSE" | jq .
    exit 1
fi

echo ""
echo -e "${CYAN}╔════════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║                         CHAINS STATUS REPORT                           ║${NC}"
echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Get timestamp
TIMESTAMP=$(echo "$STATUS_RESPONSE" | jq -r '.timestamp')
echo -e "Report generated at: ${YELLOW}$TIMESTAMP${NC}"
echo ""

# Get number of chains
CHAIN_COUNT=$(echo "$STATUS_RESPONSE" | jq '.chains | length')

if [ "$CHAIN_COUNT" -eq 0 ]; then
    echo -e "${YELLOW}No chains configured${NC}"
    exit 0
fi

# Loop through each chain
for i in $(seq 0 $((CHAIN_COUNT - 1))); do
    CHAIN=$(echo "$STATUS_RESPONSE" | jq ".chains[$i]")

    # Extract chain data
    CHAIN_ID=$(echo "$CHAIN" | jq -r '.chain_id')
    CHAIN_NAME=$(echo "$CHAIN" | jq -r '.name')
    STATUS=$(echo "$CHAIN" | jq -r '.status')
    CURRENT_BLOCK=$(echo "$CHAIN" | jq -r '.blocks.current')
    INDEXED_BLOCK=$(echo "$CHAIN" | jq -r '.blocks.indexed')
    BLOCKS_BEHIND=$(echo "$CHAIN" | jq -r '.blocks.behind')
    POLLING_RATE=$(echo "$CHAIN" | jq -r '.polling.rate_per_minute')
    TOTAL_EVENTS=$(echo "$CHAIN" | jq -r '.events.total')
    LAST_SYNC=$(echo "$CHAIN" | jq -r '.last_sync_time')

    # Event counts by type
    REGISTERED=$(echo "$CHAIN" | jq -r '.events.by_type.registered')
    METADATA_SET=$(echo "$CHAIN" | jq -r '.events.by_type.metadata_set')
    URI_UPDATED=$(echo "$CHAIN" | jq -r '.events.by_type.uri_updated')
    NEW_FEEDBACK=$(echo "$CHAIN" | jq -r '.events.by_type.new_feedback')
    FEEDBACK_REVOKED=$(echo "$CHAIN" | jq -r '.events.by_type.feedback_revoked')
    RESPONSE_APPENDED=$(echo "$CHAIN" | jq -r '.events.by_type.response_appended')
    VALIDATION_REQUEST=$(echo "$CHAIN" | jq -r '.events.by_type.validation_request')
    VALIDATION_RESPONSE=$(echo "$CHAIN" | jq -r '.events.by_type.validation_response')

    # Status color
    STATUS_COLOR=$GREEN
    if [ "$STATUS" = "failed" ]; then
        STATUS_COLOR=$RED
    elif [ "$STATUS" = "stalled" ]; then
        STATUS_COLOR=$YELLOW
    elif [ "$STATUS" = "syncing" ]; then
        STATUS_COLOR=$YELLOW
    fi

    # Blocks behind indicator
    BEHIND_COLOR=$GREEN
    if [ "$BLOCKS_BEHIND" = "null" ]; then
        BLOCKS_BEHIND="N/A"
        BEHIND_COLOR=$YELLOW
    elif [ "$BLOCKS_BEHIND" -gt 100 ]; then
        BEHIND_COLOR=$RED
    elif [ "$BLOCKS_BEHIND" -gt 10 ]; then
        BEHIND_COLOR=$YELLOW
    fi

    # Print chain header
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}📊 Chain:${NC} ${YELLOW}$CHAIN_NAME${NC} ${CYAN}(ID: $CHAIN_ID)${NC}"
    echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""

    # Status
    echo -e "  ${CYAN}Status:${NC} ${STATUS_COLOR}$STATUS${NC}"
    echo ""

    # Get blocks per minute for this chain
    CHAIN_BLOCKS_PER_MIN=$(get_blocks_per_minute "$CHAIN_ID")

    # Calculate estimated catch-up time
    CATCH_UP_TIME="N/A"
    if [ "$BLOCKS_BEHIND" != "N/A" ] && [ "$BLOCKS_BEHIND" != "null" ] && [ "$BLOCKS_BEHIND" -gt 0 ]; then
        if [ "$CHAIN_BLOCKS_PER_MIN" != "N/A" ]; then
            # Net catch-up rate = polling_rate - chain_production_rate
            # If negative, we're falling behind
            POLLING_RATE_NUM=$(echo "$POLLING_RATE" | awk '{printf "%.0f", $1}')
            NET_CATCH_UP_RATE=$((POLLING_RATE_NUM - CHAIN_BLOCKS_PER_MIN))

            if [ "$NET_CATCH_UP_RATE" -gt 0 ]; then
                # Calculate minutes to catch up
                MINUTES_TO_CATCH_UP=$(echo "scale=1; $BLOCKS_BEHIND / $NET_CATCH_UP_RATE" | bc)

                # Convert to hours if > 60 minutes
                if (( $(echo "$MINUTES_TO_CATCH_UP > 60" | bc -l) )); then
                    HOURS=$(echo "scale=1; $MINUTES_TO_CATCH_UP / 60" | bc)
                    CATCH_UP_TIME="${HOURS}h"
                else
                    CATCH_UP_TIME="${MINUTES_TO_CATCH_UP}m"
                fi
            elif [ "$NET_CATCH_UP_RATE" -eq 0 ]; then
                CATCH_UP_TIME="∞ (staying at same distance)"
            else
                CATCH_UP_TIME="⚠️  Falling behind"
            fi
        fi
    elif [ "$BLOCKS_BEHIND" = "0" ]; then
        CATCH_UP_TIME="✓ Synced"
    fi

    # Blocks section
    echo -e "  ${CYAN}⛓️  Blocks:${NC}"
    echo -e "    • Current Block:     ${YELLOW}$CURRENT_BLOCK${NC}"
    echo -e "    • Indexed Block:     ${GREEN}$INDEXED_BLOCK${NC}"
    echo -e "    • Blocks Behind:     ${BEHIND_COLOR}$BLOCKS_BEHIND${NC}"
    if [ "$CHAIN_BLOCKS_PER_MIN" != "N/A" ]; then
        echo -e "    • Chain Rate:        ${CYAN}$CHAIN_BLOCKS_PER_MIN${NC} blocks/min"
    fi
    if [ "$CATCH_UP_TIME" != "N/A" ]; then
        echo -e "    • Est. Catch-up:     ${YELLOW}$CATCH_UP_TIME${NC}"
    fi
    echo ""

    # Polling section
    echo -e "  ${CYAN}🔄 Polling:${NC}"
    echo -e "    • Rate: ${GREEN}$POLLING_RATE${NC} polls/minute"
    echo ""

    # Events section
    echo -e "  ${CYAN}📝 Events (Total: ${YELLOW}$TOTAL_EVENTS${NC}${CYAN}):${NC}"

    # Identity events
    if [ "$REGISTERED" != "0" ] || [ "$METADATA_SET" != "0" ] || [ "$URI_UPDATED" != "0" ]; then
        echo -e "    ${BLUE}Identity:${NC}"
        [ "$REGISTERED" != "0" ] && echo -e "      • Registered:        $REGISTERED"
        [ "$METADATA_SET" != "0" ] && echo -e "      • MetadataSet:       $METADATA_SET"
        [ "$URI_UPDATED" != "0" ] && echo -e "      • UriUpdated:        $URI_UPDATED"
    fi

    # Reputation events
    if [ "$NEW_FEEDBACK" != "0" ] || [ "$FEEDBACK_REVOKED" != "0" ] || [ "$RESPONSE_APPENDED" != "0" ]; then
        echo -e "    ${BLUE}Reputation:${NC}"
        [ "$NEW_FEEDBACK" != "0" ] && echo -e "      • NewFeedback:       $NEW_FEEDBACK"
        [ "$FEEDBACK_REVOKED" != "0" ] && echo -e "      • FeedbackRevoked:   $FEEDBACK_REVOKED"
        [ "$RESPONSE_APPENDED" != "0" ] && echo -e "      • ResponseAppended:  $RESPONSE_APPENDED"
    fi

    # Validation events
    if [ "$VALIDATION_REQUEST" != "0" ] || [ "$VALIDATION_RESPONSE" != "0" ]; then
        echo -e "    ${BLUE}Validation:${NC}"
        [ "$VALIDATION_REQUEST" != "0" ] && echo -e "      • ValidationRequest:  $VALIDATION_REQUEST"
        [ "$VALIDATION_RESPONSE" != "0" ] && echo -e "      • ValidationResponse: $VALIDATION_RESPONSE"
    fi

    # If no events, show message
    if [ "$TOTAL_EVENTS" = "0" ]; then
        echo -e "    ${YELLOW}No events indexed yet${NC}"
    fi

    echo ""

    # Last sync
    if [ "$LAST_SYNC" != "null" ]; then
        echo -e "  ${CYAN}Last Sync:${NC} $LAST_SYNC"
    fi

    echo ""
done

echo -e "${CYAN}╚════════════════════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${GREEN}✅ Report completed${NC}"
