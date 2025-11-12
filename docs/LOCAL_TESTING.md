# 🧪 Testing Locale - Guida Completa

Questa guida ti mostra come testare l'intero sistema in locale prima del deploy.

---

## 📋 Pre-requisiti

- [x] PostgreSQL installato e in esecuzione
- [x] Rust e Cargo installati
- [x] File `.env` configurato
- [x] File `chains.yaml` verificato

---

## 🚀 Step 1: Verifica Database

### 1.1 Controlla che PostgreSQL sia attivo

```bash
# Verifica stato PostgreSQL
pg_isready

# Output atteso:
# /tmp:5432 - accepting connections
```

### 1.2 Verifica/Crea il database

```bash
# Verifica se il database esiste
psql -l | grep erc8004_indexer

# Se non esiste, crealo
createdb erc8004_indexer

# Verifica connessione
psql erc8004_indexer -c "SELECT version();"
```

---

## 🔧 Step 2: Configura Environment

### 2.1 Verifica .env

```bash
cat .env
```

Verifica che contenga:
- ✅ `DATABASE_URL=postgresql://USERNAME@localhost:5432/erc8004_indexer`
- ✅ `JWT_SECRET` (almeno 32 caratteri)
- ✅ `AUTH_USERNAME` e `AUTH_PASSWORD`
- ✅ Altri parametri (SERVER_PORT, CORS, etc.)

### 2.2 Verifica chains.yaml

```bash
# Controlla quante chain sono abilitate
grep "enabled: true" chains.yaml | wc -l

# Output atteso: 5 (Ethereum Sepolia, Base Sepolia, Linea Sepolia, Polygon Amoy, Hedera Testnet)
```

---

## 🗄️ Step 3: Esegui Database Migrations

```bash
# Installa sqlx-cli se non l'hai già fatto
cargo install sqlx-cli --no-default-features --features postgres

# Esegui le migrazioni
sqlx migrate run

# Output atteso:
# Applied 1/001_initial_schema.sql (XXms)
# Applied 2/002_add_authentication.sql (XXms)
# Applied 3/003_add_chains_tables.sql (XXms)
# Applied 4/004_add_events_index.sql (XXms)
```

### 3.1 Verifica tabelle create

```bash
psql erc8004_indexer -c "\dt"

# Output atteso:
#  Schema |       Name        | Type  |     Owner
# --------+-------------------+-------+---------------
#  public | _sqlx_migrations  | table | matteoscurati
#  public | chain_sync_state  | table | matteoscurati
#  public | chains            | table | matteoscurati
#  public | events            | table | matteoscurati
#  public | users             | table | matteoscurati
```

---

## 🏗️ Step 4: Build del Progetto

```bash
# Build in modalità debug (più veloce per testing)
cargo build

# Oppure build ottimizzato (più lento ma più performante)
cargo build --release

# Verifica che non ci siano errori o warning
cargo clippy
```

---

## ✅ Step 5: Esegui i Test

### 5.1 Test unitari

```bash
cargo test --lib

# Output atteso:
# running 54 tests
# ......................................................
# test result: ok. 54 passed; 0 failed; 0 ignored
```

### 5.2 Test di integrazione

```bash
# Setup del database di test (prima volta)
./setup-test-db.sh

# Esegui i test di integrazione
cargo test --test integration_test -- --ignored --nocapture --test-threads=1

# Output atteso:
# running 8 tests
# ........
# test result: ok. 8 passed; 0 failed; 0 ignored
```

---

## 🚀 Step 6: Avvia l'Indexer

### 6.1 Avvio in modalità debug (con log dettagliati)

```bash
# Imposta log level a debug
export RUST_LOG=debug

# Avvia l'indexer
cargo run

# Output atteso (primi secondi):
# 2025-01-10T19:00:00.000Z INFO  [api_8004_dev] Starting ERC-8004 Multi-Chain Indexer
# 2025-01-10T19:00:00.100Z INFO  [api_8004_dev] Loaded configuration for 5 chains
# 2025-01-10T19:00:00.200Z INFO  [Ethereum Sepolia] Starting indexer for chain_id 11155111
# 2025-01-10T19:00:00.300Z INFO  [Base Sepolia] Starting indexer for chain_id 84532
# 2025-01-10T19:00:00.400Z INFO  [Linea Sepolia] Starting indexer for chain_id 59141
# 2025-01-10T19:00:00.500Z INFO  [Polygon Amoy] Starting indexer for chain_id 80002
# 2025-01-10T19:00:00.600Z INFO  [Hedera Testnet] Starting indexer for chain_id 296
# 2025-01-10T19:00:01.000Z INFO  [api_8004_dev] Server listening on http://0.0.0.0:8080
```

