use crate::models::{Event, EventQuery, EventType};
use anyhow::Result;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use sqlx::{PgPool, Row};
use std::sync::Arc;

/// Hybrid storage with in-memory cache and PostgreSQL persistence
#[derive(Clone)]
pub struct Storage {
    pool: PgPool,
    cache: Arc<DashMap<String, Event>>, // key: tx_hash:log_index
    max_cache_size: usize,
}

impl Storage {
    pub fn new(pool: PgPool, max_cache_size: usize) -> Self {
        Self {
            pool,
            cache: Arc::new(DashMap::new()),
            max_cache_size,
        }
    }

    /// Store a new event in both cache and database
    pub async fn store_event(&self, event: Event) -> Result<()> {
        // Generate cache key
        let cache_key = format!("{}:{}", event.transaction_hash, event.log_index);

        // Check if event already exists in cache
        if self.cache.contains_key(&cache_key) {
            return Ok(());
        }

        // Store in database
        let event_data_json = serde_json::to_value(&event.event_data)?;

        sqlx::query(
            r#"
            INSERT INTO events (
                chain_id, block_number, block_timestamp, transaction_hash, log_index,
                contract_address, event_type, event_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (chain_id, transaction_hash, log_index) DO NOTHING
            "#,
        )
        .bind(event.chain_id as i64)
        .bind(event.block_number as i64)
        .bind(event.block_timestamp)
        .bind(&event.transaction_hash)
        .bind(event.log_index as i32)
        .bind(&event.contract_address)
        .bind(event.event_type.as_str())
        .bind(event_data_json)
        .execute(&self.pool)
        .await?;

        // Store in cache (evict oldest if needed)
        if self.cache.len() >= self.max_cache_size {
            // Simple eviction: remove first entry
            if let Some(first_key) = self.cache.iter().next().map(|e| e.key().clone()) {
                self.cache.remove(&first_key);
            }
        }

        self.cache.insert(cache_key, event);

        Ok(())
    }

    /// Get recent events based on query parameters
    pub async fn get_recent_events(&self, query: EventQuery) -> Result<Vec<Event>> {
        // Start building the query
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT
                id, chain_id, block_number, block_timestamp, transaction_hash, log_index,
                contract_address, event_type, event_data, created_at
            FROM events
            WHERE 1=1
            "#,
        );

        // Filter by chain_id (REQUIRED for multi-chain support)
        qb.push(" AND chain_id = ");
        qb.push_bind(query.chain_id as i64);

        // Calculate cutoff block if needed
        if let Some(hours) = query.hours {
            let cutoff = Utc::now() - Duration::hours(hours as i64);
            qb.push(" AND block_timestamp >= ");
            qb.push_bind(cutoff);
        } else if let Some(blocks) = query.blocks {
            let current_block: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(block_number), 0) FROM events")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            let cutoff = current_block.saturating_sub(blocks as i64);
            qb.push(" AND block_number >= ");
            qb.push_bind(cutoff);
        }

        // Filter by contract
        if let Some(contract) = &query.contract {
            qb.push(" AND contract_address = ");
            qb.push_bind(contract.to_lowercase());
        }

        // Filter by event type
        if let Some(event_type) = &query.event_type {
            qb.push(" AND event_type = ");
            qb.push_bind(event_type);
        }

        // Filter by category (maps to multiple event types)
        if let Some(event_types) = query.event_types_for_category() {
            if !event_types.is_empty() {
                qb.push(" AND event_type IN (");
                let mut separated = qb.separated(", ");
                for event_type in event_types {
                    separated.push_bind(event_type);
                }
                separated.push_unseparated(")");
            }
        }

        // Filter by agent_id (searches within JSONB event_data)
        if let Some(agent_id) = &query.agent_id {
            qb.push(" AND event_data->>'agent_id' = ");
            qb.push_bind(agent_id);
        }

        // Add ordering
        qb.push(" ORDER BY block_number DESC, log_index DESC");

        // Add limit and offset for pagination
        if let Some(limit) = query.limit {
            qb.push(" LIMIT ");
            qb.push_bind(limit);
        }

        if let Some(offset) = query.offset {
            qb.push(" OFFSET ");
            qb.push_bind(offset);
        }

        // Execute query with proper parameter binding
        let rows = qb.build().fetch_all(&self.pool).await?;

