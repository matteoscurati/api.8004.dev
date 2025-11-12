#!/bin/bash
#
# Post-Deploy Check Script
# Verifica che il deploy sia andato a buon fine
#

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

APP_NAME="api-8004-dev"
DB_NAME="api-8004-dev-db"
API_URL="https://api-8004-dev.fly.dev"

echo -e "${BLUE}✅ Post-Deploy Check - ERC-8004 Indexer${NC}"
echo "=================================================="
echo ""

# Step 1: Check app is running
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  App Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if flyctl status -a $APP_NAME | grep -q "running"; then
    echo -e "${GREEN}✅ App is running${NC}"
    flyctl status -a $APP_NAME | grep -E "(ID|Status|Health)" || true
else
    echo -e "${RED}❌ App is not running${NC}"
    echo ""
    echo "Check logs:"
    echo "   flyctl logs -a $APP_NAME"
    exit 1
fi

echo ""

# Step 2: Health check
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  API Health Check"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Testing: $API_URL/health"
HEALTH_RESPONSE=$(curl -s $API_URL/health || echo "")

if [[ $HEALTH_RESPONSE == *"ok"* ]]; then
    echo -e "${GREEN}✅ API is healthy${NC}"
    echo "   Response: $HEALTH_RESPONSE"
else
    echo -e "${RED}❌ API health check failed${NC}"
    echo "   Response: $HEALTH_RESPONSE"
fi

echo ""

# Step 3: Check migrations
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  Database Migrations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Applied Migrations:"
flyctl postgres connect -a $DB_NAME -c "
SELECT
    version,
    description,
    installed_on
FROM _sqlx_migrations
ORDER BY version;
" 2>/dev/null | grep -v "Connecting" | grep -v "row" || true

MIGRATION_COUNT=$(flyctl postgres connect -a $DB_NAME -c "SELECT COUNT(*) FROM _sqlx_migrations;" 2>/dev/null | tail -n 1 | tr -d ' ')
LOCAL_MIGRATIONS=$(ls -1 migrations/*.sql 2>/dev/null | wc -l | tr -d ' ')

echo ""
if [ "$MIGRATION_COUNT" = "$LOCAL_MIGRATIONS" ]; then
    echo -e "${GREEN}✅ All $MIGRATION_COUNT migrations applied${NC}"
else
    echo -e "${YELLOW}⚠️  Migrations mismatch: $MIGRATION_COUNT applied, $LOCAL_MIGRATIONS expected${NC}"
fi

echo ""

# Step 4: Count events (verify no data loss)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  Event Count (Verify No Data Loss)"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

TOTAL_EVENTS=$(flyctl postgres connect -a $DB_NAME -c "SELECT COUNT(*) FROM events;" 2>/dev/null | tail -n 1 | tr -d ' ')
echo "📊 Total Events: $TOTAL_EVENTS"
echo ""

echo "📊 Events per Chain:"
flyctl postgres connect -a $DB_NAME -c "
SELECT
    chain_id,
    COUNT(*) as events
FROM events
GROUP BY chain_id
ORDER BY events DESC;
" 2>/dev/null | grep -v "Connecting" | grep -v "row" || true

echo ""

# Step 5: Check chains table (new multi-chain feature)
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  Multi-Chain Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Enabled Chains:"
flyctl postgres connect -a $DB_NAME -c "
SELECT
    c.name,
    c.chain_id,
    COALESCE(cs.last_synced_block, 0) as last_block,
    COALESCE(cs.total_events_indexed, 0) as events,
    COALESCE(cs.status, 'unknown') as status,
    cs.last_sync_time
FROM chains c
LEFT JOIN chain_sync_state cs ON c.chain_id = cs.chain_id
WHERE c.enabled = true
ORDER BY c.chain_id;
" 2>/dev/null | grep -v "Connecting" | grep -v "row" || true

echo ""

# Step 6: Test authentication
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  Authentication Test"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if .env.test exists for credentials
if [ -f ".env.test" ]; then
    source .env.test

    echo "Testing login endpoint..."
    LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
        -H "Content-Type: application/json" \
        -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}" || echo "")

    if [[ $LOGIN_RESPONSE == *"token"* ]]; then
        echo -e "${GREEN}✅ Authentication working${NC}"
        TOKEN=$(echo $LOGIN_RESPONSE | grep -o '"token":"[^"]*' | cut -d'"' -f4)
        echo "   Token received (first 50 chars): ${TOKEN:0:50}..."

        # Test protected endpoint
        echo ""
        echo "Testing protected endpoint..."
        EVENTS_RESPONSE=$(curl -s "$API_URL/events?chain_id=11155111&limit=1" \
            -H "Authorization: Bearer $TOKEN" || echo "")

        if [[ $EVENTS_RESPONSE == *"events"* ]] || [[ $EVENTS_RESPONSE == *"success"* ]]; then
            echo -e "${GREEN}✅ Protected endpoints working${NC}"
        else
            echo -e "${YELLOW}⚠️  Protected endpoint returned: ${EVENTS_RESPONSE:0:100}${NC}"
        fi
    else
        echo -e "${RED}❌ Authentication failed${NC}"
        echo "   Response: $LOGIN_RESPONSE"
    fi
else
    echo -e "${YELLOW}⚠️  .env.test not found, skipping auth test${NC}"
    echo "   Create .env.test with API_USERNAME and API_PASSWORD to test"
fi

echo ""

# Step 7: Check recent logs for errors
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "7️⃣  Recent Logs"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "Last 20 log lines:"
flyctl logs -a $APP_NAME --count 20 2>/dev/null | tail -20 || true

echo ""

ERROR_COUNT=$(flyctl logs -a $APP_NAME --count 100 2>/dev/null | grep -c "ERROR" || echo "0")
if [ "$ERROR_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  Found $ERROR_COUNT ERROR logs in last 100 lines${NC}"
    echo ""
    echo "Recent errors:"
    flyctl logs -a $APP_NAME --count 100 2>/dev/null | grep "ERROR" | tail -5
else
    echo -e "${GREEN}✅ No errors in recent logs${NC}"
fi

echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Post-Deploy Check Complete${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

echo "📊 Deployment Summary:"
echo "   • App Status: $(flyctl status -a $APP_NAME 2>/dev/null | grep "Status" | awk '{print $2}' || echo "Unknown")"
echo "   • Health Check: $(echo $HEALTH_RESPONSE | grep -o '"status":"[^"]*' | cut -d'"' -f4 || echo "Unknown")"
echo "   • Migrations: $MIGRATION_COUNT/$LOCAL_MIGRATIONS applied"
echo "   • Total Events: $TOTAL_EVENTS"
echo "   • Errors: $ERROR_COUNT in last 100 logs"
echo ""

echo "🔗 Useful Links:"
echo "   • App: $API_URL"
echo "   • Health: $API_URL/health"
echo "   • Chains: $API_URL/chains"
echo "   • Detailed Health: $API_URL/health/detailed"
echo "   • Metrics: $API_URL/metrics"
echo ""

echo "📝 Next Steps:"
echo "   • Monitor logs: flyctl logs -a $APP_NAME"
echo "   • View metrics: $API_URL/metrics"
echo "   • Test API: ./test-api.sh"
echo ""

if [ "$ERROR_COUNT" -gt 5 ]; then
    echo -e "${YELLOW}⚠️  Warning: Multiple errors detected${NC}"
    echo "   Review logs: flyctl logs -a $APP_NAME | grep ERROR"
    echo ""
fi
