# API 8004.dev - ERC-8004 Event Indexer

A production-ready, high-performance Rust-based indexer for ERC-8004 (Trustless Agents) smart contract events on Ethereum Sepolia testnet.

## Features

### Core Functionality
- **Block-by-block event indexing** from three ERC-8004 registry contracts
- **Hybrid storage**: In-memory cache (DashMap) + PostgreSQL persistence
- **REST API** with flexible query filters and JWT authentication
- **WebSocket support** for real-time event streaming
- **Automatic recovery** from last synced block
- **Rate limit handling** with configurable polling intervals

### Production-Ready Features
- **JWT Authentication** with bcrypt password hashing
- **Configurable CORS** with domain whitelisting
- **Rate limiting** by IP address
- **Prometheus metrics** export at `/metrics`
- **Structured JSON logging** for production monitoring
- **Advanced health checks** with database and cache status
- **Graceful shutdown** handling
- **Retry logic** with exponential backoff
- **Database connection pooling** with configurable limits
- **Environment variables validation** with security warnings
- **Comprehensive test suite** (11 unit tests)

## Architecture

The indexer monitors three ERC-8004 contracts:
- **IdentityRegistry**: Tracks agent registration and metadata
- **ReputationRegistry**: Manages feedback and reputation data
- **ValidationRegistry**: Handles validation requests and responses

### Event Types

**IdentityRegistry:**
- `Registered` - New agent registration
- `MetadataSet` - Agent metadata updates
- `UriUpdated` - Token URI updates

**ReputationRegistry:**
- `NewFeedback` - New feedback submission
- `FeedbackRevoked` - Feedback removal
- `ResponseAppended` - Response to feedback

**ValidationRegistry:**
- `ValidationRequest` - Validation request submitted
- `ValidationResponse` - Validation response provided

## Prerequisites

- **Rust** 1.75+
- **PostgreSQL** 14+
- **Docker** (optional, for containerized deployment)
- **sqlx-cli** (for database migrations)

## Quick Start

### 1. Install Dependencies

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Install PostgreSQL (macOS)
brew install postgresql@16
brew services start postgresql@16

# Install PostgreSQL (Ubuntu/Debian)
sudo apt install postgresql-16 postgresql-contrib
sudo systemctl start postgresql
```

### 2. Setup Database

```bash
createdb api_8004_dev
```

### 3. Configure Environment

Create a `.env` file in the project root:

```env
# Ethereum RPC Configuration
RPC_URL=https://rpc.ankr.com/eth_sepolia/YOUR_API_KEY

# Contract Addresses (Sepolia - Proxy Addresses)
IDENTITY_REGISTRY_ADDRESS=0x8004a6090Cd10A7288092483047B097295Fb8847
REPUTATION_REGISTRY_ADDRESS=0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E
VALIDATION_REGISTRY_ADDRESS=0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5

# Database Configuration
DATABASE_URL=postgresql://USERNAME@localhost:5432/api_8004_dev
DB_MAX_CONNECTIONS=10
DB_MIN_CONNECTIONS=2
DB_ACQUIRE_TIMEOUT_SECS=30

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080

# Indexer Configuration
STARTING_BLOCK=9420233
POLL_INTERVAL_MS=1000

# Storage Configuration
MAX_EVENTS_IN_MEMORY=10000

# JWT Authentication (CHANGE THESE IN PRODUCTION!)
JWT_SECRET=your-super-secret-jwt-key-change-this-in-production
JWT_EXPIRATION_HOURS=24
AUTH_USERNAME=admin

# For development: plain password (INSECURE - use AUTH_PASSWORD_HASH in production)
AUTH_PASSWORD=changeme

# For production: use bcrypt hash
# Generate with: cargo run --bin generate_password_hash
# AUTH_PASSWORD_HASH=$2b$12$...

# CORS Configuration (comma-separated domains)
CORS_ALLOWED_ORIGINS=http://localhost:3000,http://localhost:5173

# Rate Limiting
RATE_LIMIT_REQUESTS=100
RATE_LIMIT_WINDOW_SECS=60

# Logging (optional)
# LOG_FORMAT=json  # Use JSON format for production
```

### 4. Run Migrations and Start

```bash
# Run database migrations
sqlx migrate run

