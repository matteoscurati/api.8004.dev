# CI/CD Setup & Test Coverage

**Status**: ✅ Complete
**Date**: 2025-01-10
**Coverage**: 100% (8/8 event types × 5/5 chains)

---

## 📊 Test Coverage Summary

### Event Types Coverage (8/8) ✅

| Event Type | Contract | Tested | Test Location |
|------------|----------|--------|---------------|
| **Registered** | IdentityRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **MetadataSet** | IdentityRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **UriUpdated** | IdentityRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **NewFeedback** | ReputationRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **FeedbackRevoked** | ReputationRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **ResponseAppended** | ReputationRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **ValidationRequest** | ValidationRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |
| **ValidationResponse** | ValidationRegistry | ✅ | `test_all_event_types_storage_and_retrieval` |

### Chain Coverage (5/5) ✅

| Chain | Chain ID | Test |
|-------|----------|------|
| **Ethereum Sepolia** | 11155111 | `test_ethereum_sepolia_event_processing_and_storage` |
| **Base Sepolia** | 84532 | `test_base_sepolia_event_processing_and_storage` |
| **Linea Sepolia** | 59141 | `test_linea_sepolia_event_processing_and_storage` |
| **Polygon Amoy** | 80002 | `test_polygon_amoy_event_processing` |
| **Hedera Testnet** | 296 | `test_hedera_testnet_event_processing` |

### Integration Tests (8 total) ✅

1. ✅ **test_ethereum_sepolia_event_processing_and_storage** - Full event processing for Ethereum Sepolia
2. ✅ **test_base_sepolia_event_processing_and_storage** - Base Sepolia event processing
3. ✅ **test_linea_sepolia_event_processing_and_storage** - Linea Sepolia event processing
4. ✅ **test_polygon_amoy_event_processing** - Polygon Amoy event processing
5. ✅ **test_hedera_testnet_event_processing** - Hedera Testnet event processing
6. ✅ **test_multi_chain_isolation** - Verifies events are isolated by chain_id
7. ✅ **test_all_event_types_storage_and_retrieval** - All 8 event types storage/retrieval
8. ✅ **test_crash_recovery_block_minus_one** - Crash recovery mechanism (block - 1)

---

## 🚀 GitHub Actions Workflow

### File: `.github/workflows/ci.yml`

The workflow runs automatically on:
- **Push** to `main` or `develop` branches
- **Pull Requests** targeting `main` or `develop`

### Workflow Jobs

#### 1. **Test Job** (Required)
- ✅ Code formatting check (`cargo fmt`)
- ✅ Linting with clippy (`cargo clippy`)
- ✅ Build project (`cargo build`)
- ✅ Unit tests (`cargo test --lib`)
- ✅ Integration tests (`cargo test --test integration_test -- --ignored`)
- 🐘 PostgreSQL service container for database tests

#### 2. **Coverage Job** (Optional)
- 📊 Code coverage with `cargo-tarpaulin`
- ☁️ Upload to Codecov (if configured)

### Pipeline Stages

```
┌─────────────────┐
│   Push/PR       │
└────────┬────────┘
         │
    ┌────▼────┐
    │ Checkout│
    └────┬────┘
         │
    ┌────▼─────────┐
    │ Setup Rust   │
    │ + Toolchain  │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ PostgreSQL   │
    │ Service      │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ Migrations   │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ Format Check │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ Clippy Lint  │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ Build        │
    └────┬─────────┘
         │
    ┌────▼─────────┐
    │ Unit Tests   │
    └────┬─────────┘
         │
    ┌────▼─────────────┐
    │ Integration Tests│
    └────┬─────────────┘
         │
    ┌────▼─────────┐
    │ ✅ Success   │
    └──────────────┘
```

---

## 🧪 Running Tests Locally

### Prerequisites

```bash
# Install Rust toolchain
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install sqlx-cli
cargo install sqlx-cli --no-default-features --features postgres

# Ensure PostgreSQL is running
brew services start postgresql@14  # macOS
```

### Setup Test Database

```bash
./setup-test-db.sh
```

### Run All Tests