        // Parse results
        let events: Vec<Event> = rows
            .into_iter()
            .filter_map(|row| {
                let event_type_str: String = row.get("event_type");
                let event_type = match event_type_str.as_str() {
                    "Registered" => EventType::Registered,
                    "MetadataSet" => EventType::MetadataSet,
                    "UriUpdated" => EventType::UriUpdated,
                    "NewFeedback" => EventType::NewFeedback,
                    "FeedbackRevoked" => EventType::FeedbackRevoked,
                    "ResponseAppended" => EventType::ResponseAppended,
                    "ValidationRequest" => EventType::ValidationRequest,
                    "ValidationResponse" => EventType::ValidationResponse,
                    _ => return None,
                };

                let event_data_json: serde_json::Value = row.get("event_data");
                let event_data = serde_json::from_value(event_data_json).ok()?;

                Some(Event {
                    id: Some(row.get("id")),
                    chain_id: row.get::<i64, _>("chain_id") as u64,
                    block_number: row.get::<i64, _>("block_number") as u64,
                    block_timestamp: row.get("block_timestamp"),
                    transaction_hash: row.get("transaction_hash"),
                    log_index: row.get::<i32, _>("log_index") as u32,
                    contract_address: row.get("contract_address"),
                    event_type,
                    event_data,
                    created_at: Some(row.get("created_at")),
                })
            })
            .collect();

        Ok(events)
    }

    /// Count total events matching query (for pagination metadata)
    pub async fn count_events(&self, query: EventQuery) -> Result<i64> {
        // Build the count query with same filters as get_recent_events
        let mut qb = sqlx::QueryBuilder::new(
            r#"
            SELECT COUNT(*) as total
            FROM events
            WHERE 1=1
            "#,
        );

        // Filter by chain_id (REQUIRED for multi-chain support)
        qb.push(" AND chain_id = ");
        qb.push_bind(query.chain_id as i64);

        // Calculate cutoff block if needed
        if let Some(hours) = query.hours {
            let cutoff = Utc::now() - Duration::hours(hours as i64);
            qb.push(" AND block_timestamp >= ");
            qb.push_bind(cutoff);
        } else if let Some(blocks) = query.blocks {
            let current_block: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(block_number), 0) FROM events")
                .fetch_one(&self.pool)
                .await
                .unwrap_or(0);
            let cutoff = current_block.saturating_sub(blocks as i64);
            qb.push(" AND block_number >= ");
            qb.push_bind(cutoff);
        }

        // Filter by contract
        if let Some(contract) = &query.contract {
            qb.push(" AND contract_address = ");
            qb.push_bind(contract.to_lowercase());
        }

        // Filter by event type
        if let Some(event_type) = &query.event_type {
            qb.push(" AND event_type = ");
            qb.push_bind(event_type);
        }

        // Filter by category (maps to multiple event types)
        if let Some(event_types) = query.event_types_for_category() {
            if !event_types.is_empty() {
                qb.push(" AND event_type IN (");
                let mut separated = qb.separated(", ");
                for event_type in event_types {
                    separated.push_bind(event_type);
                }
                separated.push_unseparated(")");
            }
        }

        // Filter by agent_id
        if let Some(agent_id) = &query.agent_id {
            qb.push(" AND event_data->>'agent_id' = ");
            qb.push_bind(agent_id);
        }

        // Execute count query
        let row = qb.build().fetch_one(&self.pool).await?;
        let total: i64 = row.get("total");

        Ok(total)
    }

    /// Update the last synced block
    pub async fn update_last_synced_block(&self, block_number: u64) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE indexer_state
            SET last_synced_block = $1, last_synced_at = NOW()
            WHERE id = 1
            "#,
        )
        .bind(block_number as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the last synced block number
    pub async fn get_last_synced_block(&self) -> Result<u64> {
        let block: i64 =
            sqlx::query_scalar("SELECT last_synced_block FROM indexer_state WHERE id = 1")
                .fetch_one(&self.pool)
                .await?;

        Ok(block as u64)
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_cache_size)
    }

    /// Get event statistics by category
    /// Returns counts for all categories: all, agents, metadata, validation, feedback
    pub async fn get_category_stats(&self, chain_id: u64) -> Result<CategoryStats> {
        // Count all events for this chain
        let all_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1"
        )
        .bind(chain_id as i64)
        .fetch_one(&self.pool)
        .await?;

        // Count agents events (Registered)
        let agents_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1 AND event_type = 'Registered'"
        )
        .bind(chain_id as i64)
        .fetch_one(&self.pool)
        .await?;

        // Count metadata events (MetadataSet, UriUpdated)
        let metadata_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1 AND event_type IN ('MetadataSet', 'UriUpdated')"
        )
        .bind(chain_id as i64)
        .fetch_one(&self.pool)
        .await?;

        // Count validation events (ValidationRequest, ValidationResponse)
        let validation_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1 AND event_type IN ('ValidationRequest', 'ValidationResponse')"
        )
        .bind(chain_id as i64)
        .fetch_one(&self.pool)
        .await?;

        // Count feedback events (NewFeedback, FeedbackRevoked, ResponseAppended)
        let feedback_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM events WHERE chain_id = $1 AND event_type IN ('NewFeedback', 'FeedbackRevoked', 'ResponseAppended')"
        )
        .bind(chain_id as i64)
        .fetch_one(&self.pool)
        .await?;

        Ok(CategoryStats {
            all: all_count,
            agents: agents_count,
            capabilities: 0, // Not implemented yet
            metadata: metadata_count,
            validation: validation_count,
            feedback: feedback_count,
            payments: 0, // Not implemented yet
        })
    }
}

