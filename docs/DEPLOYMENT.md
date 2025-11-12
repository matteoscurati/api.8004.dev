# Deployment Guide - API 8004.dev

Quick reference guide for deploying **api.8004.dev** to Fly.io.

## 🚀 Quick Start (3 commands)

```bash
./deploy-flyio.sh init      # Setup app and database
./deploy-flyio.sh secrets   # Configure secrets
./deploy-flyio.sh deploy    # Deploy!
```

Your app: **https://api-8004-dev.fly.dev**

---

## 📋 Prerequisites

1. **Fly.io account** (free): https://fly.io/app/sign-up
2. **Fly.io CLI installed**:
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```
3. **RPC URL** for Ethereum Sepolia (Alchemy or Infura)

---

## 🎯 Step-by-Step Deployment

### 1. Initialize

```bash
./deploy-flyio.sh init
```

This will:
- Create Fly.io app: `api-8004-dev`
- Create PostgreSQL database: `api-8004-dev-db`
- Attach database to app
- Set up auto-scaling

### 2. Configure Secrets

```bash
./deploy-flyio.sh secrets
```

You'll be prompted for:
- **RPC_URL**: Your Ethereum Sepolia RPC endpoint
- **Contract addresses**: Identity, Reputation, Validation registries
- **JWT_SECRET**: Auto-generated or custom (32+ chars)
- **AUTH_USERNAME**: Admin username (default: admin)
- **AUTH_PASSWORD**: Admin password
- **STARTING_BLOCK**: Block number or "latest"

### 3. Deploy

```bash
./deploy-flyio.sh deploy
```

This will:
- Run tests
- Build Docker image
- Deploy to Fly.io
- Run health checks

**Deployment takes ~5-10 minutes** (Docker build is slow the first time)

---

## 🛠️ Management Commands

### View Logs
```bash
./deploy-flyio.sh logs
```

### Check Status
```bash
./deploy-flyio.sh status
```

### Access Database
```bash
./deploy-flyio.sh db-console
```

### SSH into App
```bash
./deploy-flyio.sh ssh
```

### Scale Resources
```bash
./deploy-flyio.sh scale
```

### Destroy Everything
```bash
./deploy-flyio.sh destroy
```

---

## 📊 Monitoring

### Health Check
```bash
curl https://api-8004-dev.fly.dev/health
```

### Metrics (Prometheus)
```bash
curl https://api-8004-dev.fly.dev/metrics
```

### API Status
```bash
curl https://api-8004-dev.fly.dev/api/status
```

---

## 🔒 Authentication

### Get JWT Token
```bash
curl -X POST https://api-8004-dev.fly.dev/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"your-password"}'
```

### Use Token
```bash
curl https://api-8004-dev.fly.dev/api/events \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

---

## 🔧 Configuration

Edit `fly.toml` to customize:

- **Region**: Change `primary_region = "ams"` to your preferred region
- **VM Size**: Edit `memory_mb` under `[[vm]]`
- **Auto-scaling**: Modify `min_machines_running`

Available regions:
- `ams` - Amsterdam
- `fra` - Frankfurt
- `lhr` - London
- `iad` - Washington DC
- `sjc` - San Jose
- `syd` - Sydney

See all regions: `flyctl platform regions`

---

## 📈 Scaling

### Vertical Scaling (More Power)
```bash
flyctl scale vm shared-cpu-2x --app api-8004-dev
flyctl scale memory 1024 --app api-8004-dev
```

### Horizontal Scaling (More Instances)
```bash
flyctl scale count 2 --app api-8004-dev
```

---

## 🐛 Troubleshooting

### Check Logs
```bash
flyctl logs --app api-8004-dev
```

### Restart App
```bash
flyctl apps restart api-8004-dev
```

### Check Database Connection
```bash
flyctl postgres connect --app api-8004-dev-db
```

### Re-deploy
```bash
./deploy-flyio.sh deploy
```

### Common Issues

**1. Build Timeout**
- Fly.io may timeout on first build (Rust compilation is slow)
- Solution: Run `./deploy-flyio.sh deploy` again

**2. Database Connection Failed**
- Check: `flyctl postgres db list --app api-8004-dev-db`
- Re-attach: `flyctl postgres attach api-8004-dev-db --app api-8004-dev`

**3. Health Check Failed**
- Wait 2-3 minutes for app to fully start
- Check logs: `./deploy-flyio.sh logs`

**4. Secrets Not Set**
- Re-run: `./deploy-flyio.sh secrets`
- Or manually: `flyctl secrets set KEY=value --app api-8004-dev`

---

## 💰 Costs

### Free Tier
- 3 shared-cpu-1x VMs (256MB RAM)
- 3GB persistent volume storage
- 160GB outbound data transfer

### Paid Usage
- **Shared CPU 1x (512MB)**: ~$3/month
- **Shared CPU 2x (1GB)**: ~$6/month
- **PostgreSQL 1GB**: Free
- **PostgreSQL 10GB**: ~$2/month

**Typical cost for this project: $5-10/month**

---

## 🔗 Useful Links

- **Fly.io Dashboard**: https://fly.io/dashboard
- **Fly.io Docs**: https://fly.io/docs/
- **Fly.io Status**: https://status.flyio.net/
- **Support**: https://community.fly.io/

---

## 🎓 Next Steps

After deployment:

1. **Test the API**:
   ```bash
   curl https://api-8004-dev.fly.dev/health
   ```

2. **Monitor logs**:
   ```bash
   ./deploy-flyio.sh logs
   ```

3. **Configure custom domain** (optional):
   ```bash
   flyctl certs create your-domain.com --app api-8004-dev
   ```

4. **Set up monitoring** with Prometheus metrics at `/metrics`

5. **Configure rate limiting** via environment variables

---

## 🆘 Get Help

- **Deployment issues**: Check `./deploy-flyio.sh logs`
- **Configuration help**: See README.md
- **Fly.io support**: https://community.fly.io/
- **Project issues**: https://github.com/yourusername/api.8004.dev/issues
