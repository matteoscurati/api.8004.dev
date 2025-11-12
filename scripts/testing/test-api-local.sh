#!/bin/bash
#
# Script per testare l'API locale
# Prerequisito: l'indexer deve essere già in esecuzione (cargo run)
# Usage: ./test-api-local.sh
#

set -e

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
API_URL="http://localhost:8080"
USERNAME="admin"
PASSWORD="changeme"

echo -e "${BLUE}🧪 Test API Locale - ERC-8004 Indexer${NC}"
echo "========================================"
echo ""

# Step 1: Health check
echo "1️⃣  Health Check..."
response=$(curl -s ${API_URL}/health)
if [[ $response == *"ok"* ]]; then
    echo -e "${GREEN}✅ Health check OK${NC}"
    echo "   Response: $response"
else
    echo -e "${RED}❌ Health check FAILED${NC}"
    echo "   Response: $response"
    echo ""
    echo -e "${YELLOW}⚠️  Assicurati che l'indexer sia in esecuzione:${NC}"
    echo "   export RUST_LOG=info"
    echo "   cargo run"
    exit 1
fi
echo ""

# Step 2: Login
echo "2️⃣  Login e ottieni JWT token..."
login_response=$(curl -s -X POST ${API_URL}/login \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"${USERNAME}\",\"password\":\"${PASSWORD}\"}")

if [[ $login_response == *"token"* ]]; then
    TOKEN=$(echo $login_response | grep -o '"token":"[^"]*' | cut -d'"' -f4)
    echo -e "${GREEN}✅ Login successful${NC}"
    echo "   Token (primi 50 caratteri): ${TOKEN:0:50}..."
else
    echo -e "${RED}❌ Login FAILED${NC}"
    echo "   Response: $login_response"
    exit 1
fi
echo ""

# Step 3: Query database stats
echo "3️⃣  Verifica stato database..."
event_count=$(psql erc8004_indexer -t -c "SELECT COUNT(*) FROM events;" 2>/dev/null || echo "0")
chain_count=$(psql erc8004_indexer -t -c "SELECT COUNT(*) FROM chains WHERE enabled = true;" 2>/dev/null || echo "0")
echo "   Eventi totali nel DB: $(echo $event_count | tr -d ' ')"
echo "   Chain abilitate: $(echo $chain_count | tr -d ' ')"
echo ""

# Step 4: Test API queries for each chain
echo "4️⃣  Test query eventi per chain..."
echo ""

chains=(
    "11155111:Ethereum Sepolia"
    "84532:Base Sepolia"
    "59141:Linea Sepolia"
    "80002:Polygon Amoy"
    "296:Hedera Testnet"
)

for chain_info in "${chains[@]}"; do
    IFS=':' read -r chain_id chain_name <<< "$chain_info"

    echo -e "${BLUE}   Chain: $chain_name (ID: $chain_id)${NC}"

    # Query events
    response=$(curl -s "${API_URL}/events?chain_id=${chain_id}&limit=5" \
        -H "Authorization: Bearer $TOKEN")

    if [[ $response == *"events"* ]]; then
        total=$(echo $response | grep -o '"total":[0-9]*' | grep -o '[0-9]*')
        event_count=$(echo $response | grep -o '"events":\[' | wc -l | tr -d ' ')
        echo -e "   ${GREEN}✅ OK${NC} - Eventi totali: $total"

        # Show sample event types if any
        if [ "$total" -gt 0 ]; then
            event_types=$(echo $response | grep -o '"event_type":"[^"]*"' | cut -d'"' -f4 | head -5 | sort | uniq | tr '\n' ', ' | sed 's/,$//')
            if [ ! -z "$event_types" ]; then
                echo "      Tipi evento: $event_types"
            fi
        fi
    else
        echo -e "   ${RED}❌ FAILED${NC}"
        echo "      Response: ${response:0:100}..."
    fi
    echo ""
done

# Step 5: Test event type filters
echo "5️⃣  Test filtri per tipo evento..."
echo ""

event_types=(
    "Registered"
    "MetadataSet"
    "UriUpdated"
    "NewFeedback"
    "FeedbackRevoked"
    "ResponseAppended"
    "ValidationRequest"
    "ValidationResponse"
)