# Build and run
cargo build --release
cargo run --release
```

## API Documentation

### Authentication

All protected endpoints require JWT authentication. First, obtain a token:

```bash
# Login to get JWT token
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}'

# Response:
# {
#   "token": "eyJ0eXAiOiJKV1QiLCJhbGc...",
#   "expires_at": "2025-11-06T17:00:00Z"
# }
```

Use the token in subsequent requests:

```bash
curl -H "Authorization: Bearer YOUR_TOKEN_HERE" \
  http://localhost:8080/events
```

### Public Endpoints

#### GET `/health`
Basic health check (no authentication required)

```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "ok",
  "service": "api.8004.dev"
}
```

#### GET `/health/detailed`
Advanced health check with database and cache status (no authentication required)

```bash
curl http://localhost:8080/health/detailed
```

Response:
```json
{
  "status": "healthy",
  "service": "api.8004.dev",
  "timestamp": "2025-11-05T17:00:00Z",
  "checks": {
    "database": {
      "status": "healthy",
      "last_synced_block": 9420500
    },
    "cache": {
      "status": "healthy",
      "size": 150,
      "max_size": 10000,
      "utilization_percent": "1.50"
    }
  }
}
```

#### GET `/metrics`
Prometheus metrics endpoint (no authentication required)

```bash
curl http://localhost:8080/metrics
```

#### POST `/login`
Authenticate and receive JWT token (no authentication required)

```bash
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"yourpassword"}'
```

### Protected Endpoints (Require Authentication)

#### GET `/events`
Query recent events with flexible filters

**Query Parameters:**
- `blocks` (optional): Number of blocks to look back (default: 100)
- `hours` (optional): Number of hours to look back (overrides `blocks`)
- `contract` (optional): Filter by contract address
- `event_type` (optional): Filter by event type
- `limit` (optional): Maximum number of results (default: 1000)

**Examples:**

```bash
# Set token for convenience
TOKEN="your_jwt_token_here"

# Get last 100 blocks of events
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events"

# Get events from last 24 hours
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events?hours=24"

# Get last 5 events
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events?limit=5"

# Get NewFeedback events only
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events?event_type=NewFeedback"

# Combine filters
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events?hours=48&event_type=Registered&limit=10"
```

**Response Format:**
```json
{
  "success": true,
  "count": 5,
  "events": [
    {
      "id": 7,
      "block_number": 9420240,
      "block_timestamp": "2025-10-15T23:13:24Z",
      "transaction_hash": "0x561afe992546abba...",
      "log_index": 67,
      "contract_address": "0x8004cb39f29c09145f24ad9dde2a108c1a2cdfc5",
      "event_type": {
        "type": "ValidationResponse"
      },
      "event_data": {
        "agent_id": "0",
        "validator_address": "0x15cbd54a73ac8e18ee84bea668ef0bed5daf14dd",
        "request_hash": "0x4302453421b1be3efadf32396c3798935be5e3aa8189db1f31bbf13d87c89a47",
        "response": 100,
        "response_uri": "ipfs://QmPsp7xxSXdT3tDGaBE66HVMd5jKJii2Aqz8PqyiiUiCj8",
        "response_hash": "0x0000000000000000000000000000000000000000000000000000000000000000",
        "tag": "0x70b2891d3251d60a68c8434b7f92b7b7994aa1c283917b93ade83614ce49335e"
      },
      "created_at": "2025-11-05T16:23:31.026388Z"
    }
  ]
}
```

#### GET `/stats`
Get indexer statistics

```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/stats
```

Response:
```json
{
  "last_synced_block": 9420500,
  "cache_size": 150,
  "cache_max_size": 10000
}
```

#### WebSocket `/ws`
Real-time event streaming

```javascript
const ws = new WebSocket('ws://localhost:8080/ws', {
  headers: {
    'Authorization': 'Bearer YOUR_TOKEN_HERE'
  }
});

ws.onmessage = (event) => {
  const data = JSON.parse(event.data);
  console.log('New event:', data);
};
```

## Deployment

### Option 1: Docker Compose (Recommended)

The easiest way to deploy the entire stack:

```bash
# Create .env file with your configuration
cp .env.example .env

