# RPC Endpoints Configuration

## Overview

This document lists all configured RPC endpoints for the ERC-8004 indexer across multiple blockchain networks. Each chain has multiple endpoint providers (Ankr, Infura, Alchemy, QuickNode) for redundancy.

---

## Testnet Chains (Enabled)

### Ethereum Sepolia (Chain ID: 11155111)

**Primary:** `https://rpc.ankr.com/eth_sepolia/c6f3980a20bb6dd2106c94f98fb6eda24ae828b8adf84f4f658687530c211fda`

**Alternatives:**
- Infura: `https://sepolia.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`
- Alchemy #1: `https://eth-sepolia.g.alchemy.com/v2/5d0eE7OcooxSxb9kqSSzhuHBIMc53_u4`
- QuickNode: `https://ultra-few-glitter.ethereum-sepolia.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`
- Alchemy #2: `https://eth-sepolia.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`

**Contracts:**
- Identity Registry: `0x8004a6090Cd10A7288092483047B097295Fb8847`
- Reputation Registry: `0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E`
- Validation Registry: `0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5`

---

### Base Sepolia (Chain ID: 84532)

**Primary:** `https://ultra-few-glitter.base-sepolia.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`

**Alternatives:**
- Infura: `https://base-sepolia.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`
- Alchemy: `https://base-sepolia.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`
- Ankr: `https://rpc.ankr.com/base_sepolia/c6f3980a20bb6dd2106c94f98fb6eda24ae828b8adf84f4f658687530c211fda`
- Public: `https://sepolia.base.org`

**Contracts:**
- Identity Registry: `0x8004AA63c570c570eBF15376c0dB199918BFe9Fb`
- Reputation Registry: `0x8004bd8daB57f14Ed299135749a5CB5c42d341BF`
- Validation Registry: `0x8004C269D0A5647E51E121FeB226200ECE932d55`

---

### Linea Sepolia (Chain ID: 59141)

**Primary:** `https://linea-sepolia.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`

**Alternatives:**
- QuickNode: `https://ultra-few-glitter.lens-testnet.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`
- Alchemy: `https://linea-sepolia.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`
- Public: `https://rpc.sepolia.linea.build`

**Contracts:**
- Identity Registry: `0x8004aa7C931bCE1233973a0C6A667f73F66282e7`
- Reputation Registry: `0x8004bd8483b99310df121c46ED8858616b2Bba02`
- Validation Registry: `0x8004c44d1EFdd699B2A26e781eF7F77c56A9a4EB`

---

### Polygon Amoy (Chain ID: 80002)

**Primary:** `https://rpc-amoy.polygon.technology`

**Contracts:**
- Identity Registry: `0x8004ad19E14B9e0654f73353e8a0B600D46C2898`
- Reputation Registry: `0x8004B12F4C2B42d00c46479e859C92e39044C930`
- Validation Registry: `0x8004C11C213ff7BaD36489bcBDF947ba5eee289B`

---

### Hedera Testnet (Chain ID: 296)

**Primary:** `https://testnet.hashio.io/api`

**Contracts:**
- Identity Registry: `0x4c74ebd72921d537159ed2053f46c12a7d8e5923`
- Reputation Registry: `0xc565edcba77e3abeade40bfd6cf6bf583b3293e0`
- Validation Registry: `0x18df085d85c586e9241e0cd121ca422f571c2da6`

---

## Mainnet Chains (Disabled - Ready for Production)

### Ethereum Mainnet (Chain ID: 1)

**Primary:** `https://rpc.ankr.com/eth/c6f3980a20bb6dd2106c94f98fb6eda24ae828b8adf84f4f658687530c211fda`

**Alternatives:**
- Alchemy: `https://eth-mainnet.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`
- QuickNode: `https://ultra-few-glitter.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`
- Infura: `https://mainnet.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`

**Status:** Disabled - Waiting for contract deployment addresses

---

### Base Mainnet (Chain ID: 8453)