for event_type in "${event_types[@]}"; do
    response=$(curl -s "${API_URL}/events?chain_id=11155111&event_type=${event_type}&limit=0" \
        -H "Authorization: Bearer $TOKEN")

    if [[ $response == *"total"* ]]; then
        count=$(echo $response | grep -o '"total":[0-9]*' | grep -o '[0-9]*')
        if [ "$count" -gt 0 ]; then
            echo -e "   ${GREEN}✅${NC} $event_type: $count eventi"
        else
            echo -e "   ${YELLOW}⚠️${NC}  $event_type: 0 eventi"
        fi
    fi
done
echo ""

# Step 6: Test category filters
echo "6️⃣  Test filtri per categoria..."
echo ""

categories=("identity" "reputation" "validation" "capabilities" "payments")

for category in "${categories[@]}"; do
    response=$(curl -s "${API_URL}/events?chain_id=11155111&category=${category}&limit=0" \
        -H "Authorization: Bearer $TOKEN")

    if [[ $response == *"total"* ]]; then
        count=$(echo $response | grep -o '"total":[0-9]*' | grep -o '[0-9]*')
        echo "   Category '$category': $count eventi"
    fi
done
echo ""

# Step 7: Test pagination
echo "7️⃣  Test paginazione..."
echo ""

# Get first page
page1=$(curl -s "${API_URL}/events?chain_id=11155111&limit=2&offset=0" \
    -H "Authorization: Bearer $TOKEN")

# Get second page
page2=$(curl -s "${API_URL}/events?chain_id=11155111&limit=2&offset=2" \
    -H "Authorization: Bearer $TOKEN")

if [[ $page1 != $page2 ]]; then
    echo -e "   ${GREEN}✅ Paginazione funziona${NC}"
    echo "      Pagina 1 != Pagina 2"
else
    echo -e "   ${YELLOW}⚠️  Paginazione: meno di 4 eventi disponibili${NC}"
fi
echo ""

# Step 8: Test invalid requests
echo "8️⃣  Test gestione errori..."
echo ""

# Test without token
response=$(curl -s -w "\n%{http_code}" "${API_URL}/events?chain_id=11155111&limit=1")
http_code=$(echo "$response" | tail -n1)
if [ "$http_code" == "401" ]; then
    echo -e "   ${GREEN}✅ Autenticazione richiesta (401)${NC}"
else
    echo -e "   ${YELLOW}⚠️  Expected 401, got $http_code${NC}"
fi

# Test with invalid token
response=$(curl -s -w "\n%{http_code}" "${API_URL}/events?chain_id=11155111&limit=1" \
    -H "Authorization: Bearer invalid_token_here")
http_code=$(echo "$response" | tail -n1)
if [ "$http_code" == "401" ]; then
    echo -e "   ${GREEN}✅ Token invalido rifiutato (401)${NC}"
else
    echo -e "   ${YELLOW}⚠️  Expected 401, got $http_code${NC}"
fi
echo ""

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "${GREEN}✅ Test completati!${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""

# Check if any events are indexed
total_events=$(psql erc8004_indexer -t -c "SELECT COUNT(*) FROM events;" 2>/dev/null | tr -d ' ')
if [ "$total_events" -gt 0 ]; then
    echo -e "${GREEN}🎉 L'indexer sta funzionando correttamente!${NC}"
    echo ""
    echo "📊 Statistiche:"
    echo "   • Eventi totali indicizzati: $total_events"

    # Events per chain
    echo ""
    echo "   • Eventi per chain:"
    psql erc8004_indexer -t -c "
        SELECT
            '     - ' || c.name || ': ' || COALESCE(COUNT(e.id), 0) || ' eventi'
        FROM chains c
        LEFT JOIN events e ON c.chain_id = e.chain_id
        WHERE c.enabled = true
        GROUP BY c.name
        ORDER BY COUNT(e.id) DESC;
    " 2>/dev/null

else
    echo -e "${YELLOW}⚠️  Nessun evento ancora indicizzato${NC}"
    echo ""
    echo "Possibili motivi:"
    echo "  1. L'indexer è appena partito (aspetta qualche minuto)"
    echo "  2. starting_block impostato su 'latest' (nessun evento storico)"
    echo "  3. Problemi con gli RPC providers"
    echo ""
    echo "Controlla i log dell'indexer per dettagli"
fi

echo ""
echo "📝 Per query personalizzate, usa il token:"
echo "   export TOKEN=\"$TOKEN\""
echo ""
echo "   # Esempio: query eventi per agent"
echo "   curl \"${API_URL}/events?chain_id=11155111&agent_id=0x...\" \\"
echo "     -H \"Authorization: Bearer \$TOKEN\""
echo ""
