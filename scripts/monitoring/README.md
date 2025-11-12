# Monitoring Scripts

Production monitoring and status checking scripts.

## Scripts

### `chain-status-report.sh`

Comprehensive status report for all chains with real-time metrics.

**Usage:**
```bash
export API_URL="https://api-8004-dev.fly.dev"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/monitoring/chain-status-report.sh
```

**Features:**
- Current vs indexed block numbers
- Blocks behind with estimated catch-up time
- Chain production rate (blocks/min)
- Polling rate statistics
- Event counts by type
- Color-coded status indicators

**Requirements:** `curl`, `jq`, `bc`

### `monitor-sync.sh`

Real-time monitoring of synchronization progress.

**Usage:**
```bash
export API_URL="http://localhost:8080"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/monitoring/monitor-sync.sh
```

### `check-prod-quick.sh`

Quick production health check (minimal output).

**Usage:**
```bash
export API_URL="https://api-8004-dev.fly.dev"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/monitoring/check-prod-quick.sh
```

### `check-prod-events.sh`

Check events count in production.

**Usage:**
```bash
export API_URL="https://api-8004-dev.fly.dev"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/monitoring/check-prod-events.sh
```

## Common Environment Variables

```bash
API_URL         # API endpoint (default: http://localhost:8080)
API_USERNAME    # API username (default: admin)
API_PASSWORD    # API password (required)
```

## Tips

1. **Monitor in real-time**: Use `watch` to run scripts repeatedly:
   ```bash
   watch -n 30 './scripts/monitoring/check-prod-quick.sh'
   ```

2. **Check specific chain**: Most scripts accept chain_id parameter
3. **Export vars once**: Add to `.bashrc` or `.zshrc` for convenience