# Edit .env with your settings
vim .env

# Start services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down
```

The `docker-compose.yml` includes:
- PostgreSQL database with automatic migrations
- ERC-8004 indexer service
- Health checks and automatic restarts
- Volume persistence for database

### Option 2: Systemd Service (Bare Metal)

For production deployment on Linux servers:

```bash
# 1. Build release binary
cargo build --release

# 2. Create deployment directory
sudo mkdir -p /opt/api.8004.dev
sudo cp target/release/api_8004_dev /opt/api.8004.dev/
sudo cp -r migrations /opt/api.8004.dev/
sudo cp .env /opt/api.8004.dev/

# 3. Create service user
sudo useradd -r -s /bin/false indexer
sudo chown -R indexer:indexer /opt/api.8004.dev

# 4. Install systemd service
sudo cp api.8004.dev.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable api.8004.dev
sudo systemctl start api.8004.dev

# 5. Check status
sudo systemctl status api.8004.dev

# View logs
sudo journalctl -u api.8004.dev -f
```

### Option 3: Manual Deployment

```bash
# 1. Setup PostgreSQL database
createdb api_8004_dev
sqlx migrate run

# 2. Build release binary
cargo build --release

# 3. Run with production settings
./target/release/api_8004_dev
```

### Reverse Proxy (Nginx)

For production, use Nginx as a reverse proxy:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://127.0.0.1:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

For HTTPS (recommended):
```bash
sudo certbot --nginx -d your-domain.com
```

### Option 4: Fly.io (Cloud Deployment) ⭐

**Recommended for quick cloud deployment with managed PostgreSQL.**

Fly.io offers:
- ✅ Free tier available (3 VMs with 256MB RAM)
- ✅ Managed PostgreSQL database
- ✅ Automatic HTTPS/SSL
- ✅ Global edge deployment
- ✅ Zero-downtime deployments

#### Prerequisites

Install Fly.io CLI:
```bash
curl -L https://fly.io/install.sh | sh
```

#### Quick Start with Deploy Script

We provide an automated deployment script:

```bash
# 1. Initialize Fly.io app and database
./deploy-flyio.sh init

# 2. Set required secrets (interactive)
./deploy-flyio.sh secrets

# 3. Deploy the application
./deploy-flyio.sh deploy
```

Your app will be available at: `https://api-8004-dev.fly.dev`

#### Available Commands

```bash
./deploy-flyio.sh init       # Initialize app and database
./deploy-flyio.sh deploy     # Deploy application
./deploy-flyio.sh secrets    # Set secrets interactively
./deploy-flyio.sh logs       # View live logs
./deploy-flyio.sh status     # Check app status
./deploy-flyio.sh db-console # Connect to PostgreSQL
./deploy-flyio.sh ssh        # SSH into the app
./deploy-flyio.sh scale      # Scale resources
./deploy-flyio.sh destroy    # Destroy app (WARNING: irreversible)
```

#### Manual Fly.io Deployment

If you prefer manual control:

```bash
# 1. Login to Fly.io
flyctl auth login

# 2. Create app
flyctl apps create api-8004-dev

# 3. Create PostgreSQL database
flyctl postgres create --name api-8004-dev-db --region ams

# 4. Attach database to app
flyctl postgres attach api-8004-dev-db --app api-8004-dev

# 5. Set required secrets
flyctl secrets set \
  RPC_URL="https://eth-sepolia.g.alchemy.com/v2/YOUR_KEY" \
  IDENTITY_REGISTRY_ADDRESS="0x8004a6090Cd10A7288092483047B097295Fb8847" \
  REPUTATION_REGISTRY_ADDRESS="0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E" \
  VALIDATION_REGISTRY_ADDRESS="0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5" \
  JWT_SECRET="your-secret-key-min-32-chars" \
  AUTH_USERNAME="admin" \
  AUTH_PASSWORD="your-password" \
  STARTING_BLOCK="latest" \
  --app api-8004-dev

# 6. Deploy
flyctl deploy --app api-8004-dev

# 7. Check status
flyctl status --app api-8004-dev

# 8. View logs
flyctl logs --app api-8004-dev
```

