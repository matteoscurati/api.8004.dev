# Scripts Directory

This directory contains utility scripts for development, testing, deployment, and monitoring of the ERC-8004 Indexer.

## Directory Structure

```
scripts/
├── deployment/     # Deployment and release scripts
├── development/    # Development setup and utilities
├── monitoring/     # Production monitoring and status scripts
├── testing/        # Automated testing scripts
└── utils/          # General utility scripts
```

## Quick Reference

### Development

```bash
# Initial setup
./scripts/development/setup.sh

# Setup test database
./scripts/development/setup-test-db.sh

# Find test blocks with events
./scripts/development/find-test-blocks.sh
```

### Testing

```bash
# Quick API test
./scripts/testing/test-quick.sh

# Full local test suite
./scripts/testing/test-local-full.sh

# Test specific functionality
./scripts/testing/test-pagination.sh
./scripts/testing/test-category-filter.sh
```

### Monitoring

```bash
# Chain status report
./scripts/monitoring/chain-status-report.sh

# Production quick check
./scripts/monitoring/check-prod-quick.sh

# Monitor sync progress
./scripts/monitoring/monitor-sync.sh
```

### Deployment

```bash
# Pre-deployment checks
./scripts/deployment/pre-deploy-check.sh

# Deploy to Fly.io
./scripts/deployment/deploy-flyio.sh

# Post-deployment verification
./scripts/deployment/post-deploy-check.sh
```

### Utilities

```bash
# Get all events from API
./scripts/utils/get-all-events.sh

# Check event types in database
./scripts/utils/check-event-types.sh
```

## Environment Variables

Most scripts support environment variables for configuration. Common variables:

```bash
# API Configuration
export API_URL="http://localhost:8080"          # or production URL
export API_USERNAME="admin"
export API_PASSWORD="your-password"

# Database (for setup scripts)
export DATABASE_URL="postgresql://user@localhost/dbname"

# Fly.io (for deployment)
export FLY_API_TOKEN="your-token"
```

## Best Practices

1. **Always run scripts from the repository root**:
   ```bash
   ./scripts/testing/test-quick.sh
   ```

2. **Set environment variables before running**:
   ```bash
   export API_PASSWORD="mypassword"
   ./scripts/monitoring/chain-status-report.sh
   ```

3. **Check script help for usage**:
   ```bash
   ./scripts/deployment/deploy-flyio.sh --help
   ```

## Adding New Scripts

When adding new scripts:

1. Place in appropriate subdirectory
2. Make executable: `chmod +x script-name.sh`
3. Add shebang: `#!/bin/bash`
4. Include usage documentation in comments
5. Use environment variables for configuration
6. Update the relevant subdirectory README

## See Also

- [Development Setup](../docs/LOCAL_TESTING.md)
- [Deployment Guide](../docs/DEPLOYMENT.md)
- [API Documentation](../docs/API_EXAMPLES.md)
