# ⚡ Quick Test Guide

Guida rapida per testare l'indexer in locale in 5 minuti.

---

## 🚀 Test Automatico Completo

```bash
# Esegui TUTTI i controlli e test
./test-local-full.sh
```

Questo script verifica automaticamente:
- ✅ PostgreSQL in esecuzione
- ✅ Database esiste
- ✅ File di configurazione (.env, chains.yaml)
- ✅ Migrazioni database
- ✅ Build del progetto
- ✅ Test unitari (54 tests)
- ✅ Test di integrazione (8 tests)
- ✅ Configurazione sicurezza

**Durata**: ~2-3 minuti

---

## 🎯 Test Manuale Veloce

### 1. Avvia l'Indexer

```bash
# Terminale 1
export RUST_LOG=info
cargo run
```

**Verifica** che vedi:
```
✅ Starting ERC-8004 Multi-Chain Indexer
✅ Loaded configuration for 5 chains
✅ Server listening on http://0.0.0.0:8080
```

### 2. Testa l'API

```bash
# Terminale 2
./test-api-local.sh
```

Questo script testa:
- ✅ Health check
- ✅ Login e JWT
- ✅ Query eventi per tutte le 5 chain
- ✅ Filtri per tipo evento
- ✅ Filtri per categoria
- ✅ Paginazione
- ✅ Gestione errori

**Durata**: ~30 secondi

---

## 🔍 Test Manuale Singolo Endpoint

### Health Check (no auth)
```bash
curl http://localhost:8080/health
# ✅ {"status":"ok"}
```

### Login
```bash
curl -X POST http://localhost:8080/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"changeme"}'

# Salva il token
export TOKEN="eyJ0eXAi..."
```

### Query Eventi
```bash
# Ethereum Sepolia
curl "http://localhost:8080/events?chain_id=11155111&limit=10" \
  -H "Authorization: Bearer $TOKEN"

# Base Sepolia
curl "http://localhost:8080/events?chain_id=84532&limit=10" \
  -H "Authorization: Bearer $TOKEN"
```

---

## 📊 Verifica Indicizzazione

```bash
# Eventi totali
psql erc8004_indexer -c "SELECT COUNT(*) FROM events;"

# Eventi per chain
psql erc8004_indexer -c "
SELECT c.name, COUNT(e.id) as eventi
FROM chains c
LEFT JOIN events e ON c.chain_id = e.chain_id
WHERE c.enabled = true
GROUP BY c.name;"

# Stato delle chain
psql erc8004_indexer -c "
SELECT
  c.name,
  cs.last_synced_block,
  cs.total_events_indexed,
  cs.status
FROM chains c
LEFT JOIN chain_sync_state cs ON c.chain_id = cs.chain_id
WHERE c.enabled = true;"
```

---

## ❌ Troubleshooting Rapido

### "Database connection failed"
```bash
# Verifica PostgreSQL
pg_isready

# Avvia se necessario
brew services start postgresql

# Crea database se manca
createdb erc8004_indexer
```

### "All RPC providers unavailable"
```bash
# Testa un endpoint RPC manualmente
curl -X POST https://eth-sepolia.g.alchemy.com/v2/5d0eE7OcooxSxb9kqSSzhuHBIMc53_u4 \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

### Port 8080 già in uso
```bash
# Trova e termina il processo
lsof -i :8080
kill -9 <PID>
```

### Nessun evento indicizzato
```bash
# Verifica starting_block in chains.yaml
grep "starting_block" chains.yaml

# Se vedi "latest", l'indexer parte dal blocco attuale
# Per indicizzare eventi storici, imposta un blocco specifico:
# starting_block: "9420233"
```

---

## ✅ Checklist Successo

L'indexer funziona se vedi:

- [x] Health endpoint risponde: `{"status":"ok"}`
- [x] Login restituisce un JWT token
- [x] Almeno 3 chain su 5 in stato "active" o "syncing"
- [x] Nuovi eventi appaiono nel database
- [x] API query restituiscono dati
- [x] Nessun errore critico nei log

---

## 📚 Documentazione Dettagliata

Per maggiori dettagli:
- **LOCAL_TESTING.md** - Guida completa passo-passo
- **README.md** - Documentazione generale
- **DEPLOYMENT.md** - Deploy in produzione
- **MULTICHAIN_IMPLEMENTATION.md** - Architettura multi-chain

---

**Pronto per il deploy?** Segui `DEPLOYMENT.md`