**Primary:** `https://base-mainnet.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`

**Alternatives:**
- QuickNode: `https://ultra-few-glitter.base-mainnet.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`
- Alchemy: `https://base-mainnet.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`
- Ankr: `https://rpc.ankr.com/base/c6f3980a20bb6dd2106c94f98fb6eda24ae828b8adf84f4f658687530c211fda`
- Public: `https://mainnet.base.org`

**Status:** Disabled - Waiting for contract deployment addresses

---

### Linea Mainnet (Chain ID: 59144)

**Primary:** `https://linea-mainnet.infura.io/v3/fe7200f3a9b14894b3ad27e00b4e9afb`

**Alternatives:**
- QuickNode: `https://ultra-few-glitter.linea-mainnet.quiknode.pro/ae42b0b43feb8e1d20f512d3c718e2b6ff52ed76`
- Alchemy: `https://linea-mainnet.g.alchemy.com/v2/ZlrhTJUl6Mq89yHjlROTh`
- Public: `https://rpc.linea.build`

**Status:** Disabled - Waiting for contract deployment addresses

---

## Provider Summary

### Active Providers

| Provider   | Testnets | Mainnets | Total Endpoints |
|------------|----------|----------|----------------|
| **Ankr**   | 2        | 2        | 4              |
| **Infura** | 3        | 3        | 6              |
| **Alchemy**| 3        | 3        | 6              |
| **QuickNode**| 3      | 3        | 6              |
| **Public** | 4        | 2        | 6              |

---

## How to Switch RPC Endpoints

If an RPC endpoint fails or experiences rate limiting, you can manually switch to an alternative:

1. **Edit `chains.yaml`**:
   ```bash
   nano chains.yaml
   ```

2. **Update the `rpc_url` field** for the affected chain by copying one of the alternative URLs from the comments

3. **Restart the service**:
   ```bash
   # Local development
   cargo run

   # Production (Fly.io)
   flyctl deploy
   ```

---

## RPC Endpoint Health Monitoring

To check if an RPC endpoint is working:

```bash
# Test Ethereum Sepolia
curl -X POST https://rpc.ankr.com/eth_sepolia/YOUR_API_KEY \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'

# Test Base Sepolia
curl -X POST https://base-sepolia.infura.io/v3/YOUR_API_KEY \
  -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","method":"eth_blockNumber","params":[],"id":1}'
```

---

## Rate Limits

Different providers have different rate limits:

| Provider   | Free Tier Requests/Day | Notes                          |
|------------|------------------------|--------------------------------|
| Ankr       | Unlimited (throttled)  | Shared infrastructure          |
| Infura     | 100,000/day            | Per project                    |
| Alchemy    | 300M compute units/mo  | Varies by method               |
| QuickNode  | Varies by plan         | Contact for details            |
| Public     | Variable               | No guarantees, may be slow     |

---

## Automatic Failover (Future Enhancement)

Currently, RPC failover must be done manually. A future enhancement will add automatic failover:

```yaml
# Future feature - not yet implemented
fallback_rpcs:
  - "https://alternative1.com"
  - "https://alternative2.com"
```

This will automatically switch to backup RPCs if the primary fails.

---

## Security Notes

1. **API Keys**: The RPC URLs in `chains.yaml` contain API keys. Keep this file secure!
2. **Git Ignore**: Consider adding `chains.yaml` to `.gitignore` if committing to a public repo
3. **Environment Variables**: For extra security, you can use environment variable substitution (future enhancement)

---

## Support

If you experience RPC issues:

1. Check provider status pages:
   - Ankr: https://status.ankr.com
   - Infura: https://status.infura.io
   - Alchemy: https://status.alchemy.com
   - QuickNode: https://status.quiknode.com

2. Monitor the indexer health endpoint:
   ```bash
   curl https://api-8004-dev.fly.dev/health/detailed
   ```

3. Check logs for RPC-related errors:
   ```bash
   flyctl logs
   ```

---

Last Updated: 2025-01-07
