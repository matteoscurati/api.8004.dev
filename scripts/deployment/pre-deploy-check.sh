#!/bin/bash
#
# Pre-Deploy Check Script
# Verifica lo stato attuale in produzione prima del deploy
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

echo -e "${BLUE}🔍 Pre-Deploy Check - ERC-8004 Indexer${NC}"
echo "=================================================="
echo ""

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    echo -e "${RED}❌ flyctl not found${NC}"
    echo "   Install it: curl -L https://fly.io/install.sh | sh"
    exit 1
fi

# Check if logged in
if ! flyctl auth whoami &> /dev/null; then
    echo -e "${RED}❌ Not logged in to Fly.io${NC}"
    echo "   Run: flyctl auth login"
    exit 1
fi

echo -e "${GREEN}✅ flyctl installed and authenticated${NC}"
echo ""

# Step 1: Check app status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "1️⃣  App Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if flyctl status -a $APP_NAME &> /dev/null; then
    echo -e "${GREEN}✅ App exists and is running${NC}"
    flyctl status -a $APP_NAME | grep -E "(ID|Status|Image|Hostname)" || true
else
    echo -e "${YELLOW}⚠️  App does not exist or is not running${NC}"
fi

echo ""

# Step 2: Check database status
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "2️⃣  Database Status"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if flyctl postgres list 2>/dev/null | grep -q $DB_NAME; then
    echo -e "${GREEN}✅ Database exists: $DB_NAME${NC}"
    flyctl postgres list | grep -E "(NAME|$DB_NAME)" || true
else
    echo -e "${YELLOW}⚠️  Database not found${NC}"
    echo ""
    echo "This is your first deploy. Database will be created."
    exit 0
fi

echo ""

