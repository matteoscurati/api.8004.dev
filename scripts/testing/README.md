# Testing Scripts

Automated testing scripts for API functionality, filters, and edge cases.

## Quick Start

```bash
# Run quick tests
./scripts/testing/test-quick.sh

# Run full local test suite
./scripts/testing/test-local-full.sh
```

## Scripts

### Core API Tests

#### `test-api.sh`
Main API test suite for production deployment.

**Usage:**
```bash
export API_URL="https://api-8004-dev.fly.dev"
export API_USERNAME="admin"
export API_PASSWORD="your-password"
./scripts/testing/test-api.sh
```

#### `test-api-local.sh`
Comprehensive local API testing.

**Usage:**
```bash
export API_URL="http://localhost:8080"
export API_USERNAME="admin"
export API_PASSWORD="changeme"
./scripts/testing/test-api-local.sh
```

#### `test-endpoints.sh`
Test all API endpoints systematically.

### Filter Tests

#### `test-pagination.sh`
Test pagination with offset and limit parameters.

#### `test-category-filter.sh`
Test category filtering (agents, metadata, validation, feedback).

#### `test-agent-filter.sh`
Test filtering by agent_id parameter.

#### `test-chain-agent-filter.sh`
Test combined chain_id and agent_id filtering.

#### `test-empty-categories.sh`
Test categories with no events (capabilities, payments).

### Configuration Tests

#### `test-config.sh`
Test multi-chain configuration loading from chains.yaml.

**Usage:**
```bash
./scripts/testing/test-config.sh
```

#### `test-rpc-connectivity.sh`
Test RPC provider connectivity and failover.

**Usage:**
```bash
./scripts/testing/test-rpc-connectivity.sh
```

### Integrated Tests

#### `test-quick.sh`
Quick smoke test for essential functionality.

**Usage:**
```bash
./scripts/testing/test-quick.sh
```

**Tests:**
- Login/authentication
- Event retrieval
- Basic filtering
- Chain status

#### `test-local.sh`
Local development test suite.

#### `test-local-full.sh`
Complete local test coverage.

**Tests:**
- All API endpoints
- All filter combinations
- Error handling
- Edge cases
- Performance

#### `test-missing-chain.sh`
Test behavior with invalid chain_id.

## Environment Variables

```bash
API_URL         # API endpoint (default: http://localhost:8080)
API_USERNAME    # API username (default: admin)
API_PASSWORD    # API password (default: changeme for local)
```

## Running Tests

### Individual Test

```bash
./scripts/testing/test-pagination.sh
```

### Multiple Tests

```bash
# Run specific tests
for test in test-pagination.sh test-category-filter.sh; do
    echo "Running $test..."
    ./scripts/testing/$test
done
```

### All Tests

```bash
# Production
./scripts/testing/test-api.sh

# Local development
./scripts/testing/test-local-full.sh
```

## Test Output

Tests use color-coded output:
- 🟢 **Green**: Test passed
- 🔴 **Red**: Test failed
- 🟡 **Yellow**: Warning or partial success

## Writing New Tests

Template for new test scripts:

```bash
#!/bin/bash
set -e

# Configuration
API_URL="${API_URL:-http://localhost:8080}"
API_USERNAME="${API_USERNAME:-admin}"
API_PASSWORD="${API_PASSWORD:-changeme}"

echo "🧪 Testing: [Feature Name]"

# Login
TOKEN=$(curl -s -X POST "$API_URL/login" \
    -H "Content-Type: application/json" \
    -d "{\"username\":\"$API_USERNAME\",\"password\":\"$API_PASSWORD\"}" \
    | jq -r '.token')

# Your test logic here
# ...

echo "✅ All tests passed"
```

## See Also

- [Local Testing Guide](../../docs/LOCAL_TESTING.md)
- [API Examples](../../docs/API_EXAMPLES.md)
