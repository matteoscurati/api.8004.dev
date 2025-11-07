-- Add chain_id column to events table for multi-chain support
ALTER TABLE events ADD COLUMN IF NOT EXISTS chain_id BIGINT NOT NULL DEFAULT 11155111;

-- Create index for chain_id queries
CREATE INDEX IF NOT EXISTS idx_events_chain_id ON events(chain_id);

-- Create composite index for chain_id + agent_id queries
CREATE INDEX IF NOT EXISTS idx_events_chain_agent ON events(chain_id, (event_data->>'agent_id'));

-- Update the unique constraint to include chain_id for multi-chain support
-- First drop the old constraint
ALTER TABLE events DROP CONSTRAINT IF EXISTS events_transaction_hash_log_index_key;

-- Add new unique constraint with chain_id
ALTER TABLE events ADD CONSTRAINT events_chain_tx_log_unique
    UNIQUE(chain_id, transaction_hash, log_index);

-- Comment on column
COMMENT ON COLUMN events.chain_id IS 'Blockchain network chain ID (e.g., 1 for Ethereum mainnet, 11155111 for Sepolia)';