# Step 3: Count events in database
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "3️⃣  Events Currently in Database"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if events table exists
echo "Checking if events table exists..."
TABLE_EXISTS=$(flyctl postgres connect -a $DB_NAME -c "
SELECT COUNT(*)
FROM information_schema.tables
WHERE table_name = 'events';
" 2>/dev/null | tail -n 1 | tr -d ' ' || echo "0")

if [ "$TABLE_EXISTS" = "1" ]; then
    echo -e "${GREEN}✅ Events table exists${NC}"
    echo ""

    # Total events
    echo "📊 Total Events:"
    TOTAL_EVENTS=$(flyctl postgres connect -a $DB_NAME -c "SELECT COUNT(*) as total FROM events;" 2>/dev/null | grep -v "total" | grep -v "row" | tail -n 1 | tr -d ' ' || echo "0")
    echo "   Total: $TOTAL_EVENTS events"
    echo ""

    # Events per chain
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

    # Events per type
    echo "📊 Events per Type:"
    flyctl postgres connect -a $DB_NAME -c "
    SELECT
        event_type,
        COUNT(*) as count
    FROM events
    GROUP BY event_type
    ORDER BY count DESC;
    " 2>/dev/null | grep -v "Connecting" | grep -v "row" || true

else
    echo -e "${YELLOW}⚠️  Events table does not exist yet${NC}"
    echo "   This is expected for a fresh database."
    TOTAL_EVENTS=0
fi

echo ""

# Step 4: Check migrations
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "4️⃣  Database Migrations"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

MIGRATIONS_TABLE=$(flyctl postgres connect -a $DB_NAME -c "
SELECT COUNT(*)
FROM information_schema.tables
WHERE table_name = '_sqlx_migrations';
" 2>/dev/null | tail -n 1 | tr -d ' ' || echo "0")

if [ "$MIGRATIONS_TABLE" = "1" ]; then
    echo "Applied Migrations:"
    flyctl postgres connect -a $DB_NAME -c "
    SELECT
        version,
        description,
        installed_on
    FROM _sqlx_migrations
    ORDER BY version;
    " 2>/dev/null | grep -v "Connecting" | grep -v "row" || true

    MIGRATION_COUNT=$(flyctl postgres connect -a $DB_NAME -c "SELECT COUNT(*) FROM _sqlx_migrations;" 2>/dev/null | tail -n 1 | tr -d ' ' || echo "0")
    echo ""
    echo "   Applied: $MIGRATION_COUNT migrations"

    # Check for pending migrations
    LOCAL_MIGRATIONS=$(ls -1 migrations/*.sql 2>/dev/null | wc -l | tr -d ' ')
    echo "   Local: $LOCAL_MIGRATIONS migrations"

    if [ "$MIGRATION_COUNT" -lt "$LOCAL_MIGRATIONS" ]; then
        PENDING=$((LOCAL_MIGRATIONS - MIGRATION_COUNT))
        echo -e "   ${YELLOW}⚠️  $PENDING new migrations will be applied on deploy${NC}"
    else
        echo -e "   ${GREEN}✅ All migrations up to date${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  No migrations table found${NC}"
    echo "   All migrations will be applied on first deploy"
fi

echo ""

# Step 5: Check chain sync state
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "5️⃣  Chain Sync State"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

CHAINS_TABLE=$(flyctl postgres connect -a $DB_NAME -c "
SELECT COUNT(*)
FROM information_schema.tables
WHERE table_name = 'chains';
" 2>/dev/null | tail -n 1 | tr -d ' ' || echo "0")

if [ "$CHAINS_TABLE" = "1" ]; then
    echo "Chain Status:"
    flyctl postgres connect -a $DB_NAME -c "
    SELECT
        c.name,
        c.chain_id,
        COALESCE(cs.last_synced_block, 0) as last_block,
        COALESCE(cs.total_events_indexed, 0) as events,
        COALESCE(cs.status, 'unknown') as status
    FROM chains c
    LEFT JOIN chain_sync_state cs ON c.chain_id = cs.chain_id
    WHERE c.enabled = true
    ORDER BY events DESC;
    " 2>/dev/null | grep -v "Connecting" | grep -v "row" || true
else
    echo -e "${YELLOW}⚠️  Chains table does not exist yet${NC}"
    echo "   Multi-chain tables will be created on deploy"
fi

echo ""

# Step 6: Backup recommendation
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "6️⃣  Backup Recommendation"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

if [ "$TOTAL_EVENTS" -gt 0 ]; then
    echo -e "${YELLOW}⚠️  You have $TOTAL_EVENTS events in production${NC}"
    echo ""
    echo "Recommended: Create a backup before deploy"
    echo ""
    read -p "Do you want to create a backup now? (y/n) " -n 1 -r
    echo ""

    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo ""
        echo "Creating backup..."

        # Create backup directory
        BACKUP_DIR="backups/$(date +%Y%m%d_%H%M%S)"
        mkdir -p "$BACKUP_DIR"

        # Backup events
        echo "📦 Backing up events..."
        flyctl postgres connect -a $DB_NAME -c "
        COPY (SELECT * FROM events ORDER BY id) TO STDOUT WITH CSV HEADER
        " > "$BACKUP_DIR/events.csv" 2>/dev/null

        # Backup chains (if exists)
        if [ "$CHAINS_TABLE" = "1" ]; then
            echo "📦 Backing up chains..."
            flyctl postgres connect -a $DB_NAME -c "
            COPY (SELECT * FROM chains) TO STDOUT WITH CSV HEADER
            " > "$BACKUP_DIR/chains.csv" 2>/dev/null

            echo "📦 Backing up chain_sync_state..."
            flyctl postgres connect -a $DB_NAME -c "
            COPY (SELECT * FROM chain_sync_state) TO STDOUT WITH CSV HEADER
            " > "$BACKUP_DIR/chain_sync_state.csv" 2>/dev/null
        fi

        # Create metadata
        cat > "$BACKUP_DIR/backup_info.txt" << EOF
Backup created: $(date)
App: $APP_NAME
Database: $DB_NAME
Total events: $TOTAL_EVENTS
Migrations applied: $MIGRATION_COUNT

Files:
- events.csv ($TOTAL_EVENTS rows)
$([ "$CHAINS_TABLE" = "1" ] && echo "- chains.csv" || echo "")
$([ "$CHAINS_TABLE" = "1" ] && echo "- chain_sync_state.csv" || echo "")
EOF

        echo ""
        echo -e "${GREEN}✅ Backup created: $BACKUP_DIR${NC}"
        echo ""
        ls -lh "$BACKUP_DIR/"
        echo ""
    fi
fi

# Summary
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Pre-Deploy Check Complete${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Create summary
echo "📊 Summary:"
echo "   • App: $APP_NAME"
echo "   • Database: $DB_NAME"
echo "   • Events in DB: $TOTAL_EVENTS"
echo "   • Migrations: $MIGRATION_COUNT applied"
if [ "$MIGRATION_COUNT" -lt "$LOCAL_MIGRATIONS" ]; then
    echo "   • Pending migrations: $((LOCAL_MIGRATIONS - MIGRATION_COUNT))"
fi
echo ""

if [ "$TOTAL_EVENTS" -gt 0 ]; then
    echo -e "${GREEN}✅ Your data is safe!${NC}"
    echo "   Deploy will:"
    echo "   • Update application code only"
    echo "   • Apply new migrations (non-destructive)"
    echo "   • Keep all $TOTAL_EVENTS existing events"
    echo "   • Resume indexing from last synced block"
else
    echo -e "${BLUE}ℹ️  This appears to be a fresh deployment${NC}"
    echo "   Deploy will:"
    echo "   • Create database schema"
    echo "   • Start indexing from configured starting block"
fi

echo ""
echo "🚀 Ready to deploy? Run:"
echo "   flyctl deploy"
echo ""