/// Statistics for event categories
#[derive(Debug, Clone, serde::Serialize)]
pub struct CategoryStats {
    pub all: i64,
    pub agents: i64,
    pub capabilities: i64,
    pub metadata: i64,
    pub validation: i64,
    pub feedback: i64,
    pub payments: i64,
}

#[cfg(test)]
mod tests {
    use crate::models::*;
    use chrono::Utc;

    fn create_test_event(chain_id: u64, agent_id: &str, block_number: u64, tx_hash: &str, log_index: u32) -> Event {
        Event {
            id: None,
            chain_id,
            block_number,
            block_timestamp: Utc::now(),
            transaction_hash: tx_hash.to_string(),
            log_index,
            contract_address: "0x1234".to_string(),
            event_type: EventType::Registered,
            event_data: EventData::Registered(RegisteredData {
                agent_id: agent_id.to_string(),
                token_uri: "https://example.com".to_string(),
                owner: "0x5678".to_string(),
            }),
            created_at: None,
        }
    }

    #[test]
    fn test_cache_key_format() {
        let event = create_test_event(11155111, "1", 100, "0xabc", 0);
        let cache_key = format!("{}:{}", event.transaction_hash, event.log_index);
        assert_eq!(cache_key, "0xabc:0");
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let event1 = create_test_event(11155111, "1", 100, "0xabc", 0);
        let event2 = create_test_event(11155111, "1", 100, "0xabc", 1);
        let event3 = create_test_event(11155111, "1", 100, "0xdef", 0);

        let key1 = format!("{}:{}", event1.transaction_hash, event1.log_index);
        let key2 = format!("{}:{}", event2.transaction_hash, event2.log_index);
        let key3 = format!("{}:{}", event3.transaction_hash, event3.log_index);

        assert_ne!(key1, key2); // Same tx, different log_index
        assert_ne!(key1, key3); // Different tx, same log_index
    }

    #[test]
    fn test_event_query_clone() {
        let query = EventQuery {
            chain_id: 11155111,
            blocks: Some(100),
            hours: None,
            contract: Some("0x1234".to_string()),
            event_type: Some("Registered".to_string()),
            agent_id: Some("42".to_string()),
            category: None,
            offset: Some(10),
            limit: Some(50),
        };

        let cloned = query.clone();
        assert_eq!(cloned.chain_id, 11155111);
        assert_eq!(cloned.agent_id, Some("42".to_string()));
        assert_eq!(cloned.category, None);
        assert_eq!(cloned.offset, Some(10));
        assert_eq!(cloned.limit, Some(50));
    }

    #[test]
    fn test_event_with_chain_id() {
        let event_sepolia = create_test_event(11155111, "1", 100, "0xabc", 0);
        let event_mainnet = create_test_event(1, "1", 100, "0xabc", 0);

        assert_eq!(event_sepolia.chain_id, 11155111);
        assert_eq!(event_mainnet.chain_id, 1);
        assert_ne!(event_sepolia.chain_id, event_mainnet.chain_id);
    }

    #[test]
    fn test_event_data_agent_id() {
        let event = create_test_event(11155111, "42", 100, "0xabc", 0);

        match &event.event_data {
            EventData::Registered(data) => {
                assert_eq!(data.agent_id, "42");
            }
            _ => panic!("Expected Registered event data"),
        }
    }
}
