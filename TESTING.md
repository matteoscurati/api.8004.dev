# Testing Guide - API 8004.dev

This guide explains how to test all aspects of the ERC-8004 Indexer API.

---

## 📋 Table of Contents

1. [Unit Tests](#unit-tests)
2. [API Endpoint Tests](#api-endpoint-tests)
3. [WebSocket Tests](#websocket-tests)
4. [Test Scripts Reference](#test-scripts-reference)

---

## 🧪 Unit Tests

Run all Rust unit tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_jwt_token_creation

# Run in single thread (avoid env var conflicts)
cargo test -- --test-threads=1
```

**Coverage:** 11 tests covering JWT, authentication, and configuration.

---

## 🌐 API Endpoint Tests

### Quick Test - All Endpoints

Run the comprehensive test script:

```bash
./test-endpoints.sh
```

This tests:
- ✅ Health endpoints
- ✅ Login/authentication
- ✅ Protected endpoints (stats, events)
- ✅ Metrics endpoint

### Manual Testing

#### 1. Test Health Endpoint

```bash
curl https://api-8004-dev.fly.dev/health
```

#### 2. Test Login & Get Token

```bash
curl -X POST https://api-8004-dev.fly.dev/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

Save the token:
```bash
export TOKEN="eyJ0eXAiOiJKV1QiLCJhbGc..."
```

#### 3. Test Protected Endpoints

```bash
# Get indexer stats
curl -H "Authorization: Bearer $TOKEN" \
  https://api-8004-dev.fly.dev/stats

# Get events
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?limit=10"

# Filter by contract
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?contract=0x8004a6090Cd10A7288092483047B097295Fb8847"

# Filter by event type
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?event_type=Registered"

# Filter by time range
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?hours=24&limit=1000"
```

---

## 🔌 WebSocket Tests

### Option 1: Browser Interface (Recommended)

Open the interactive test page:

```bash
open test-websocket.html
```

**Features:**
- Visual connection status
- Real-time event display
- Connection statistics
- Easy login form

### Option 2: Node.js Script

```bash
# Install dependencies first
npm install ws node-fetch

# Run the test
node test-websocket.js admin your-password https://api-8004-dev.fly.dev
```

### Option 3: Python Script

```bash
# Install dependencies first
pip install websockets requests

# Run the test
python3 test-websocket.py admin your-password https://api-8004-dev.fly.dev
```

### Option 4: wscat (CLI Tool)

```bash
# Install wscat
npm install -g wscat

# Get token first
TOKEN=$(curl -s -X POST https://api-8004-dev.fly.dev/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}' | \
  python3 -c "import sys, json; print(json.load(sys.stdin)['token'])")

# Connect to WebSocket
wscat -c "wss://api-8004-dev.fly.dev/ws?token=$TOKEN"
```

---

## 📚 Test Scripts Reference

### `test-endpoints.sh`

Comprehensive endpoint testing script.

**Usage:**
```bash
./test-endpoints.sh
```

**Tests:**
- Health endpoints (public)
- Login endpoint
- Stats endpoint (authenticated)
- Events endpoint (authenticated)
- Metrics endpoint (public)

---

### `get-all-events.sh`

Download events from the API with flexible parameters.

**Usage:**
```bash
./get-all-events.sh [username] [password] [api-url] [limit]
```

**Examples:**
```bash
# Default: admin/admin123, limit 10000
./get-all-events.sh

# Custom credentials and limit
./get-all-events.sh admin mypassword https://api-8004-dev.fly.dev 5000

# Production with limit
./get-all-events.sh admin mypass https://api-8004-dev.fly.dev 50000
```

**Output:** Saves to `events_TIMESTAMP.json`

---

### `test-websocket.html`

Interactive browser-based WebSocket tester.

**Usage:**
```bash
open test-websocket.html
```

**Features:**
- Login form
- Connection status indicator
- Real-time event display
- Statistics (events/sec, uptime, count)
- Event details with syntax highlighting

---

### `test-websocket.js`

Node.js WebSocket test client.

**Usage:**
```bash
node test-websocket.js [username] [password] [api-url]
```

**Examples:**
```bash
# Test with defaults
node test-websocket.js

# Test production
node test-websocket.js admin mypass https://api-8004-dev.fly.dev

# Test localhost
node test-websocket.js admin admin123 http://localhost:8080
```

**Output:** Console with formatted event details

---

### `test-websocket.py`

Python WebSocket test client.

**Usage:**
```bash
python3 test-websocket.py [username] [password] [api-url]
```

**Examples:**
```bash
# Test with defaults
python3 test-websocket.py

# Test production
python3 test-websocket.py admin mypass https://api-8004-dev.fly.dev
```

**Output:** Console with formatted event details and statistics

---

### `test-api.sh`

Legacy API test script with multiple commands.

**Usage:**
```bash
./test-api.sh [command]
```

**Commands:**
- `health` - Check API health
- `login` - Get JWT token
- `status` - Check indexer status (requires JWT_TOKEN)
- `events` - Get all events (requires JWT_TOKEN)
- `identity` - Get identity events (requires JWT_TOKEN)
- `reputation` - Get reputation events (requires JWT_TOKEN)
- `validation` - Get validation events (requires JWT_TOKEN)
- `metrics` - Get Prometheus metrics

**Example workflow:**
```bash
# 1. Login and get token
./test-api.sh login
export JWT_TOKEN="your-token-here"

# 2. Get stats
./test-api.sh status

# 3. Get events
./test-api.sh events
```

---

## 🐛 Troubleshooting

### WebSocket Connection Fails

**Issue:** WebSocket closes immediately after connection.

**Solution:** Ensure you're using the correct token format:
```
wss://api-8004-dev.fly.dev/ws?token=YOUR_JWT_TOKEN
```

Not:
```
wss://api-8004-dev.fly.dev/ws
Headers: Authorization: Bearer TOKEN
```

Browsers don't support custom headers in WebSocket connections.

---

### Authentication Errors

**Issue:** 401 Unauthorized or "Wrong credentials"

**Solutions:**
1. Check password is correct
2. Verify username is "admin" (or configured value)
3. Check JWT_SECRET is properly set on server
4. Ensure token hasn't expired (24h default)

---

### No Events Returned

**Issue:** Events endpoint returns empty array.

**Possible reasons:**
1. No events indexed yet (check `/stats` for last_synced_block)
2. Filters too restrictive (try removing filters)
3. Events outside time/block range (increase `hours` or `blocks`)

**Solution:**
```bash
# Get all available events
curl -H "Authorization: Bearer $TOKEN" \
  "https://api-8004-dev.fly.dev/events?blocks=999999&limit=50000"
```

---

## 📊 Expected Results

### Successful Health Check
```json
{
  "status": "ok",
  "service": "erc8004-indexer"
}
```

### Successful Login
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
  "expires_at": "2025-11-07T16:14:22Z"
}
```

### Successful WebSocket Connection
```
Connected to: wss://api-8004-dev.fly.dev/ws?token=...
✅ Connected!
📦 Message: {"type":"connected","message":"Connected to ERC-8004 event stream"}
```

### Events Response
```json
{
  "success": true,
  "count": 15,
  "events": [
    {
      "id": 15,
      "block_number": 9421561,
      "transaction_hash": "0xa520c4507...",
      "event_type": {"type": "Registered"},
      "contract_address": "0x8004a609...",
      ...
    }
  ]
}
```

---

## 🔗 Related Documentation

- [README.md](README.md) - Main documentation
- [DEPLOYMENT.md](DEPLOYMENT.md) - Deployment guide
- [TEST_REPORT.md](TEST_REPORT.md) - Latest test results

---

**Last Updated:** 2025-11-06
**Version:** 0.1.0