#### Fly.io Configuration

The deployment is configured via `fly.toml`:
- **Region**: Amsterdam (ams) - change in fly.toml if needed
- **VM Size**: shared-cpu-1x with 512MB RAM (upgradeable)
- **Auto-scaling**: Enabled with min 1 machine
- **Health checks**: Automatic monitoring of `/health` endpoint
- **HTTPS**: Automatic SSL certificate

#### Scaling on Fly.io

Scale up for production traffic:

```bash
# Scale VM size
flyctl scale vm shared-cpu-2x --app api-8004-dev

# Scale memory
flyctl scale memory 1024 --app api-8004-dev

# Scale instances (for high availability)
flyctl scale count 2 --app api-8004-dev
```

#### Monitoring

```bash
# View real-time logs
flyctl logs --app api-8004-dev

# Check app metrics
flyctl status --app api-8004-dev

# Connect to database
flyctl postgres connect --app api-8004-dev-db

# SSH into the machine
flyctl ssh console --app api-8004-dev
```

#### Costs

Fly.io pricing (as of 2024):
- **Free tier**: 3 VMs (256MB RAM), 3GB storage
- **Shared CPU 1x (512MB)**: ~$3/month
- **Shared CPU 2x (1GB)**: ~$6/month
- **PostgreSQL**: Free 1GB, then ~$2/GB/month

Total estimated cost for small-medium deployment: **$5-10/month**

## Security Best Practices

### Before Production Deployment

1. **Generate Secure JWT Secret**
   ```bash
   # Generate random 32+ character secret
   openssl rand -base64 32
   ```

2. **Hash Your Password**
   ```bash
   # Use bcrypt hashing
   cargo run --bin generate_password_hash
   # Copy the hash to AUTH_PASSWORD_HASH in .env
   # Remove AUTH_PASSWORD from .env
   ```

3. **Configure CORS**
   ```env
   # Replace with your actual frontend domains
   CORS_ALLOWED_ORIGINS=https://yourdomain.com,https://app.yourdomain.com
   ```

4. **Adjust Rate Limiting**
   ```env
   RATE_LIMIT_REQUESTS=100  # Requests per IP
   RATE_LIMIT_WINDOW_SECS=60  # Time window
   ```

5. **Enable JSON Logging**
   ```env
   LOG_FORMAT=json  # Better for log aggregation tools
   ```

## Monitoring

### Prometheus Metrics

The indexer exposes Prometheus metrics at `/metrics`:

```bash
curl http://localhost:8080/metrics
```

Key metrics:
- `events_indexed_total` - Total events indexed by type and contract
- `blocks_synced_total` - Total blocks processed
- `last_synced_block` - Current block height
- `http_requests_total` - HTTP request counts by endpoint
- `http_request_duration_seconds` - Request latency histogram
- `db_queries_total` - Database query counts
- `cache_size` - Current cache utilization
- `rpc_requests_total` - RPC request counts and errors

### Example Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'api.8004.dev'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Grafana Dashboard

Import the provided Grafana dashboard (coming soon) or create custom visualizations for:
- Indexing rate (events/second)
- Block processing latency
- Cache hit rate
- API response times
- Error rates

## Testing

The project includes comprehensive unit tests:

```bash
# Run all tests
cargo test

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test test_jwt_token_creation

# Run tests in single thread (avoid env var conflicts)
cargo test -- --test-threads=1
```

Test coverage:
- ✅ **11 unit tests** covering:
  - JWT token creation and validation
  - Password hashing and verification
  - Credential validation (plain and bcrypt)
  - Configuration loading and validation
  - Security settings validation

## Database Operations

### Reset from Specific Block

**Option 1: Update indexer state only (keeps existing events)**

```bash
psql api_8004_dev -c "UPDATE indexer_state SET last_synced_block = BLOCK_NUMBER WHERE id = 1;"
```

**Option 2: Complete database reset**

```bash
dropdb api_8004_dev && createdb api_8004_dev && sqlx migrate run
```

**Option 3: Delete events from specific block onwards**

