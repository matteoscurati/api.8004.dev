#!/bin/bash
# Setup test database for integration tests

set -e

echo "=========================================="
echo "Setting up Test Database"
echo "=========================================="
echo ""

DB_NAME="api_8004_dev_test"
DB_USER="$(whoami)"
DB_PASSWORD=""
DB_HOST="localhost"
DB_PORT="5432"

# Check if PostgreSQL is running
if ! pg_isready -h $DB_HOST -p $DB_PORT > /dev/null 2>&1; then
    echo "❌ PostgreSQL is not running on $DB_HOST:$DB_PORT"
    echo ""
    echo "Please start PostgreSQL first:"
    echo "  brew services start postgresql@14  (macOS with Homebrew)"
    echo "  or"
    echo "  docker run -d --name postgres-test -e POSTGRES_PASSWORD=$DB_PASSWORD -p 5432:5432 postgres:14"
    exit 1
fi

echo "✅ PostgreSQL is running"
echo ""

# Drop existing test database if it exists
echo "Dropping existing test database (if any)..."
psql -h $DB_HOST -p $DB_PORT -d postgres -c "DROP DATABASE IF EXISTS $DB_NAME;" 2>/dev/null || true

# Create test database
echo "Creating test database: $DB_NAME..."
psql -h $DB_HOST -p $DB_PORT -d postgres -c "CREATE DATABASE $DB_NAME;"

echo "✅ Test database created"
echo ""

# Run migrations
echo "Running migrations..."
export DATABASE_URL="postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"

if command -v sqlx &> /dev/null; then
    sqlx migrate run
    echo "✅ Migrations completed using sqlx"
else
    echo "⚠️  sqlx-cli not found, migrations will run during tests"
fi

echo ""
echo "=========================================="
echo "Test Database Setup Complete!"
echo "=========================================="
echo ""
echo "Database URL: postgresql://$DB_USER@$DB_HOST:$DB_PORT/$DB_NAME"
echo ""
echo "Run integration tests with:"
echo "  cargo test --test integration_test -- --ignored --nocapture"
echo ""