```bash
# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_test -- --ignored --nocapture --test-threads=1

# All tests (unit + integration)
cargo test && cargo test --test integration_test -- --ignored --test-threads=1
```

### Run Specific Test

```bash
# Test all event types
cargo test --test integration_test test_all_event_types_storage_and_retrieval -- --ignored --nocapture

# Test specific chain
cargo test --test integration_test test_polygon_amoy_event_processing -- --ignored --nocapture
```

### Code Quality Checks

```bash
# Check formatting
cargo fmt -- --check

# Run linting
cargo clippy -- -D warnings

# Generate coverage report
cargo tarpaulin --verbose --all-features --workspace --timeout 300 --out Html
# Open tarpaulin-report.html in browser
```

---

## 📝 Test Database Configuration

**Database**: `api_8004_dev_test`
**User**: `matteoscurati` (or your system user on macOS)
**Host**: `localhost:5432`

For GitHub Actions:
- **User**: `postgres`
- **Password**: `postgres`
- **Database**: `api_8004_dev_test`

The integration tests automatically:
1. Ensure each chain exists in the `chains` table
2. Clean up existing test data before each test
3. Run sequentially (`--test-threads=1`) to avoid conflicts

---

## 🔧 Integration Test Helpers

All event types have dedicated helper functions in `tests/integration_test.rs`:

```rust
// IdentityRegistry events
create_registered_event(chain_id, block_number, agent_id)
create_metadata_set_event(chain_id, block_number, agent_id)
create_uri_updated_event(chain_id, block_number, agent_id)

// ReputationRegistry events
create_new_feedback_event(chain_id, block_number, agent_id)
create_feedback_revoked_event(chain_id, block_number, agent_id)
create_response_appended_event(chain_id, block_number, agent_id)

// ValidationRegistry events
create_validation_request_event(chain_id, block_number, agent_id)
create_validation_response_event(chain_id, block_number, agent_id)
```

---

## 🎯 What Gets Tested

### Storage & Retrieval
- ✅ Events are stored correctly in PostgreSQL
- ✅ Events can be queried by chain_id
- ✅ Events can be queried by agent_id
- ✅ All event data fields are preserved

### Multi-Chain Support
- ✅ Events from different chains are isolated
- ✅ Each chain has independent sync state
- ✅ Querying one chain doesn't return events from another

### Crash Recovery
- ✅ System resumes from `last_synced_block - 1`
- ✅ No events are missed during recovery
- ✅ Sync state is persisted correctly

### Event Data Validation
- ✅ All 8 event types can be stored and retrieved
- ✅ EventData enum variants serialize/deserialize correctly
- ✅ Required fields are present for each event type

---

## 🚨 CI/CD Failure Scenarios

The pipeline will fail if:

1. **Code formatting issues**
   ```bash
   # Fix with:
   cargo fmt
   ```

2. **Clippy warnings**
   ```bash
   # Fix with:
   cargo clippy --fix
   ```

3. **Build errors**
   - Check Rust compiler errors
   - Ensure dependencies are up to date

4. **Test failures**
   - Check test output for specific failure
   - Run locally to debug: `cargo test --test integration_test -- --ignored --nocapture`

5. **Database migrations fail**
   - Verify migrations in `./migrations/` are valid SQL
   - Check PostgreSQL service is running

---

## 📚 Documentation

- **Integration Tests**: `tests/integration_test.rs`
- **Event Models**: `src/models/events.rs`
- **Storage Layer**: `src/storage/mod.rs`
- **Migrations**: `migrations/`
- **Test Setup Script**: `setup-test-db.sh`

---

## ✅ Pre-Deploy Checklist

Before deploying to production:

- [ ] All CI/CD checks pass (GitHub Actions green ✅)
- [ ] Integration tests pass locally
- [ ] No clippy warnings
- [ ] Code is formatted (`cargo fmt`)
- [ ] Database migrations tested
- [ ] RPC endpoints tested (`./test-rpc-connectivity.sh`)
- [ ] Environment variables configured
- [ ] Secrets configured in deployment environment

---

**Last Updated**: 2025-01-10
**Maintainer**: Matteo Scurati
**Test Count**: 8 integration tests + unit tests
**Coverage**: 100% event types, 100% chains