```bash
psql api_8004_dev -c "DELETE FROM events WHERE block_number >= BLOCK_NUMBER;"
psql api_8004_dev -c "UPDATE indexer_state SET last_synced_block = BLOCK_NUMBER - 1 WHERE id = 1;"
```

### Query Database Directly

```bash
# Connect to database
psql api_8004_dev

# View all events
SELECT * FROM events ORDER BY block_number DESC LIMIT 10;

# Count events by type
SELECT event_type, COUNT(*) FROM events GROUP BY event_type;

# Check indexer state
SELECT * FROM indexer_state;

# View events from specific contract
SELECT * FROM events WHERE contract_address = '0x8004a6090cd10a7288092483047b097295fb8847';

# Get events from last hour
SELECT * FROM events
WHERE block_timestamp > NOW() - INTERVAL '1 hour'
ORDER BY block_number DESC;
```

## Project Structure

```
api.8004.dev/
├── src/
│   ├── main.rs              # Application entry point
│   ├── lib.rs               # Library exports for testing
│   ├── api/                 # REST API and WebSocket handlers
│   │   └── mod.rs           # Routes, authentication, CORS
│   ├── auth/                # JWT authentication and password hashing
│   │   └── mod.rs           # JWT config, validation, bcrypt
│   ├── config.rs            # Configuration management with validation
│   ├── contracts/           # Contract ABI and event definitions
│   │   └── mod.rs           # Solidity event definitions
│   ├── indexer/             # Core indexing logic
│   │   └── mod.rs           # Block processing, event extraction
│   ├── metrics/             # Prometheus metrics
│   │   └── mod.rs           # Metric definitions and recording
│   ├── models/              # Data models and types
│   │   ├── mod.rs
│   │   └── events.rs        # Event structures
│   ├── rate_limit/          # Rate limiting middleware
│   │   └── mod.rs
│   ├── retry/               # Retry logic with exponential backoff
│   │   └── mod.rs
│   └── storage/             # Database and cache management
│       └── mod.rs           # PostgreSQL + DashMap hybrid storage
├── migrations/              # Database migrations
│   └── 001_init.sql         # Initial schema
├── tests/                   # Integration tests
├── Cargo.toml              # Rust dependencies
├── Dockerfile              # Multi-stage build
├── docker-compose.yml      # Complete stack setup
├── api.8004.dev.service # Systemd service file
├── .env                    # Configuration (not in git)
├── .env.example            # Example configuration
├── PRODUCTION_DEPLOY.md    # Detailed deployment guide
├── API_AUTHENTICATION.md   # Authentication guide
└── README.md               # This file
```

## Development

**Run in development mode:**
```bash
cargo run
```

**Watch mode (auto-reload):**
```bash
cargo install cargo-watch
cargo watch -x run
```

**Check code:**
```bash
cargo check
```

**Format code:**
```bash
cargo fmt
```

**Lint code:**
```bash
cargo clippy
```

**Generate password hash:**
```bash
cargo run --bin generate_password_hash
```

## Troubleshooting

### Rate Limit Errors (429)

If you see rate limit errors:
1. Increase `POLL_INTERVAL_MS` in `.env` (e.g., to 2000 or 5000)
2. Switch to a different RPC provider (Ankr is recommended for Sepolia)
3. Use a paid RPC endpoint with higher rate limits

### Database Connection Issues

```bash
# Check PostgreSQL is running
pg_isready

# macOS
brew services restart postgresql@16

# Ubuntu/Debian
sudo systemctl restart postgresql

# Verify database exists
psql -l | grep api_8004_dev

# Check connection string
psql $DATABASE_URL
```

### Authentication Issues

```bash
# Generate new password hash
cargo run --bin generate_password_hash

# Test login endpoint
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"yourpassword"}'

# Verify JWT secret is at least 32 characters
echo -n "$JWT_SECRET" | wc -c
```

### No Events Being Indexed

1. Verify contract addresses are **proxy addresses** (not implementation)
2. Check RPC URL is accessible:
   ```bash
   curl -X POST $RPC_URL \
     -H "Content-Type: application/json" \
     -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
   ```
