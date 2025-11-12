#!/bin/bash
# Quick check of production events

echo "🔍 Checking production database..."
echo ""

echo "1. Total events:"
flyctl postgres connect -a api-8004-dev-db -c "SELECT COUNT(*) FROM events;"

echo ""
echo "2. Events per chain:"
flyctl postgres connect -a api-8004-dev-db -c "
SELECT chain_id, COUNT(*) as events
FROM events
GROUP BY chain_id
ORDER BY events DESC;"

echo ""
echo "3. Applied migrations:"
flyctl postgres connect -a api-8004-dev-db -c "
SELECT version, description
FROM _sqlx_migrations
ORDER BY version;"

echo ""
echo "4. Chain sync state:"
flyctl postgres connect -a api-8004-dev-db -c "
SELECT c.name, cs.last_synced_block, cs.total_events_indexed
FROM chains c
LEFT JOIN chain_sync_state cs ON c.chain_id = cs.chain_id
WHERE c.enabled = true;"
