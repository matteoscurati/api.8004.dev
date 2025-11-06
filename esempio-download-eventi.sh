#!/bin/bash

# Esempio completo: Download eventi ERC-8004
# Questo script mostra tutte le opzioni disponibili

set -e

API_URL="${1:-https://api-8004-dev.fly.dev}"
USERNAME="${2:-admin}"
PASSWORD="${3}"

# Check if password is provided
if [ -z "$PASSWORD" ]; then
    echo "❌ Error: Password required!"
    echo ""
    echo "Usage: $0 [api-url] [username] <password>"
    echo ""
    echo "Example:"
    echo "  $0 https://api-8004-dev.fly.dev admin 'your-password'"
    echo ""
    exit 1
fi

echo "========================================="
echo "📥 Download Eventi ERC-8004"
echo "========================================="
echo ""

# Passo 1: Login
echo "🔐 Passo 1: Login..."
LOGIN_RESPONSE=$(curl -s -X POST "$API_URL/login" \
  -H "Content-Type: application/json" \
  -d "{\"username\":\"$USERNAME\",\"password\":\"$PASSWORD\"}")

TOKEN=$(echo "$LOGIN_RESPONSE" | python3 -c "import sys, json; print(json.load(sys.stdin)['token'])" 2>/dev/null)

if [ -z "$TOKEN" ]; then
    echo "❌ Login fallito!"
    exit 1
fi

echo "✅ Token ottenuto!"
echo ""

# Passo 2: Download con varie opzioni
echo "📥 Passo 2: Download eventi..."
echo ""

# Opzione A: Tutti gli eventi recenti (default)
echo "📦 A) Scarico ultimi eventi (default)..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events" | python3 -m json.tool > eventi_recenti.json
COUNT_A=$(cat eventi_recenti.json | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null || echo "0")
echo "   Salvati $COUNT_A eventi in: eventi_recenti.json"
echo ""

# Opzione B: Ultime 24 ore
echo "📦 B) Scarico eventi ultime 24 ore..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?hours=24&limit=10000" | python3 -m json.tool > eventi_24h.json
COUNT_B=$(cat eventi_24h.json | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null || echo "0")
echo "   Salvati $COUNT_B eventi in: eventi_24h.json"
echo ""

# Opzione C: Solo eventi Registered
echo "📦 C) Scarico solo eventi 'Registered'..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?event_type=Registered&limit=10000" | python3 -m json.tool > eventi_registered.json
COUNT_C=$(cat eventi_registered.json | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null || echo "0")
echo "   Salvati $COUNT_C eventi in: eventi_registered.json"
echo ""

# Opzione D: Per contratto (Identity Registry)
echo "📦 D) Scarico eventi Identity Registry..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?contract=0x8004a6090Cd10A7288092483047B097295Fb8847&limit=10000" | \
  python3 -m json.tool > eventi_identity.json
COUNT_D=$(cat eventi_identity.json | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null || echo "0")
echo "   Salvati $COUNT_D eventi in: eventi_identity.json"
echo ""

# Opzione E: TUTTI gli eventi disponibili
echo "📦 E) Scarico TUTTI gli eventi disponibili..."
curl -s -H "Authorization: Bearer $TOKEN" \
  "$API_URL/events?blocks=999999&limit=50000" | python3 -m json.tool > tutti_eventi.json
COUNT_E=$(cat tutti_eventi.json | python3 -c "import sys, json; print(json.load(sys.stdin)['count'])" 2>/dev/null || echo "0")
echo "   Salvati $COUNT_E eventi in: tutti_eventi.json"
echo ""

# Passo 3: Sommario
echo "========================================="
echo "✅ Download completato!"
echo "========================================="
echo ""
echo "📊 Sommario file creati:"
echo "   eventi_recenti.json     - $COUNT_A eventi (ultimi 100 blocchi)"
echo "   eventi_24h.json         - $COUNT_B eventi (ultime 24 ore)"
echo "   eventi_registered.json  - $COUNT_C eventi (tipo Registered)"
echo "   eventi_identity.json    - $COUNT_D eventi (Identity Registry)"
echo "   tutti_eventi.json       - $COUNT_E eventi (TUTTI disponibili)"
echo ""
echo "💡 Per vedere un file:"
echo "   cat tutti_eventi.json | python3 -m json.tool | less"
echo ""
echo "💡 Per cercare in un file:"
echo "   cat tutti_eventi.json | python3 -m json.tool | grep 'agent_id'"
echo ""
