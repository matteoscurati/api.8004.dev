# Next Steps

Your ERC-8004 Indexer is now ready! Here's how to get it running.

## 1. Quick Start

```bash
# 1. Install PostgreSQL if not already installed
brew install postgresql
brew services start postgresql

# 2. Copy and configure environment
cp .env.example .env

# 3. Edit .env with your settings
# Most importantly: add your Ethereum RPC URL
nano .env

# 4. Run setup script (creates DB and builds)
./setup.sh

# 5. Start the indexer
cargo run --release
```

## 2. Get an RPC URL

You'll need an Ethereum Sepolia RPC endpoint. Get one from:

- **Alchemy** (recommended): https://www.alchemy.com/
  - Sign up for free
  - Create a new app on Sepolia network
  - Copy the HTTP URL

- **Infura**: https://infura.io/
  - Sign up for free
  - Create a new API key
  - Select Sepolia network

Then update in `.env`:
```
RPC_URL=https://eth-sepolia.g.alchemy.com/v2/YOUR_API_KEY
```

## 3. Testing the API

Once running, test the endpoints:

```bash
# Health check
curl http://localhost:8080/health

# Get recent events
curl "http://localhost:8080/events?blocks=100"

# Get last 24 hours of events
curl "http://localhost:8080/events?hours=24"

# Get only AgentRegistered events
curl "http://localhost:8080/events?event_type=AgentRegistered"

# Stats
curl http://localhost:8080/stats
```

## 4. WebSocket Connection

Test real-time events with websocat or a browser:

```bash
# Install websocat
brew install websocat

# Connect to stream
websocat ws://localhost:8080/ws
```

Or use JavaScript:
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');
ws.onmessage = (event) => console.log(JSON.parse(event.data));
```

## 5. Production Deployment

For production use:

### Option A: Docker (recommended)
Create a `Dockerfile`:
```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libpq5 ca-certificates
COPY --from=builder /app/target/release/api_8004_dev /usr/local/bin/
CMD ["api_8004_dev"]
```

Then:
```bash
docker build -t api.8004.dev .
docker run -d --env-file .env api.8004.dev
```

### Option B: systemd Service
Create `/etc/systemd/system/api.8004.dev.service`:
```ini
[Unit]
Description=ERC-8004 Event Indexer
After=network.target postgresql.service

[Service]
Type=simple
User=youruser
WorkingDirectory=/path/to/api.8004.dev
Environment="RUST_LOG=info"
EnvironmentFile=/path/to/api.8004.dev/.env
ExecStart=/path/to/api.8004.dev/target/release/api_8004_dev
Restart=always

[Install]
WantedBy=multi-user.target
```

Then:
```bash
sudo systemctl daemon-reload
sudo systemctl enable api.8004.dev
sudo systemctl start api.8004.dev
```

## 6. Monitoring

Check logs:
```bash
# Development
RUST_LOG=debug cargo run

# Production (systemd)
journalctl -u api.8004.dev -f
```

Monitor via API:
```bash
# Check sync status
watch -n 5 'curl -s http://localhost:8080/stats | jq'
```

## 7. Updating Contract Addresses

When you need to switch networks or use different contract addresses:

1. Edit `.env`:
```env
IDENTITY_REGISTRY_ADDRESS=0xYourNewAddress
REPUTATION_REGISTRY_ADDRESS=0xYourNewAddress
VALIDATION_REGISTRY_ADDRESS=0xYourNewAddress
RPC_URL=https://your-new-network-rpc
```

2. Reset database (if needed):
```bash
psql -d api_8004_dev -c "TRUNCATE events; UPDATE indexer_state SET last_synced_block=0;"
```

3. Restart the indexer

## 8. Troubleshooting

### Database connection issues
```bash
# Check PostgreSQL is running
pg_isready

# Check database exists
psql -l | grep api_8004_dev

# Manually run migrations
cargo install sqlx-cli
sqlx migrate run
```

### RPC rate limiting
If you see many RPC errors, increase poll interval in `.env`:
```env
POLL_INTERVAL_MS=30000  # 30 seconds instead of 12
```

### High memory usage
Reduce in-memory cache size in `.env`:
```env
MAX_EVENTS_IN_MEMORY=5000  # Default is 10000
```

## 9. Custom Queries

The PostgreSQL database stores all events. You can run custom SQL queries:

```sql
-- Count events by type
SELECT event_type, COUNT(*)
FROM events
GROUP BY event_type;

-- Recent agent registrations
SELECT event_data->>'agent_domain' as domain,
       event_data->>'agent_address' as address,
       block_timestamp
FROM events
WHERE event_type = 'AgentRegistered'
ORDER BY block_timestamp DESC
LIMIT 10;

-- Events in last hour
SELECT COUNT(*)
FROM events
WHERE block_timestamp > NOW() - INTERVAL '1 hour';
```

## 10. Further Development

Ideas for extending the indexer:

- Add Prometheus metrics endpoint
- Implement GraphQL API
- Add event notifications (Discord, Telegram, email)
- Create web dashboard
- Add support for multiple networks simultaneously
- Implement event replay/backfill tools
- Add API authentication/rate limiting

Happy indexing! 🚀
