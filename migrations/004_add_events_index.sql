-- Add composite index on events for efficient chain-specific queries
-- This index significantly improves performance for queries like:
-- SELECT * FROM events WHERE chain_id = ? ORDER BY block_number DESC LIMIT N

CREATE INDEX IF NOT EXISTS idx_events_chain_block
ON events(chain_id, block_number DESC);

-- Add index on (chain_id, block_timestamp) for time-based queries
CREATE INDEX IF NOT EXISTS idx_events_chain_timestamp
ON events(chain_id, block_timestamp DESC);

-- Add index for agent_id searches within event_data JSONB
CREATE INDEX IF NOT EXISTS idx_events_agent_id
ON events((event_data->>'agent_id'));
