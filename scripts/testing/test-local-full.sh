#!/bin/bash
#
# Script completo per testare l'indexer in locale
# Usage: ./test-local-full.sh
#

set -e  # Exit on error

echo "🧪 ERC-8004 Indexer - Test Locale Completo"
echo "=========================================="
echo ""

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Helper functions
success() {
    echo -e "${GREEN}✅ $1${NC}"
}

error() {
    echo -e "${RED}❌ $1${NC}"
}

warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

info() {
    echo -e "ℹ️  $1"
}

# Step 1: Check prerequisites
echo "📋 Step 1/9: Verifica prerequisiti..."
echo ""

# Check PostgreSQL
if pg_isready &> /dev/null; then
    success "PostgreSQL è in esecuzione"
else
    error "PostgreSQL non è in esecuzione"
    echo "   Avvialo con: brew services start postgresql"
    exit 1
fi

# Check database exists
if psql -l | grep -q erc8004_indexer; then
    success "Database 'erc8004_indexer' esiste"
else
    warning "Database 'erc8004_indexer' non esiste"
    read -p "Vuoi crearlo ora? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        createdb erc8004_indexer
        success "Database creato"
    else
        error "Database richiesto per continuare"
        exit 1
    fi
fi

# Check .env file
if [ -f .env ]; then
    success "File .env esiste"
else
    error "File .env non trovato"
    echo "   Copia .env.example: cp .env.example .env"
    exit 1
fi

# Check chains.yaml
if [ -f chains.yaml ]; then
    success "File chains.yaml esiste"
    enabled_chains=$(grep "enabled: true" chains.yaml | wc -l | tr -d ' ')
    info "Chain abilitate: $enabled_chains"
else
    error "File chains.yaml non trovato"
    exit 1
fi

echo ""

# Step 2: Run migrations
echo "🗄️  Step 2/9: Esegui migrazioni database..."
echo ""

if sqlx migrate run; then
    success "Migrazioni completate"
else
    error "Errore durante le migrazioni"
    exit 1
fi

echo ""

# Step 3: Build project
echo "🏗️  Step 3/9: Build del progetto..."
echo ""

if cargo build --quiet 2>&1 | grep -i "error"; then
    error "Errore durante la build"
    exit 1
else
    success "Build completata"
fi

echo ""

# Step 4: Run tests
echo "✅ Step 4/9: Esegui test unitari..."
echo ""

if cargo test --lib --quiet; then
    test_count=$(cargo test --lib --quiet 2>&1 | grep "test result:" | grep -o "[0-9]* passed" | grep -o "[0-9]*")
    success "Test unitari: $test_count passed"
else
    error "Alcuni test unitari sono falliti"
    exit 1
fi

echo ""

# Step 5: Setup test database
echo "🧪 Step 5/9: Setup database di test..."
echo ""

if [ -f setup-test-db.sh ]; then
    if ./setup-test-db.sh &> /dev/null; then
        success "Database di test configurato"
    else
        warning "Errore nel setup database di test (potrebbe già esistere)"
    fi
else
    warning "setup-test-db.sh non trovato, skip"
fi

echo ""

# Step 6: Run integration tests
echo "🔗 Step 6/9: Esegui test di integrazione..."
echo ""

if cargo test --test integration_test -- --ignored --test-threads=1 --quiet; then
    integration_count=$(cargo test --test integration_test -- --ignored --test-threads=1 --quiet 2>&1 | grep "test result:" | grep -o "[0-9]* passed" | grep -o "[0-9]*")
    success "Test di integrazione: $integration_count passed"
else
    error "Alcuni test di integrazione sono falliti"
    exit 1
fi

echo ""

# Step 7: Check database tables
echo "📊 Step 7/9: Verifica tabelle database..."
echo ""

table_count=$(psql erc8004_indexer -t -c "\dt" | grep public | wc -l | tr -d ' ')
if [ "$table_count" -ge 5 ]; then
    success "Tabelle database: $table_count create"
    psql erc8004_indexer -c "\dt" | grep public
else
    error "Tabelle mancanti nel database"
    exit 1
fi

echo ""

# Step 8: Verify configuration
echo "⚙️  Step 8/9: Verifica configurazione..."
echo ""

# Check JWT_SECRET length
jwt_secret=$(grep JWT_SECRET .env | cut -d= -f2)
if [ ${#jwt_secret} -ge 32 ]; then
    success "JWT_SECRET configurato (${#jwt_secret} caratteri)"
else
    warning "JWT_SECRET troppo corto (${#jwt_secret} caratteri, minimo 32)"
fi

# Check AUTH credentials
if grep -q "AUTH_USERNAME" .env && grep -q "AUTH_PASSWORD" .env; then
    success "Credenziali AUTH configurate"
else
    error "AUTH_USERNAME o AUTH_PASSWORD mancanti"
fi

# Check DATABASE_URL
if grep -q "DATABASE_URL" .env; then
    success "DATABASE_URL configurato"
else
    error "DATABASE_URL mancante"
fi

echo ""

# Step 9: Ready to start
echo "🚀 Step 9/9: Pronto per l'avvio!"
echo ""

success "Tutti i controlli completati!"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📝 Prossimi step:"
echo ""
echo "1. Avvia l'indexer:"
echo "   export RUST_LOG=info"
echo "   cargo run"
echo ""
echo "2. In un altro terminale, testa l'API:"
echo "   # Health check"
echo "   curl http://localhost:8080/health"
echo ""
echo "   # Login"
echo "   curl -X POST http://localhost:8080/api/login \\"
echo "     -H 'Content-Type: application/json' \\"
echo "     -d '{\"username\":\"admin\",\"password\":\"changeme\"}'"
echo ""
echo "3. Monitora i log:"
echo "   tail -f <output del cargo run>"
echo ""
echo "4. Verifica indicizzazione:"
echo "   psql erc8004_indexer -c 'SELECT COUNT(*) FROM events;'"
echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📚 Per maggiori dettagli, consulta: LOCAL_TESTING.md"
echo ""
