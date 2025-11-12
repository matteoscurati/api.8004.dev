# Development Scripts

Utilities for local development setup and testing.

## Scripts

### `setup.sh`

Initial project setup for development.

**Usage:**
```bash
./scripts/development/setup.sh
```

**What it does:**
- Installs Rust toolchain if needed
- Sets up database
- Runs migrations
- Creates `.env` from template
- Installs dependencies
- Builds project

### `setup-test-db.sh`

Setup test database with sample data.

**Usage:**
```bash
export DATABASE_URL="postgresql://user@localhost/erc8004_test"
./scripts/development/setup-test-db.sh
```

**Features:**
- Creates test database
- Runs migrations
- Inserts sample events for testing
- Configures test chains

**Environment Variables:**
```bash
DATABASE_URL    # Test database connection string
```

### `find-test-blocks.sh`

Find blockchain blocks containing ERC-8004 events for testing.

**Usage:**
```bash
./scripts/development/find-test-blocks.sh
```

**Output:** `test-blocks-real.json` with actual blocks containing events from each chain.

**Use Case:**
- Identify blocks for replay testing
- Verify event detection
- Create test fixtures

## Common Workflows

### First Time Setup

```bash
# 1. Initial setup
./scripts/development/setup.sh

# 2. Start database
docker-compose up -d postgres

# 3. Run migrations
sqlx migrate run

# 4. Start development server
cargo run
```

### Reset Development Database

```bash
# Drop and recreate
dropdb erc8004_indexer
createdb erc8004_indexer

# Run migrations
sqlx migrate run

# Or use setup-test-db for sample data
./scripts/development/setup-test-db.sh
```

### Finding New Test Data

```bash
# Find blocks with events
./scripts/development/find-test-blocks.sh

# Review output
cat test-blocks-real.json

# Use in tests
./scripts/testing/test-quick.sh
```

## Environment Variables

```bash
DATABASE_URL            # Database connection string
RPC_URL                 # Ethereum RPC endpoint (for find-test-blocks)
IDENTITY_REGISTRY       # Identity contract address
REPUTATION_REGISTRY     # Reputation contract address
VALIDATION_REGISTRY     # Validation contract address
```

## Tips

1. **Use local .env file**: Copy `.env.example` to `.env` and customize
2. **Database tools**: Install `postgresql-client` for psql access
3. **Watch mode**: Use `cargo watch -x run` for auto-reload during development
4. **Logs**: Set `RUST_LOG=debug` for verbose logging

## Troubleshooting

### Database Connection Error

```bash
# Check if PostgreSQL is running
pg_isready

# Check connection string
echo $DATABASE_URL

# Reset database
./scripts/development/setup-test-db.sh
```

### Migration Errors

```bash
# Check migration status
sqlx migrate info

# Revert last migration
sqlx migrate revert

# Rerun all migrations
sqlx migrate run
```

## See Also

- [Local Testing Guide](../../docs/LOCAL_TESTING.md)
- [Quick Start Guide](../../docs/QUICK_START.md)
