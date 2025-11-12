# Quick Start Guide - Multi-Chain ERC-8004 Indexer

## Configuration Complete!

Your multi-chain indexer is now configured with RPC endpoints for 5 active testnets and 3 mainnet chains ready for production.

---

## What's Been Configured

### Active Testnet Chains (5)
1. **Ethereum Sepolia** (Chain ID: 11155111) - Ankr RPC
2. **Base Sepolia** (Chain ID: 84532) - QuickNode RPC
3. **Linea Sepolia** (Chain ID: 59141) - Infura RPC
4. **Polygon Amoy** (Chain ID: 80002) - Public RPC
5. **Hedera Testnet** (Chain ID: 296) - Hashio RPC

### Ready for Production (Disabled)
- Ethereum Mainnet (Chain ID: 1)
- Base Mainnet (Chain ID: 8453)
- Linea Mainnet (Chain ID: 59144)

---

## Files Created/Updated

```
✅ chains.yaml           - Multi-chain configuration with RPC endpoints
✅ test-config.sh        - Configuration validation script
✅ RPC_ENDPOINTS.md      - Complete RPC endpoint documentation
✅ QUICK_START.md        - This file
```

---

## Testing the Configuration

### 1. Validate Configuration
```bash
./test-config.sh
```

This will verify that `chains.yaml` is valid and display all configured chains.

### 2. Build the Project
```bash
cargo build --release
```

### 3. Set Up Environment Variables

Make sure your `.env` file has the required variables:

```bash
# Required
DATABASE_URL=postgresql://user:password@localhost:5432/api_8004_dev
JWT_SECRET=your-32-character-or-longer-secret-key-here
AUTH_USERNAME=admin
AUTH_PASSWORD=your-secure-password

# Optional
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
MAX_EVENTS_IN_MEMORY=10000
```

### 4. Run Database Migrations
```bash
# Make sure PostgreSQL is running
cargo install sqlx-cli  # If not already installed
sqlx migrate run
```

### 5. Start the Indexer
```bash
cargo run
```

---

## Quick Testing Guide

### Start with One Chain (Recommended)

For initial testing, enable only Ethereum Sepolia:

1. Edit `chains.yaml`:
   ```yaml
   - name: "Ethereum Sepolia"
     enabled: true  # Keep this
     # ...

   - name: "Base Sepolia"
     enabled: false  # Change to false temporarily
     # ...
   ```

2. Run the indexer:
   ```bash
   cargo run
   ```

3. Verify it's working:
   ```bash
   # Check health
   curl http://localhost:8080/health/detailed

   # Check chains status
   curl http://localhost:8080/chains
   ```

### Enable All Chains

Once single-chain testing is successful:

1. Edit `chains.yaml` and set `enabled: true` for all desired chains
2. Restart the service
3. Monitor logs for any issues

---

## Monitoring Your Indexer

### Health Check
```bash
curl http://localhost:8080/health/detailed | jq
```

### Chain Status
```bash
curl http://localhost:8080/chains | jq
```

### Query Events (with authentication)
```bash
# Login first
TOKEN=$(curl -X POST http://localhost:8080/api/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}' | jq -r '.token')

# Get events for Ethereum Sepolia
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/api/events?chain_id=11155111&limit=10" | jq
```

---

## Common Operations

### Switch RPC Endpoint

If an RPC endpoint fails, switch to an alternative:

1. Open `chains.yaml`
2. Find the chain with issues
3. Copy one of the alternative RPC URLs from the comments
4. Replace the `rpc_url` value
5. Restart the service

Example:
```yaml
# Before
rpc_url: "https://rpc.ankr.com/eth_sepolia/..."

# After (switching to Infura)
rpc_url: "https://sepolia.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb"
```

### Disable a Chain

Set `enabled: false` in `chains.yaml` and restart.

### Add a New Chain

1. Add a new chain entry in `chains.yaml`:
   ```yaml
   - name: "New Chain"
     chain_id: 12345
     enabled: true
     rpc_url: "https://rpc.example.com"
     contracts:
       identity_registry: "0x..."
       reputation_registry: "0x..."
       validation_registry: "0x..."
     starting_block: "latest"
     poll_interval_ms: 2000
     batch_size: 5
     adaptive_polling: true
   ```

2. Add the chain to the database:
   ```sql
   INSERT INTO chains (chain_id, name, enabled) VALUES (12345, 'New Chain', true);
   ```

3. Restart the service

---

## Deployment to Fly.io

### Prerequisites
```bash
# Install flyctl if not already installed
brew install flyctl

# Login to Fly.io
flyctl auth login
```

### Deploy
```bash
# Deploy the application
flyctl deploy

# Check status
flyctl status

# View logs
flyctl logs

# Check health
curl https://api-8004-dev.fly.dev/health/detailed
```

### Set Secrets
```bash
flyctl secrets set \
  JWT_SECRET="your-secret-key" \
  AUTH_USERNAME="admin" \
  AUTH_PASSWORD="your-password"
```

---

## Troubleshooting

### Issue: "No enabled chains found"
**Solution:** Check that at least one chain has `enabled: true` in `chains.yaml`

### Issue: RPC connection errors
**Solution:**
1. Test the RPC endpoint manually (see RPC_ENDPOINTS.md)
2. Switch to an alternative RPC URL
3. Check provider status pages

### Issue: Database connection failed
**Solution:**
1. Verify PostgreSQL is running
2. Check DATABASE_URL in `.env`
3. Run migrations: `sqlx migrate run`

### Issue: High memory usage
**Solution:**
1. Reduce `MAX_EVENTS_IN_MEMORY` in `.env`
2. Disable some chains
3. Increase poll intervals in `chains.yaml`

### Issue: Chain shows "stalled" status
**Solution:**
1. Check RPC endpoint is working
2. Verify contract addresses are correct
3. Check blockchain explorer for recent blocks
4. Increase retry limits in `chains.yaml`

---

## Performance Tips

### Optimize Poll Intervals

Adjust based on chain characteristics:

- **Fast chains** (Base, Linea): 2000ms or lower
- **Medium chains** (Ethereum): 12000ms
- **Custom chains**: Test to find optimal value

### Batch Processing

For chains with many events:
- Increase `batch_size` (e.g., 10 for Base)
- Enable `adaptive_polling: true`
- Monitor memory usage

### Database Optimization

```sql
-- Add indexes if needed
CREATE INDEX IF NOT EXISTS idx_events_timestamp ON events(timestamp);
CREATE INDEX IF NOT EXISTS idx_events_category ON events(category);
```

---

## Next Steps

1. ✅ Configuration complete
2. ⏭️ Test with single chain (Ethereum Sepolia)
3. ⏭️ Enable all testnet chains
4. ⏭️ Deploy to production (Fly.io)
5. ⏭️ Monitor and optimize
6. ⏭️ Enable mainnet chains when contracts are deployed

---

## Documentation

- `chains.yaml` - Main configuration file
- `RPC_ENDPOINTS.md` - Complete RPC endpoint reference
- `MULTICHAIN_IMPLEMENTATION.md` - Technical implementation details
- `TESTING_GUIDE.md` - Comprehensive testing guide
- `DEPLOYMENT.md` - Deployment instructions

---

## Support

For issues or questions:
1. Check the troubleshooting section above
2. Review logs: `flyctl logs` or local console output
3. Check `/health/detailed` endpoint
4. Review provider status pages (see RPC_ENDPOINTS.md)

---

**Ready to start!** Run `./test-config.sh` to verify your configuration, then `cargo run` to launch the indexer.

---

Last Updated: 2025-01-07
