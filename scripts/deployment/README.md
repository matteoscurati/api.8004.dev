# Deployment Scripts

Scripts for deploying and verifying the ERC-8004 Indexer on Fly.io.

## Scripts

### `deploy-flyio.sh`

Full deployment to Fly.io with all checks and verifications.

**Usage:**
```bash
./scripts/deployment/deploy-flyio.sh
```

**What it does:**
1. Runs pre-deployment checks
2. Builds and deploys to Fly.io
3. Runs post-deployment verification
4. Reports deployment status

**Requirements:**
- `flyctl` installed and authenticated
- Environment variables configured in Fly.io secrets

### `pre-deploy-check.sh`

Pre-deployment checklist and validation.

**Usage:**
```bash
./scripts/deployment/pre-deploy-check.sh
```

**Checks:**
- Code compilation
- Test suite passing
- Linting and code quality
- Configuration validation
- Migration readiness
- Docker build success

### `post-deploy-check.sh`

Post-deployment verification and smoke tests.

**Usage:**
```bash
export API_URL="https://api-8004-dev.fly.dev"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/deployment/post-deploy-check.sh
```

**Verifies:**
- API health endpoint
- Database connectivity
- Authentication working
- All chains syncing
- Event retrieval
- WebSocket functionality

## Environment Variables

```bash
# Fly.io
FLY_API_TOKEN          # Fly.io API token (for CI/CD)
FLY_APP_NAME           # App name (default: api-8004-dev)

# Post-deployment checks
API_URL                # Deployed API URL
API_USERNAME           # Admin username
API_PASSWORD           # Admin password
```

## Deployment Workflow

Recommended deployment process:

```bash
# 1. Run pre-deployment checks
./scripts/deployment/pre-deploy-check.sh

# 2. Deploy if checks pass
./scripts/deployment/deploy-flyio.sh

# 3. Verify deployment
export API_URL="https://your-app.fly.dev"
export API_PASSWORD="your-password"
./scripts/deployment/post-deploy-check.sh
```

## Rollback

If deployment fails:

```bash
# Check logs
flyctl logs

# Rollback to previous version
flyctl releases list
flyctl releases rollback <version>
```

## See Also

- [Deployment Guide](../../docs/DEPLOYMENT.md)
- [CI/CD Setup](../../docs/CI_CD_SETUP.md)
