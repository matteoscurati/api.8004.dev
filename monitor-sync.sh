#!/bin/bash

# Monitor indexer sync progress
# Usage: ./monitor-sync.sh [interval_seconds]

INTERVAL="${1:-10}"
TARGET_BLOCK=9575697  # Approximate current block (update if needed)

echo "🔍 Monitoring Indexer Sync Progress"
echo "Target block: ~$TARGET_BLOCK"
echo "Checking every $INTERVAL seconds..."
echo ""

PREV_BLOCK=0
PREV_TIME=$(date +%s)

while true; do
    # Get current synced block
    RESPONSE=$(curl -s https://api-8004-dev.fly.dev/health/detailed)
    CURRENT_BLOCK=$(echo "$RESPONSE" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d['checks']['database']['last_synced_block'])" 2>/dev/null)

    if [ -z "$CURRENT_BLOCK" ]; then
        echo "❌ Failed to get block number"
        sleep $INTERVAL
        continue
    fi

    CURRENT_TIME=$(date +%s)
    TIME_DIFF=$((CURRENT_TIME - PREV_TIME))

    # Calculate progress
    REMAINING=$((TARGET_BLOCK - CURRENT_BLOCK))
    PERCENT=$(echo "scale=2; ($CURRENT_BLOCK * 100) / $TARGET_BLOCK" | bc)

    # Calculate speed
    if [ $PREV_BLOCK -ne 0 ]; then
        BLOCKS_SYNCED=$((CURRENT_BLOCK - PREV_BLOCK))
        BLOCKS_PER_SEC=$(echo "scale=2; $BLOCKS_SYNCED / $TIME_DIFF" | bc)

        # Estimate remaining time
        if [ "$BLOCKS_PER_SEC" != "0" ] && [ "$BLOCKS_PER_SEC" != "0.00" ]; then
            SECONDS_LEFT=$(echo "scale=0; $REMAINING / $BLOCKS_PER_SEC" | bc)
            HOURS_LEFT=$(echo "scale=1; $SECONDS_LEFT / 3600" | bc)

            echo "📊 Block: $(printf "%'d" $CURRENT_BLOCK) | Progress: ${PERCENT}% | Speed: ${BLOCKS_PER_SEC} bl/s | Remaining: ~${HOURS_LEFT}h (~$(printf "%'d" $REMAINING) blocks)"
        else
            echo "📊 Block: $(printf "%'d" $CURRENT_BLOCK) | Progress: ${PERCENT}% | Remaining: ~$(printf "%'d" $REMAINING) blocks"
        fi
    else
        echo "📊 Block: $(printf "%'d" $CURRENT_BLOCK) | Progress: ${PERCENT}% | Remaining: ~$(printf "%'d" $REMAINING) blocks"
    fi

    PREV_BLOCK=$CURRENT_BLOCK
    PREV_TIME=$CURRENT_TIME

    # Check if synced
    if [ $REMAINING -le 10 ]; then
        echo ""
        echo "✅ SYNC COMPLETE! Only $REMAINING blocks behind."
        echo ""
        echo "💡 You can now slow down the polling:"
        echo "   flyctl secrets set POLL_INTERVAL_MS=\"12000\" --app api-8004-dev"
        break
    fi

    sleep $INTERVAL
done
