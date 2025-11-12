# Utility Scripts

General-purpose utility scripts for working with the ERC-8004 API.

## Scripts

### `get-all-events.sh`

Fetch all events from the API with pagination support.

**Usage:**
```bash
./scripts/utils/get-all-events.sh [username] [password] [api-url] [chain-id] [limit]
```

**Examples:**
```bash
# Get all events (default: 100)
./scripts/utils/get-all-events.sh admin mypassword

# Get events from specific chain
./scripts/utils/get-all-events.sh admin mypassword https://api-8004-dev.fly.dev 11155111

# Get specific number of events
./scripts/utils/get-all-events.sh admin mypassword https://api-8004-dev.fly.dev 11155111 500
```

**Output:** JSON array of events to stdout

**Parameters:**
- `username`: API username (default: admin)
- `password`: API password (required)
- `api-url`: API endpoint (default: https://api-8004-dev.fly.dev)
- `chain-id`: Filter by chain ID (optional)
- `limit`: Max events to fetch (default: 100)

### `get-events-safe.sh`

Fetch events using environment variables (safer than command-line args).

**Usage:**
```bash
# Create .env.test with:
# API_URL=https://api-8004-dev.fly.dev
# API_USERNAME=admin
# API_PASSWORD=your-password

source .env.test
./scripts/utils/get-events-safe.sh
```

**Features:**
- Reads credentials from environment
- No password in command history
- Supports all query parameters

### `check-event-types.sh`

Check event types distribution in the database.

**Usage:**
```bash
./scripts/utils/check-event-types.sh
```

**Output:**
- Count of each event type
- Percentage distribution
- Per-chain breakdown

**Requires:** Direct database access via `DATABASE_URL`

## Common Use Cases

### Export Events to File

```bash
# Export all Ethereum Sepolia events
./scripts/utils/get-all-events.sh admin password \
    https://api-8004-dev.fly.dev 11155111 1000 \
    > sepolia-events.json

# Pretty print
./scripts/utils/get-all-events.sh admin password | jq . > events-pretty.json
```

### Filter and Process Events

```bash
# Get only Registered events
./scripts/utils/get-all-events.sh admin password | \
    jq '.[] | select(.event_type == "Registered")'

# Count events by type
./scripts/utils/get-all-events.sh admin password | \
    jq 'group_by(.event_type) | map({type: .[0].event_type, count: length})'
```

### Compare Chains

```bash
# Get events from multiple chains
for chain_id in 11155111 84532 59141; do
    echo "Chain $chain_id:"
    ./scripts/utils/get-all-events.sh admin password \
        https://api-8004-dev.fly.dev $chain_id | jq 'length'
done
```

### Check Database Stats

```bash
# Event type distribution
./scripts/utils/check-event-types.sh

# Combine with other tools
./scripts/utils/check-event-types.sh | \
    grep "Registered" | \
    awk '{print $2}'
```

## Environment Variables

```bash
# For get-events-safe.sh
API_URL         # API endpoint
API_USERNAME    # API username
API_PASSWORD    # API password

# For check-event-types.sh
DATABASE_URL    # PostgreSQL connection string
```

## Tips

1. **Pagination**: Use limit and offset for large datasets
2. **Rate limiting**: Add delays between requests if fetching many pages
3. **Output formats**: Pipe to `jq` for JSON processing
4. **Credentials**: Use `.env.test` file with `get-events-safe.sh` for security

## Examples with jq

```bash
# Get unique agent IDs
./scripts/utils/get-all-events.sh admin pass | \
    jq -r '.[].event_data.agent_id' | sort -u

# Events in last 24h
./scripts/utils/get-all-events.sh admin pass | \
    jq --arg date "$(date -u -d '24 hours ago' +%Y-%m-%d)" \
    '.[] | select(.block_timestamp > $date)'

# Export as CSV
./scripts/utils/get-all-events.sh admin pass | \
    jq -r '["chain_id","block_number","event_type","tx_hash"] as $header |
    ($header | @csv),
    (.[] | [.chain_id, .block_number, .event_type, .transaction_hash] | @csv)'
```

## See Also

- [API Examples](../../docs/API_EXAMPLES.md)
- [Quick Start Guide](../../docs/QUICK_START.md)
