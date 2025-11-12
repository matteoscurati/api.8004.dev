# Production Deployment Guide

This guide covers deploying the ERC-8004 indexer to production.

## Completed Production Features ✅

- ✅ **Password Hashing**: bcrypt for secure password storage
- ✅ **CORS Configuration**: Whitelist specific domains
- ✅ **Prometheus Metrics**: `/metrics` endpoint for monitoring
- ✅ **Graceful Shutdown**: SIGTERM/SIGINT handling
- ✅ **Docker Support**: Dockerfile and docker-compose.yml
- ✅ **Systemd Service**: For bare metal deployments
- ✅ **JWT Authentication**: Secure API access

## Pending Features ⚠️

- ⚠️ Rate Limiting (use nginx/reverse proxy for now)
- ⚠️ Structured JSON Logging
- ⚠️ Advanced Health Checks
- ⚠️ Retry Logic with Exponential Backoff
- ⚠️ Connection Pool Tuning

## Deployment Options

### Option 1: Docker Compose (Recommended)

**1. Clone and configure:**
```bash
git clone <your-repo>
cd api.8004.dev
cp .env.example .env
```

**2. Generate password hash:**
```bash
cargo run --bin generate_password_hash
# Add output to .env as AUTH_PASSWORD_HASH
```

**3. Configure .env:**
```bash
# Required changes:
JWT_SECRET=<generate-strong-random-string>
AUTH_PASSWORD_HASH=<from-step-2>
RPC_URL=<your-rpc-endpoint>
CORS_ALLOWED_ORIGINS=https://yourdomain.com
```

**4. Start services:**
```bash
docker-compose up -d
```

**5. Check health:**
```bash
curl http://localhost:8080/health
curl http://localhost:8080/metrics
```

### Option 2: Systemd Service (Bare Metal)

**1. Build release binary:**
```bash
cargo build --release
```

**2. Install:**
```bash
sudo useradd -r -s /bin/false indexer
sudo mkdir -p /opt/api.8004.dev
sudo cp target/release/api_8004_dev /opt/api.8004.dev/
sudo cp .env /opt/api.8004.dev/
sudo cp -r migrations /opt/api.8004.dev/
sudo chown -R indexer:indexer /opt/api.8004.dev
```

**3. Install systemd service:**
```bash
sudo cp api.8004.dev.service /etc/systemd/system/
sudo systemctl daemon-reload
sudo systemctl enable api.8004.dev
sudo systemctl start api.8004.dev
```

**4. Check status:**
```bash
sudo systemctl status api.8004.dev
sudo journalctl -u api.8004.dev -f
```

## Production Configuration

### Environment Variables

```bash
# === CRITICAL SECURITY ===
JWT_SECRET=<64-char-random-string>
AUTH_PASSWORD_HASH=<bcrypt-hash>
CORS_ALLOWED_ORIGINS=https://app.yourdomain.com,https://admin.yourdomain.com

# === RPC Configuration ===
RPC_URL=https://your-production-rpc-endpoint
POLL_INTERVAL_MS=2000  # Adjust based on RPC rate limits

# === Database ===
DATABASE_URL=postgresql://user:pass@host:5432/dbname
# Use connection pooling:
# Max connections: 10-20 for most workloads

# === Indexer ===
STARTING_BLOCK=<deployment-block>
MAX_EVENTS_IN_MEMORY=10000

# === Server ===
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
```

### Nginx Reverse Proxy

```nginx
server {
    listen 443 ssl http2;
    server_name api.yourdomain.com;

    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api_limit:10m rate=10r/s;
    limit_req zone=api_limit burst=20 nodelay;

    location / {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # WebSocket support
    location /ws {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
    }
}
```

## Monitoring

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'api.8004.dev'
    static_configs:
      - targets: ['localhost:8080']
    metrics_path: '/metrics'
    scrape_interval: 15s
```

### Key Metrics

- `events_indexed_total` - Total events indexed
- `blocks_synced_total` - Total blocks processed
- `last_synced_block` - Current sync position
- `http_requests_total` - API request count
- `http_request_duration_seconds` - API latency
- `db_queries_total` - Database query count
- `cache_utilization` - Memory cache usage %

### Grafana Dashboard

Import dashboard from `grafana-dashboard.json` (TODO: create this)

## Backup Strategy

### Database Backups

```bash
# Daily backup
0 2 * * * pg_dump api_8004_dev | gzip > /backups/erc8004_$(date +\%Y\%m\%d).sql.gz

# Keep last 30 days
find /backups -name "erc8004_*.sql.gz" -mtime +30 -delete
```

### State Recovery

If indexer crashes, it will automatically resume from `last_synced_block`.

To reset from specific block:
```sql
UPDATE indexer_state SET last_synced_block = BLOCK_NUMBER WHERE id = 1;
```

## Security Checklist

- [ ] Change `JWT_SECRET` to strong random string (64+ chars)
- [ ] Use `AUTH_PASSWORD_HASH` (not plain `AUTH_PASSWORD`)
- [ ] Configure `CORS_ALLOWED_ORIGINS` (not `*`)
- [ ] Enable HTTPS (use nginx/caddy reverse proxy)
- [ ] Use separate database user with limited permissions
- [ ] Enable PostgreSQL SSL connections
- [ ] Set up firewall rules (only allow necessary ports)
- [ ] Regular security updates
- [ ] Enable audit logging
- [ ] Monitor `/metrics` for anomalies

## Scaling

### Horizontal Scaling

- **Indexer**: Single instance only (stateful)
- **API**: Multiple instances behind load balancer
- **Database**: PostgreSQL replication for reads

### Vertical Scaling

- Increase `MAX_EVENTS_IN_MEMORY` for better performance
- Tune PostgreSQL `shared_buffers`, `work_mem`
- Use faster RPC endpoint

## Troubleshooting

### High Memory Usage
```bash
# Check cache utilization
curl http://localhost:8080/stats

# Reduce MAX_EVENTS_IN_MEMORY if needed
```

### RPC Rate Limits
```bash
# Increase POLL_INTERVAL_MS
# Use dedicated RPC endpoint
# Check /metrics for rpc_requests_total
```

### Database Connection Issues
```bash
# Check connection pool
SELECT count(*) FROM pg_stat_activity WHERE datname = 'api_8004_dev';

# Tune pool size in config
```

### Slow Queries
```bash
# Enable slow query log
ALTER DATABASE api_8004_dev SET log_min_duration_statement = 1000;

# Check indexes
\di in psql
```

## Maintenance

### Updating
```bash
# Docker
git pull
docker-compose build
docker-compose up -d

# Systemd
cargo build --release
sudo systemctl stop api.8004.dev
sudo cp target/release/api_8004_dev /opt/api.8004.dev/
sudo systemctl start api.8004.dev
```

### Database Migrations
```bash
sqlx migrate run
```

### Log Rotation
```bash
# Add to /etc/logrotate.d/api.8004.dev
/var/log/api.8004.dev/*.log {
    daily
    rotate 14
    compress
    delaycompress
    notifempty
    create 0640 indexer indexer
}
```

## Support

- GitHub Issues: <your-repo>/issues
- Documentation: README.md
- API Docs: API_AUTHENTICATION.md