### 6.2 Avvio in modalità production (log essenziali)

```bash
# Log level info
export RUST_LOG=info

# Avvia versione ottimizzata
cargo run --release
```

---

## 🧪 Step 7: Test dell'API

### 7.1 Verifica health endpoint (senza autenticazione)

```bash
curl http://localhost:8080/health

# Output atteso:
# {"status":"ok"}
```

### 7.2 Login e ottieni JWT token

```bash
# Login con credenziali da .env
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}'

# Output atteso:
# {"token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."}

# Salva il token in una variabile
export TOKEN="eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9..."
```

### 7.3 Query degli eventi

```bash
# Ottieni eventi recenti per Ethereum Sepolia
curl "http://localhost:8080/events?chain_id=11155111&limit=10" \
  -H "Authorization: Bearer $TOKEN"

# Output atteso:
# {"events":[...],"total_count":X}
```

### 7.4 Query per agent_id specifico

```bash
# Sostituisci con un agent_id reale dal tuo sistema
curl "http://localhost:8080/events?chain_id=11155111&agent_id=0x8004000000000000000000000000000000000001" \
  -H "Authorization: Bearer $TOKEN"
```

### 7.5 Filtra per tipo di evento

```bash
# Solo eventi Registered
curl "http://localhost:8080/events?chain_id=11155111&event_type=Registered&limit=10" \
  -H "Authorization: Bearer $TOKEN"

# Solo eventi NewFeedback
curl "http://localhost:8080/events?chain_id=11155111&event_type=NewFeedback&limit=10" \
  -H "Authorization: Bearer $TOKEN"
```

### 7.6 Statistiche aggregate

```bash
# Statistiche per una chain
curl "http://localhost:8080/events?chain_id=11155111&limit=0" \
  -H "Authorization: Bearer $TOKEN" | jq '.total_count'

# Conta per tipo di evento
curl "http://localhost:8080/events?chain_id=11155111&event_type=Registered&limit=0" \
  -H "Authorization: Bearer $TOKEN" | jq '.total_count'
```

---

## 🔍 Step 8: Verifica Indicizzazione

### 8.1 Controlla lo stato delle chain

```bash
# Query diretta al database
psql erc8004_indexer -c "
SELECT
  c.chain_id,
  c.name,
  cs.last_synced_block,
  cs.total_events_indexed,
  cs.status,
  cs.last_sync_time
FROM chains c
LEFT JOIN chain_sync_state cs ON c.chain_id = cs.chain_id
WHERE c.enabled = true
ORDER BY c.chain_id;
"

# Output atteso:
#  chain_id |       name        | last_synced_block | total_events_indexed |  status  |     last_sync_time
# ----------+-------------------+-------------------+----------------------+----------+------------------------
#  11155111 | Ethereum Sepolia  |           9598975 |                  142 | active   | 2025-01-10 19:30:00
#    84532 | Base Sepolia      |           1542389 |                   87 | active   | 2025-01-10 19:30:00
#    59141 | Linea Sepolia     |            892456 |                   45 | active   | 2025-01-10 19:30:00
#    80002 | Polygon Amoy      |           2345678 |                   12 | active   | 2025-01-10 19:30:00
#      296 | Hedera Testnet    |             54321 |                    0 | syncing  | 2025-01-10 19:30:00
```

### 8.2 Verifica eventi nel database

```bash
# Conta totale eventi
psql erc8004_indexer -c "SELECT COUNT(*) as total_events FROM events;"

# Eventi per chain
psql erc8004_indexer -c "
SELECT
  c.name,
  COUNT(e.id) as event_count
FROM chains c
LEFT JOIN events e ON c.chain_id = e.chain_id
WHERE c.enabled = true
GROUP BY c.name
ORDER BY event_count DESC;
"

# Eventi per tipo
psql erc8004_indexer -c "
SELECT
  event_type,
  COUNT(*) as count
FROM events
GROUP BY event_type
ORDER BY count DESC;
"
```

### 8.3 Monitora i log in tempo reale

In un terminale separato:

```bash
# Segui i log dell'indexer
tail -f <output dell'indexer>

# Cerca eventi specifici
# - "New event detected" = nuovo evento trovato
# - "Event stored" = evento salvato nel DB
# - "Block processed" = blocco processato
# - "ERROR" = errori da investigare
```

---

## 🔧 Step 9: Test Avanzati (Opzionali)

### 9.1 Test WebSocket (streaming eventi in tempo reale)