3. Verify starting block has events on [Sepolia Etherscan](https://sepolia.etherscan.io)
4. Check logs for error messages
5. Verify database migrations ran successfully:
   ```bash
   psql api_8004_dev -c "\d events"
   ```

### Performance Issues

1. **Increase database pool size:**
   ```env
   DB_MAX_CONNECTIONS=20
   DB_MIN_CONNECTIONS=5
   ```

2. **Adjust cache size:**
   ```env
   MAX_EVENTS_IN_MEMORY=20000
   ```

3. **Monitor metrics:**
   ```bash
   curl http://localhost:8080/health/detailed
   ```

4. **Check database performance:**
   ```sql
   SELECT pg_stat_statements_reset();  -- Reset stats
   SELECT * FROM pg_stat_statements ORDER BY total_exec_time DESC LIMIT 10;
   ```

## Configuration Reference

### Required Environment Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `RPC_URL` | Ethereum RPC endpoint | `https://rpc.ankr.com/eth_sepolia/...` |
| `IDENTITY_REGISTRY_ADDRESS` | Identity contract proxy address | `0x8004a6090Cd10A7288092483...` |
| `REPUTATION_REGISTRY_ADDRESS` | Reputation contract proxy address | `0x8004B8FD1A363aa02fDC0763...` |
| `VALIDATION_REGISTRY_ADDRESS` | Validation contract proxy address | `0x8004CB39f29c09145F24Ad9d...` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://user@localhost/db` |
| `JWT_SECRET` | JWT signing secret (32+ chars) | Generated securely |
| `AUTH_USERNAME` | Admin username | `admin` |
| `AUTH_PASSWORD` or `AUTH_PASSWORD_HASH` | Admin password/hash | Bcrypt hash recommended |

### Optional Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `SERVER_HOST` | `0.0.0.0` | API server bind address |
| `SERVER_PORT` | `8080` | API server port |
| `STARTING_BLOCK` | `latest` | Block to start indexing from |
| `POLL_INTERVAL_MS` | `12000` | Delay between blocks (ms) |
| `MAX_EVENTS_IN_MEMORY` | `10000` | Cache size limit |
| `DB_MAX_CONNECTIONS` | `10` | Max database connections |
| `DB_MIN_CONNECTIONS` | `2` | Min database connections |
| `DB_ACQUIRE_TIMEOUT_SECS` | `30` | Connection timeout |
| `JWT_EXPIRATION_HOURS` | `24` | JWT token expiration |
| `CORS_ALLOWED_ORIGINS` | `*` | CORS whitelist (comma-separated) |
| `RATE_LIMIT_REQUESTS` | `100` | Max requests per IP |
| `RATE_LIMIT_WINDOW_SECS` | `60` | Rate limit time window |
| `LOG_FORMAT` | `text` | Log format (`text` or `json`) |

## Performance

- **Indexing Speed**: ~1 block/second (depending on RPC provider)
- **API Latency**: <50ms for cached queries
- **Memory Usage**: ~50-100MB (base) + cache
- **Database Size**: ~1KB per event

## Roadmap

- [ ] GraphQL API support
- [ ] Advanced filtering and aggregation queries
- [ ] Historical data export
- [ ] Event replay functionality
- [ ] Multi-chain support
- [ ] WebSocket authentication improvements
- [ ] Distributed deployment support

## License

MIT

## Contributing

Contributions are welcome! Please:
1. Fork the repository
2. Create a feature branch
3. Write tests for new functionality
4. Ensure all tests pass (`cargo test`)
5. Submit a pull request

## Resources

- [ERC-8004 Specification](https://eips.ethereum.org/EIPS/eip-8004)
- [Alloy Documentation](https://alloy.rs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Axum Documentation](https://docs.rs/axum/)
- [Tokio Documentation](https://tokio.rs/)

## Support

For issues and questions:
- Open an issue on GitHub
- Check existing documentation in `PRODUCTION_DEPLOY.md` and `API_AUTHENTICATION.md`
- Review troubleshooting section above

## Acknowledgments

Built with:
- [Alloy](https://github.com/alloy-rs/alloy) - Ethereum library
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [SQLx](https://github.com/launchbadge/sqlx) - Async SQL toolkit
- [Tokio](https://github.com/tokio-rs/tokio) - Async runtime
