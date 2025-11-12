# Chain Status Monitoring

This document describes the new chain status monitoring endpoint and script added to the ERC-8004 Multi-Chain Indexer.

## Overview

The monitoring system provides detailed real-time information about each blockchain being indexed, including:
- Current blockchain block number
- Indexer's current block position
- Number of blocks behind
- Polling rate (polls per minute)
- Event counts by type

## API Endpoint

### GET `/chains/status`

**Authentication**: Required (JWT token)

Returns detailed status information for all enabled chains.

#### Response Format

```json
{
  "success": true,
  "timestamp": "2025-01-12T10:30:00Z",
  "chains": [
    {
      "chain_id": 11155111,
      "name": "Ethereum Sepolia",
      "status": "syncing",
      "blocks": {
        "current": 5000000,
        "indexed": 4999950,
        "behind": 50
      },
      "polling": {
        "rate_per_minute": "5.00"
      },
      "events": {
        "total": 1250,
        "by_type": {
          "registered": 120,
          "metadata_set": 350,
          "uri_updated": 80,
          "new_feedback": 400,
          "feedback_revoked": 50,
          "response_appended": 150,
          "validation_request": 60,
          "validation_response": 40
        }
      },
      "last_sync_time": "2025-01-12T10:29:55Z"
    }
  ]
}
```

#### Fields Description

- **chain_id**: Blockchain network identifier
- **name**: Human-readable chain name
- **status**: Indexer status (`active`, `syncing`, `stalled`, `failed`)
- **blocks.current**: Latest block on the blockchain
- **blocks.indexed**: Last block processed by the indexer
- **blocks.behind**: How many blocks the indexer is behind
- **polling.rate_per_minute**: Number of RPC polls per minute
- **events.total**: Total number of events indexed
- **events.by_type**: Breakdown of events by type:
  - `registered`: New agent registrations
  - `metadata_set`: Metadata updates
  - `uri_updated`: URI updates
  - `new_feedback`: New feedback submissions
  - `feedback_revoked`: Revoked feedback
  - `response_appended`: Appended responses
  - `validation_request`: Validation requests
  - `validation_response`: Validation responses
- **last_sync_time**: Timestamp of last successful sync

## Status Script

### `chain-status-report.sh`

A bash script that queries the `/chains/status` endpoint and displays a formatted, color-coded report.

#### Prerequisites

- `curl` - HTTP client
- `jq` - JSON processor
- `bc` - Basic calculator for catch-up time calculations

Install on macOS:
```bash
brew install jq bc
```

Install on Ubuntu/Debian:
```bash
sudo apt-get install jq curl bc
```

#### Usage

```bash
# Set required environment variables
export API_URL="http://localhost:8080"  # or your production URL
export API_USERNAME="admin"
export API_PASSWORD="your-password"

# Run the script
./chain-status-report.sh
```

#### Example Output

```
=== ERC-8004 Multi-Chain Indexer Status Report ===

Authenticating...
Authenticated successfully

Fetching chains status...

╔════════════════════════════════════════════════════════════════════════╗
║                         CHAINS STATUS REPORT                           ║
╚════════════════════════════════════════════════════════════════════════╝

Report generated at: 2025-01-12T10:30:00Z

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
📊 Chain: Ethereum Sepolia (ID: 11155111)
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

  Status: syncing

  ⛓️  Blocks:
    • Current Block:     5000000
    • Indexed Block:     4999950
    • Blocks Behind:     50
    • Chain Rate:        5 blocks/min
    • Est. Catch-up:     1.0h

  🔄 Polling:
    • Rate: 10.00 polls/minute

  📝 Events (Total: 1250):
    Identity:
      • Registered:        120
      • MetadataSet:       350
      • UriUpdated:        80
    Reputation:
      • NewFeedback:       400
      • FeedbackRevoked:   50
      • ResponseAppended:  150
    Validation:
      • ValidationRequest:  60
      • ValidationResponse: 61

  Last Sync: 2025-01-12T10:29:55Z

╚════════════════════════════════════════════════════════════════════════╝

✅ Report completed
```

#### Chain Metrics Explained

**Chain Rate**: The number of blocks produced per minute by the blockchain network
- Ethereum Sepolia: 5 blocks/min (~12s block time)
- Base Sepolia: 30 blocks/min (~2s block time)
- Linea Sepolia: 30 blocks/min (~2s block time)
- Polygon Amoy: 30 blocks/min (~2s block time)
- Hedera Testnet: 30 blocks/min (~2s block time)

**Est. Catch-up**: Estimated time to catch up with the current blockchain head
- Calculated as: `blocks_behind / (polling_rate - chain_rate)`
- Shows hours (h) or minutes (m) until fully synced
- Displays "⚠️ Falling behind" if polling rate < chain rate
- Displays "✓ Synced" if blocks behind = 0

#### Color Coding

- **Status**:
  - 🟢 Green: `active`
  - 🟡 Yellow: `syncing`, `stalled`
  - 🔴 Red: `failed`

- **Blocks Behind**:
  - 🟢 Green: ≤ 10 blocks
  - 🟡 Yellow: 11-100 blocks
  - 🔴 Red: > 100 blocks

## Implementation Details

### Components Added

1. **`src/stats.rs`**: New module for tracking polling statistics
   - `StatsTracker`: Thread-safe statistics tracker using DashMap
   - Tracks polling events with sliding 1-minute window
   - Records current block numbers per chain

2. **Modified Files**:
   - `src/main.rs`: Create and pass StatsTracker to supervisors and API
   - `src/indexer/supervisor.rs`: Accept and forward StatsTracker
   - `src/indexer/mod.rs`: Record polling events and update current blocks
   - `src/api/mod.rs`: Add `/chains/status` endpoint
   - `src/storage/mod.rs`: Add `get_event_counts_by_type()` method

3. **Script**:
   - `chain-status-report.sh`: Bash script for formatted status display

### Statistics Tracking

The system tracks:
- **Polling Events**: Recorded every time the indexer queries the RPC for the latest block
- **Current Block**: Updated when successfully fetching the latest block from the blockchain
- **Event Counts**: Queried from the database, grouped by event type

### Performance Considerations

- Statistics are stored in-memory using DashMap for thread-safe concurrent access
- Polling history uses a sliding window (last 60 seconds) to calculate rate
- Event counts are queried from PostgreSQL with indexed lookups

## Testing

To test the endpoint manually with curl:

```bash
# Login to get token
TOKEN=$(curl -s -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}' \
  | jq -r '.token')

# Query status
curl -s http://localhost:8080/chains/status \
  -H "Authorization: Bearer $TOKEN" \
  | jq .
```

## Recent Enhancements

**January 2025**:
- ✅ Added chain production rate (blocks per minute) for all supported networks
- ✅ Implemented estimated catch-up time calculation
- ✅ Visual indicators for chains falling behind vs catching up

## Future Enhancements

Potential improvements:
- Add alerting for chains that fall too far behind
- Historical polling rate graphs
- Event ingestion rate tracking
- RPC provider health metrics
- Webhook notifications for status changes
- Auto-adjustment of polling rate based on chain velocity
