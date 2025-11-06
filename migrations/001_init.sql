-- Create events table to store all ERC-8004 events
CREATE TABLE IF NOT EXISTS events (
    id BIGSERIAL PRIMARY KEY,
    block_number BIGINT NOT NULL,
    block_timestamp TIMESTAMPTZ NOT NULL,
    transaction_hash VARCHAR(66) NOT NULL,
    log_index INTEGER NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    event_type VARCHAR(50) NOT NULL,
    event_data JSONB NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW(),

    -- Ensure uniqueness of events
    UNIQUE(transaction_hash, log_index)
);

-- Create indexes for efficient querying
CREATE INDEX idx_events_block_number ON events(block_number DESC);
CREATE INDEX idx_events_contract_address ON events(contract_address);
CREATE INDEX idx_events_event_type ON events(event_type);
CREATE INDEX idx_events_block_timestamp ON events(block_timestamp DESC);
CREATE INDEX idx_events_created_at ON events(created_at DESC);

-- Composite index for common queries
CREATE INDEX idx_events_contract_type ON events(contract_address, event_type);

-- GIN index for JSONB queries
CREATE INDEX idx_events_data ON events USING GIN(event_data);

-- Create indexer_state table to track sync progress
CREATE TABLE IF NOT EXISTS indexer_state (
    id INTEGER PRIMARY KEY DEFAULT 1,
    last_synced_block BIGINT NOT NULL,
    last_synced_at TIMESTAMPTZ NOT NULL,
    CHECK (id = 1) -- Ensure only one row
);

-- Insert initial state
INSERT INTO indexer_state (last_synced_block, last_synced_at)
VALUES (0, NOW())
ON CONFLICT (id) DO NOTHING;
