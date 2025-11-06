#!/bin/bash

# API Testing Script for api.8004.dev
# Usage: ./test-api.sh [command]

set -e

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

API_URL="https://api-8004-dev.fly.dev"
JWT_SECRET="5tkqFytIWZfiLV33IcHkSz0B7T5Z2kCHzwFVQV9RMDq5VXLae7vzbB9ulRZfK+7/"

# Function to display usage
usage() {
    echo "Usage: $0 [command]"
    echo ""
    echo "Commands:"
    echo "  health       - Check API health"
    echo "  login        - Get JWT token"
    echo "  status       - Check indexer status"
    echo "  events       - Get all events"
    echo "  identity     - Get identity events"
    echo "  reputation   - Get reputation events"
    echo "  validation   - Get validation events"
    echo "  metrics      - Get Prometheus metrics"
    echo ""
    exit 1
}

# Get JWT token
get_token() {
    echo -e "${YELLOW}Enter username (default: admin):${NC}"
    read -r USERNAME
    USERNAME=${USERNAME:-admin}

    echo -e "${YELLOW}Enter password:${NC}"
    read -rs PASSWORD
    echo ""

    echo -e "${BLUE}Logging in...${NC}"
    RESPONSE=$(curl -s -X POST "$API_URL/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

    TOKEN=$(echo "$RESPONSE" | jq -r '.token // empty')

    if [ -z "$TOKEN" ]; then
        echo -e "${YELLOW}❌ Login failed${NC}"
        echo "$RESPONSE" | jq .
        exit 1
    fi

    echo -e "${GREEN}✅ Login successful!${NC}"
    echo ""
    echo -e "${BLUE}Your JWT token:${NC}"
    echo "$TOKEN"
    echo ""
    echo -e "${BLUE}Export it:${NC}"
    echo "export JWT_TOKEN=\"$TOKEN\""
}

# Check health
check_health() {
    echo -e "${BLUE}Checking API health...${NC}"
    curl -s "$API_URL/health" | jq .
}

# Get indexer status
get_status() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}⚠️  JWT_TOKEN not set. Run: $0 login${NC}"
        exit 1
    fi

    echo -e "${BLUE}Getting indexer status...${NC}"
    curl -s "$API_URL/stats" \
        -H "Authorization: Bearer $JWT_TOKEN" | jq .
}

# Get all events
get_events() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}⚠️  JWT_TOKEN not set. Run: $0 login${NC}"
        exit 1
    fi

    LIMIT=${1:-10}
    OFFSET=${2:-0}

    echo -e "${BLUE}Getting events (limit=$LIMIT, offset=$OFFSET)...${NC}"
    curl -s "$API_URL/events?limit=$LIMIT&offset=$OFFSET" \
        -H "Authorization: Bearer $JWT_TOKEN" | jq .
}

# Get identity events
get_identity_events() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}⚠️  JWT_TOKEN not set. Run: $0 login${NC}"
        exit 1
    fi

    echo -e "${BLUE}Getting Identity Registry events...${NC}"
    curl -s "$API_URL/events?contract=0x8004a6090Cd10A7288092483047B097295Fb8847" \
        -H "Authorization: Bearer $JWT_TOKEN" | jq .
}

# Get reputation events
get_reputation_events() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}⚠️  JWT_TOKEN not set. Run: $0 login${NC}"
        exit 1
    fi

    echo -e "${BLUE}Getting Reputation Registry events...${NC}"
    curl -s "$API_URL/events?contract=0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E" \
        -H "Authorization: Bearer $JWT_TOKEN" | jq .
}

# Get validation events
get_validation_events() {
    if [ -z "$JWT_TOKEN" ]; then
        echo -e "${YELLOW}⚠️  JWT_TOKEN not set. Run: $0 login${NC}"
        exit 1
    fi

    echo -e "${BLUE}Getting Validation Registry events...${NC}"
    curl -s "$API_URL/events?contract=0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5" \
        -H "Authorization: Bearer $JWT_TOKEN" | jq .
}

# Get metrics
get_metrics() {
    echo -e "${BLUE}Getting Prometheus metrics...${NC}"
    curl -s "$API_URL/metrics"
}

# Main script
case "${1:-}" in
    health)
        check_health
        ;;
    login)
        get_token
        ;;
    status)
        get_status
        ;;
    events)
        get_events "${2:-10}" "${3:-0}"
        ;;
    identity)
        get_identity_events
        ;;
    reputation)
        get_reputation_events
        ;;
    validation)
        get_validation_events
        ;;
    metrics)
        get_metrics
        ;;
    *)
        usage
        ;;
esac
