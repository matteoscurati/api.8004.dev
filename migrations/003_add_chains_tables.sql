-- Create chains table for managing supported blockchain networks
CREATE TABLE IF NOT EXISTS chains (
    chain_id BIGINT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    rpc_url TEXT NOT NULL,
    enabled BOOLEAN DEFAULT true NOT NULL,
    identity_registry TEXT NOT NULL,
    reputation_registry TEXT NOT NULL,
    validation_registry TEXT NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create chain_sync_state table for tracking indexer progress per chain
CREATE TABLE IF NOT EXISTS chain_sync_state (
    chain_id BIGINT PRIMARY KEY REFERENCES chains(chain_id) ON DELETE CASCADE,
    last_synced_block BIGINT NOT NULL DEFAULT 0,
    last_sync_time TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    status TEXT DEFAULT 'active' NOT NULL CHECK (status IN ('active', 'syncing', 'catching_up', 'stalled', 'failed')),
    error_message TEXT,
    total_events_indexed BIGINT DEFAULT 0 NOT NULL,
    errors_last_hour INTEGER DEFAULT 0 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Create index for enabled chains (for quick lookup of active chains)
CREATE INDEX IF NOT EXISTS idx_chains_enabled ON chains(enabled) WHERE enabled = true;

-- Create index for chain status queries
CREATE INDEX IF NOT EXISTS idx_chain_sync_status ON chain_sync_state(status);

-- Add comments for documentation
COMMENT ON TABLE chains IS 'Stores configuration for all supported blockchain networks';
COMMENT ON TABLE chain_sync_state IS 'Tracks indexing progress and health status for each chain';
COMMENT ON COLUMN chains.chain_id IS 'Blockchain network chain ID (e.g., 1 for Ethereum mainnet, 11155111 for Sepolia)';
COMMENT ON COLUMN chains.enabled IS 'Whether indexing is enabled for this chain';
COMMENT ON COLUMN chain_sync_state.status IS 'Current status: active (up-to-date), syncing (normal), catching_up (behind), stalled (not progressing), failed (error)';
COMMENT ON COLUMN chain_sync_state.last_synced_block IS 'Last block number successfully indexed for this chain';
COMMENT ON COLUMN chain_sync_state.errors_last_hour IS 'Number of errors in the past hour (for alerting)';

-- Insert initial chains from chains.yaml
-- Ethereum Sepolia
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    11155111,
    'Ethereum Sepolia',
    'https://rpc.ankr.com/eth_sepolia',
    true,
    '0x8004a6090Cd10A7288092483047B097295Fb8847',
    '0x8004B8FD1A363aa02fDC07635C0c5F94f6Af5B7E',
    '0x8004CB39f29c09145F24Ad9dDe2A108C1A2cdfC5'
) ON CONFLICT (chain_id) DO NOTHING;

-- Base Sepolia
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    84532,
    'Base Sepolia',
    'https://sepolia.base.org',
    true,
    '0x8004AA63c570c570eBF15376c0dB199918BFe9Fb',
    '0x8004bd8daB57f14Ed299135749a5CB5c42d341BF',
    '0x8004C269D0A5647E51E121FeB226200ECE932d55'
) ON CONFLICT (chain_id) DO NOTHING;

-- Linea Sepolia
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    59141,
    'Linea Sepolia',
    'https://rpc.sepolia.linea.build',
    true,
    '0x8004aa7C931bCE1233973a0C6A667f73F66282e7',
    '0x8004bd8483b99310df121c46ED8858616b2Bba02',
    '0x8004c44d1EFdd699B2A26e781eF7F77c56A9a4EB'
) ON CONFLICT (chain_id) DO NOTHING;

-- Polygon Amoy
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    80002,
    'Polygon Amoy',
    'https://rpc-amoy.polygon.technology',
    true,
    '0x8004ad19E14B9e0654f73353e8a0B600D46C2898',
    '0x8004B12F4C2B42d00c46479e859C92e39044C930',
    '0x8004C11C213ff7BaD36489bcBDF947ba5eee289B'
) ON CONFLICT (chain_id) DO NOTHING;

-- Hedera Testnet
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    296,
    'Hedera Testnet',
    'https://testnet.hashio.io/api',
    true,
    '0x4c74ebd72921d537159ed2053f46c12a7d8e5923',
    '0xc565edcba77e3abeade40bfd6cf6bf583b3293e0',
    '0x18df085d85c586e9241e0cd121ca422f571c2da6'
) ON CONFLICT (chain_id) DO NOTHING;

-- HyperEVM Testnet (disabled until RPC confirmed)
INSERT INTO chains (chain_id, name, rpc_url, enabled, identity_registry, reputation_registry, validation_registry)
VALUES (
    998,
    'HyperEVM Testnet',
    'https://api.hyperliquid-testnet.xyz/evm',
    false,
    '0x8004A9560C0edce880cbD24Ba19646470851C986',
    '0x8004b490779A65D3290a31fD96471122050dF671',
    '0x8004C86198fdB8d8169c0405D510EC86cc7B0551'
) ON CONFLICT (chain_id) DO NOTHING;

-- Initialize sync state for all chains
INSERT INTO chain_sync_state (chain_id, last_synced_block, status)
SELECT chain_id, 0, 'active'
FROM chains
WHERE enabled = true
ON CONFLICT (chain_id) DO NOTHING;
