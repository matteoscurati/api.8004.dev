#!/bin/bash

# ERC-8004 Indexer Setup Script

set -e

echo "🚀 ERC-8004 Indexer Setup"
echo "=========================="
echo ""

# Check if PostgreSQL is installed
if ! command -v psql &> /dev/null; then
    echo "❌ PostgreSQL is not installed. Please install PostgreSQL first:"
    echo "   macOS: brew install postgresql"
    echo "   Linux: apt-get install postgresql"
    exit 1
fi

echo "✅ PostgreSQL found"

# Check if .env file exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "⚠️  Please edit .env file with your configuration:"
    echo "   - Add your Ethereum RPC URL (Alchemy/Infura)"
    echo "   - Update DATABASE_URL if needed"
    echo ""
    read -p "Press enter to continue after editing .env..."
fi

# Source .env
export $(grep -v '^#' .env | xargs)

echo "🗄️  Setting up database..."

# Extract database name from DATABASE_URL
DB_NAME=$(echo $DATABASE_URL | sed -n 's/.*\/\([^?]*\).*/\1/p')

# Check if database exists
if psql -lqt | cut -d \| -f 1 | grep -qw $DB_NAME; then
    echo "✅ Database '$DB_NAME' exists"
else
    echo "📦 Creating database '$DB_NAME'..."
    createdb $DB_NAME || psql -U postgres -c "CREATE DATABASE $DB_NAME;"
    echo "✅ Database created"
fi

echo "🔨 Building project..."
cargo build --release

echo ""
echo "✅ Setup complete!"
echo ""
echo "To run the indexer:"
echo "  cargo run --release"
echo ""
echo "Or run in development mode with logs:"
echo "  RUST_LOG=debug cargo run"
echo ""
echo "API will be available at:"
echo "  HTTP: http://localhost:8080"
echo "  WebSocket: ws://localhost:8080/ws"
echo ""
echo "Endpoints:"
echo "  GET /health          - Health check"
echo "  GET /events          - Get recent events"
echo "  GET /stats           - Indexer statistics"
echo "  WS  /ws              - Real-time event stream"