```bash
# Installa websocat se non l'hai già
brew install websocat  # macOS
# oppure: cargo install websocat

# Connettiti al WebSocket endpoint
websocat "ws://localhost:8080/ws/events?chain_id=11155111&token=$TOKEN"

# Lascia aperto e osserva gli eventi in tempo reale mentre l'indexer li processa
```

### 9.2 Test carico API

```bash
# Installa apache bench (di solito già presente su macOS)
ab -V

# Test 100 richieste, 10 concorrenti
ab -n 100 -c 10 \
  -H "Authorization: Bearer $TOKEN" \
  "http://localhost:8080/events?chain_id=11155111&limit=10"

# Verifica response time e throughput
```

### 9.3 Test RPC provider failover

```bash
# Monitora i log mentre l'indexer ruota tra provider
export RUST_LOG=debug
cargo run 2>&1 | grep -i "provider\|rotating\|failover"

# Output atteso:
# [Ethereum Sepolia] Provider 0 request #40 successful (weight: 40/40)
# [Ethereum Sepolia] Rotating from provider 0 (reached weight 40)
# [Ethereum Sepolia] Provider 1 request #1 successful
```

---

## ❌ Troubleshooting Comuni

### Problema 1: "Database connection failed"

```bash
# Verifica che PostgreSQL sia in esecuzione
pg_isready

# Verifica DATABASE_URL in .env
grep DATABASE_URL .env

# Testa connessione manuale
psql $(grep DATABASE_URL .env | cut -d= -f2)
```

### Problema 2: "All RPC providers unavailable"

```bash
# Controlla chains.yaml - verifica che gli RPC URL siano corretti
# Verifica manualmente un endpoint RPC
curl -X POST https://eth-sepolia.g.alchemy.com/v2/5d0eE7OcooxSxb9kqSSzhuHBIMc53_u4 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Output atteso: {"jsonrpc":"2.0","id":1,"result":"0x..."}
```

### Problema 3: "JWT token expired"

```bash
# Richiedi un nuovo token
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}'

# Aggiorna la variabile TOKEN
export TOKEN="nuovo_token_qui"
```

### Problema 4: Nessun evento viene indicizzato

```bash
# Verifica starting_block in chains.yaml
grep -A 2 "starting_block" chains.yaml

# Se è "latest", gli eventi precedenti non verranno indicizzati
# Per indicizzare da un blocco specifico, modifica chains.yaml:
# starting_block: "9420233"  # Blocco con eventi noti

# Poi riavvia l'indexer
```

### Problema 5: Port 8080 già in uso

```bash
# Trova processo sulla porta 8080
lsof -i :8080

# Termina il processo
kill -9 <PID>

# Oppure cambia porta in .env
echo "SERVER_PORT=8081" >> .env
```

---

## ✅ Checklist Pre-Deploy

Prima di deployare in produzione, verifica:

- [ ] Tutti i test passano (unit + integration)
- [ ] L'indexer si avvia senza errori
- [ ] Tutte le 5 chain sono in stato "active" o "syncing"
- [ ] Gli eventi vengono correttamente indicizzati (verifica nel DB)
- [ ] L'API risponde correttamente
- [ ] JWT authentication funziona
- [ ] Rate limiting funziona
- [ ] WebSocket funziona (opzionale)
- [ ] Log non mostrano errori critici
- [ ] Database migrations sono tutte applicate
- [ ] `.env` non contiene valori di default insicuri
- [ ] `chains.yaml` ha RPC provider funzionanti

---

## 📊 Metriche di Successo

L'indicizzatore funziona correttamente se:

1. **Health Check**: ✅ `/health` risponde con `{"status":"ok"}`
2. **Database**: ✅ Tutte le tabelle esistono e sono popolate
3. **Chain Sync**: ✅ Almeno 3 chain su 5 in stato "active"
4. **Eventi**: ✅ Nuovi eventi appaiono nel database ogni minuto
5. **API**: ✅ Query ritornano dati corretti
6. **Performance**: ✅ Response time < 500ms per query semplici
7. **Uptime**: ✅ Nessun crash dopo 10+ minuti di esecuzione

---

## 🎯 Prossimi Step

Una volta verificato tutto in locale:

1. 📝 Commit delle modifiche finali
2. 🚀 Deploy su Fly.io (vedi `DEPLOYMENT.md`)
3. 🔍 Monitora i log in produzione
4. ✅ Verifica che tutto funzioni come in locale

---

**Hai domande?** Controlla:
- `README.md` - Documentazione principale
- `DEPLOYMENT.md` - Guida al deployment
- `CI_CD_SETUP.md` - Setup CI/CD e test
- `MULTICHAIN_IMPLEMENTATION.md` - Architettura multi-chain
