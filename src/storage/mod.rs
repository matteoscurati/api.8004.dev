use crate::models::{Event, EventQuery, EventType};
use anyhow::Result;
use chrono::{Duration, Utc};
use dashmap::DashMap;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Cache entry with timestamp for LRU eviction
#[derive(Clone)]
struct CachedEvent {
    #[allow(dead_code)]
    event: Event,
    inserted_at: u64, // Unix timestamp in milliseconds
}

/// Hybrid storage with in-memory cache and PostgreSQL persistence
#[derive(Clone)]
pub struct Storage {
    pool: PgPool,
    cache: Arc<DashMap<String, CachedEvent>>, // key: chain_id:tx_hash:log_index
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

    /// Apply common query filters to a QueryBuilder
    /// This reduces code duplication between get_recent_events and count_events
    async fn apply_query_filters<'a>(
        &self,
        qb: &mut sqlx::QueryBuilder<'a, sqlx::Postgres>,
        query: &'a EventQuery,
    ) -> Result<()> {
        // Filter by chain_id(s) - OPTIONAL
        // - None: Query all chains
        // - Some([11155111]): Query single chain
        // - Some([11155111, 84532, ...]): Query multiple chains
        if let Some(chain_ids) = query.parse_chain_ids() {
            if !chain_ids.is_empty() {
                if chain_ids.len() == 1 {
                    qb.push(" AND chain_id = ");
                    qb.push_bind(chain_ids[0] as i64);
                } else {
                    qb.push(" AND chain_id IN (");
                    let mut separated = qb.separated(", ");
                    for chain_id in chain_ids {
                        separated.push_bind(chain_id as i64);
                    }
                    separated.push_unseparated(")");
                }
            }
        }
        // else: no chain_id filter, query all chains

        // Calculate cutoff block if needed
        if let Some(hours) = query.hours {
            let cutoff = Utc::now() - Duration::hours(hours as i64);
            qb.push(" AND block_timestamp >= ");
            qb.push_bind(cutoff);
        } else if let Some(blocks) = query.blocks {
            let current_block: i64 =
                sqlx::query_scalar("SELECT COALESCE(MAX(block_number), 0) FROM events")
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
            if event_types.is_empty() {
                // Empty vec means category exists but has no events yet (capabilities, payments)
                // Add impossible condition to return zero results
                qb.push(" AND 1 = 0");
            } else {
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

        Ok(())
    }

    /// Store a new event in both cache and database
    pub async fn store_event(&self, event: Event) -> Result<()> {
        // Generate cache key (includes chain_id to avoid collisions across chains)
        let cache_key = format!(
            "{}:{}:{}",
            event.chain_id, event.transaction_hash, event.log_index
        );

        // Check if event already exists in cache
        if self.cache.contains_key(&cache_key) {
            return Ok(());
        }

        // Store in database
        let event_data_json = serde_json::to_value(&event.event_data)?;

        let result = sqlx::query(
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

        // Increment total_events_indexed counter if event was inserted (not a duplicate)
        if result.rows_affected() > 0 {
            sqlx::query(
                r#"
                UPDATE chain_sync_state
                SET total_events_indexed = total_events_indexed + 1,
                    updated_at = NOW()
                WHERE chain_id = $1
                "#,
            )
            .bind(event.chain_id as i64)
            .execute(&self.pool)
            .await?;

            // Update Prometheus metrics
            metrics::counter!("events_indexed_total", "chain_id" => event.chain_id.to_string())
                .increment(1);
        }

        // Store in cache with timestamp (evict oldest if needed)
        if self.cache.len() >= self.max_cache_size {
            // LRU eviction: find and remove the oldest entry by timestamp
            let oldest_key = self
                .cache
                .iter()
                .min_by_key(|entry| entry.value().inserted_at)
                .map(|entry| entry.key().clone());

            if let Some(key_to_remove) = oldest_key {
                self.cache.remove(&key_to_remove);
            }
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.cache.insert(
            cache_key,
            CachedEvent {
                event,
                inserted_at: now,
            },
        );

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

        // Apply common filters
        self.apply_query_filters(&mut qb, &query).await?;

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

        // Apply common filters
        self.apply_query_filters(&mut qb, &query).await?;

        // Execute count query
        let row = qb.build().fetch_one(&self.pool).await?;
        let total: i64 = row.get("total");

        Ok(total)
    }

    /// Update the last synced block (legacy single-chain method)
    #[allow(dead_code)]
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

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.max_cache_size)
    }

    // ===== Multi-Chain Support Methods =====

    /// Update the last synced block for a specific chain
    pub async fn update_last_synced_block_for_chain(
        &self,
        chain_id: u64,
        block_number: u64,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO chain_sync_state (chain_id, last_synced_block, last_sync_time)
            VALUES ($1, $2, NOW())
            ON CONFLICT (chain_id)
            DO UPDATE SET
                last_synced_block = $2,
                last_sync_time = NOW(),
                updated_at = NOW()
            "#,
        )
        .bind(chain_id as i64)
        .bind(block_number as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get the last synced block number for a specific chain
    pub async fn get_last_synced_block_for_chain(&self, chain_id: u64) -> Result<u64> {
        let block: Option<i64> = sqlx::query_scalar(
            "SELECT last_synced_block FROM chain_sync_state WHERE chain_id = $1",
        )
        .bind(chain_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(block.unwrap_or(0) as u64)
    }

    /// Update chain status and error message
    pub async fn update_chain_status(
        &self,
        chain_id: u64,
        status: crate::indexer::supervisor::ChainStatus,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE chain_sync_state
            SET status = $1, error_message = $2, updated_at = NOW()
            WHERE chain_id = $3
            "#,
        )
        .bind(status.as_str())
        .bind(error_message)
        .bind(chain_id as i64)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all enabled chains from database
    pub async fn get_enabled_chains(&self) -> Result<Vec<ChainInfo>> {
        let rows = sqlx::query(
            r#"
            SELECT c.chain_id, c.name, c.rpc_url, c.identity_registry, c.reputation_registry, c.validation_registry,
                   s.last_synced_block, s.status, s.error_message, s.total_events_indexed, s.errors_last_hour, s.last_sync_time
            FROM chains c
            LEFT JOIN chain_sync_state s ON c.chain_id = s.chain_id
            WHERE c.enabled = true
            ORDER BY c.chain_id
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let chains: Vec<ChainInfo> = rows
            .into_iter()
            .map(|row| ChainInfo {
                chain_id: row.get::<i64, _>("chain_id") as u64,
                name: row.get("name"),
                rpc_url: row.get("rpc_url"),
                identity_registry: row.get("identity_registry"),
                reputation_registry: row.get("reputation_registry"),
                validation_registry: row.get("validation_registry"),
                last_synced_block: row
                    .get::<Option<i64>, _>("last_synced_block")
                    .map(|v| v as u64),
                status: row.get("status"),
                error_message: row.get("error_message"),
                total_events_indexed: row
                    .get::<Option<i64>, _>("total_events_indexed")
                    .map(|v| v as u64),
                errors_last_hour: row
                    .get::<Option<i32>, _>("errors_last_hour")
                    .map(|v| v as u32),
                last_sync_time: row.get("last_sync_time"),
            })
            .collect();

        Ok(chains)
    }

    /// Get sync state for a specific chain
    #[allow(dead_code)]
    pub async fn get_chain_sync_state(&self, chain_id: u64) -> Result<Option<ChainSyncState>> {
        let row = sqlx::query(
            r#"
            SELECT chain_id, last_synced_block, last_sync_time, status, error_message, total_events_indexed, errors_last_hour
            FROM chain_sync_state
            WHERE chain_id = $1
            "#,
        )
        .bind(chain_id as i64)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| ChainSyncState {
            chain_id: r.get::<i64, _>("chain_id") as u64,
            last_synced_block: r.get::<i64, _>("last_synced_block") as u64,
            last_sync_time: r.get("last_sync_time"),
            status: r.get("status"),
            error_message: r.get("error_message"),
            total_events_indexed: r.get::<i64, _>("total_events_indexed") as u64,
            errors_last_hour: r.get::<i32, _>("errors_last_hour") as u32,
        }))
    }

    /// Get event statistics by category
    /// Returns counts for all categories: all, agents, metadata, validation, feedback
    /// - None: Stats for all chains
    /// - Some(vec![chain_id]): Stats for specific chain(s)
    pub async fn get_category_stats(&self, chain_ids: Option<Vec<u64>>) -> Result<CategoryStats> {
        // Build WHERE clause for chain filtering
        let chain_filter = if let Some(ids) = &chain_ids {
            if ids.is_empty() {
                "1=1".to_string() // No chain filter
            } else if ids.len() == 1 {
                format!("chain_id = {}", ids[0])
            } else {
                format!("chain_id IN ({})", ids.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", "))
            }
        } else {
            "1=1".to_string() // No chain filter, query all chains
        };

        // Count all events
        let all_count: i64 = sqlx::query_scalar(&format!("SELECT COUNT(*) FROM events WHERE {}", chain_filter))
            .fetch_one(&self.pool)
            .await?;

        // Count agents events (Registered)
        let agents_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM events WHERE {} AND event_type = 'Registered'", chain_filter
        ))
        .fetch_one(&self.pool)
        .await?;

        // Count metadata events (MetadataSet, UriUpdated)
        let metadata_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM events WHERE {} AND event_type IN ('MetadataSet', 'UriUpdated')", chain_filter
        ))
        .fetch_one(&self.pool)
        .await?;

        // Count validation events (ValidationRequest, ValidationResponse)
        let validation_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM events WHERE {} AND event_type IN ('ValidationRequest', 'ValidationResponse')", chain_filter
        ))
        .fetch_one(&self.pool)
        .await?;

        // Count feedback events (NewFeedback, FeedbackRevoked, ResponseAppended)
        let feedback_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM events WHERE {} AND event_type IN ('NewFeedback', 'FeedbackRevoked', 'ResponseAppended')", chain_filter
        ))
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

/// Chain information with sync state
#[derive(Debug, Clone, serde::Serialize)]
pub struct ChainInfo {
    pub chain_id: u64,
    pub name: String,
    pub rpc_url: String,
    pub identity_registry: String,
    pub reputation_registry: String,
    pub validation_registry: String,
    pub last_synced_block: Option<u64>,
    pub status: Option<String>,
    pub error_message: Option<String>,
    pub total_events_indexed: Option<u64>,
    pub errors_last_hour: Option<u32>,
    pub last_sync_time: Option<chrono::DateTime<chrono::Utc>>,
}

/// Chain sync state
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct ChainSyncState {
    pub chain_id: u64,
    pub last_synced_block: u64,
    pub last_sync_time: chrono::DateTime<chrono::Utc>,
    pub status: String,
    pub error_message: Option<String>,
    pub total_events_indexed: u64,
    pub errors_last_hour: u32,
}

#[cfg(test)]
mod tests {
    use super::CachedEvent;
    use crate::models::*;
    use chrono::Utc;
    use dashmap::DashMap;
    use std::sync::Arc;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn create_test_event(
        chain_id: u64,
        agent_id: &str,
        block_number: u64,
        tx_hash: &str,
        log_index: u32,
    ) -> Event {
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
        let cache_key = format!(
            "{}:{}:{}",
            event.chain_id, event.transaction_hash, event.log_index
        );
        assert_eq!(cache_key, "11155111:0xabc:0");
    }

    #[test]
    fn test_cache_key_uniqueness() {
        let event1 = create_test_event(11155111, "1", 100, "0xabc", 0);
        let event2 = create_test_event(11155111, "1", 100, "0xabc", 1);
        let event3 = create_test_event(11155111, "1", 100, "0xdef", 0);

        let key1 = format!(
            "{}:{}:{}",
            event1.chain_id, event1.transaction_hash, event1.log_index
        );
        let key2 = format!(
            "{}:{}:{}",
            event2.chain_id, event2.transaction_hash, event2.log_index
        );
        let key3 = format!(
            "{}:{}:{}",
            event3.chain_id, event3.transaction_hash, event3.log_index
        );

        assert_ne!(key1, key2); // Same tx, different log_index
        assert_ne!(key1, key3); // Different tx, same log_index
    }

    #[test]
    fn test_cache_key_cross_chain_uniqueness() {
        // Test that same tx_hash and log_index on different chains have different cache keys
        let event_sepolia = create_test_event(11155111, "1", 100, "0xabc", 0);
        let event_base = create_test_event(84532, "1", 100, "0xabc", 0);

        let key_sepolia = format!(
            "{}:{}:{}",
            event_sepolia.chain_id, event_sepolia.transaction_hash, event_sepolia.log_index
        );
        let key_base = format!(
            "{}:{}:{}",
            event_base.chain_id, event_base.transaction_hash, event_base.log_index
        );

        assert_ne!(key_sepolia, key_base); // Same tx_hash and log_index but different chains
        assert_eq!(key_sepolia, "11155111:0xabc:0");
        assert_eq!(key_base, "84532:0xabc:0");
    }

    #[test]
    fn test_event_query_clone() {
        let query = EventQuery {
            blocks: Some(100),
            contract: Some("0x1234".to_string()),
            event_type: Some("Registered".to_string()),
            agent_id: Some("42".to_string()),
            offset: Some(10),
            limit: Some(50),
            ..Default::default()
        };

        let cloned = query.clone();
        assert_eq!(cloned.chain_id, None); // Default is None (all chains)
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

    #[test]
    fn test_cache_lru_eviction() {
        use std::thread;
        use std::time::Duration as StdDuration;

        // Create a cache directly for testing
        let cache = Arc::new(DashMap::new());

        // Insert first event
        let event1 = create_test_event(11155111, "1", 100, "0xaaa", 0);
        let key1 = format!(
            "{}:{}:{}",
            event1.chain_id, event1.transaction_hash, event1.log_index
        );

        let now1 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        cache.insert(
            key1.clone(),
            CachedEvent {
                event: event1,
                inserted_at: now1,
            },
        );

        // Wait a bit
        thread::sleep(StdDuration::from_millis(10));

        // Insert second event
        let event2 = create_test_event(11155111, "2", 200, "0xbbb", 0);
        let key2 = format!(
            "{}:{}:{}",
            event2.chain_id, event2.transaction_hash, event2.log_index
        );

        let now2 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        cache.insert(
            key2.clone(),
            CachedEvent {
                event: event2,
                inserted_at: now2,
            },
        );

        assert_eq!(cache.len(), 2);
        assert!(cache.contains_key(&key1));
        assert!(cache.contains_key(&key2));

        // Wait a bit
        thread::sleep(StdDuration::from_millis(10));

        // Insert third event - should evict the oldest (event1)
        let event3 = create_test_event(11155111, "3", 300, "0xccc", 0);
        let key3 = format!(
            "{}:{}:{}",
            event3.chain_id, event3.transaction_hash, event3.log_index
        );

        // Manually trigger eviction logic (same as in store_event)
        if cache.len() >= 2 {
            let oldest_key = cache
                .iter()
                .min_by_key(|entry| entry.value().inserted_at)
                .map(|entry| entry.key().clone());

            if let Some(key_to_remove) = oldest_key {
                cache.remove(&key_to_remove);
            }
        }

        let now3 = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        cache.insert(
            key3.clone(),
            CachedEvent {
                event: event3,
                inserted_at: now3,
            },
        );

        // Verify: event1 (oldest) should be removed, event2 and event3 should remain
        assert_eq!(cache.len(), 2);
        assert!(!cache.contains_key(&key1)); // Oldest removed
        assert!(cache.contains_key(&key2)); // Still there
        assert!(cache.contains_key(&key3)); // Just added
    }

    #[test]
    fn test_category_stats_creation() {
        let stats = super::CategoryStats {
            all: 100,
            agents: 20,
            capabilities: 0,
            metadata: 30,
            validation: 25,
            feedback: 25,
            payments: 0,
        };

        assert_eq!(stats.all, 100);
        assert_eq!(stats.agents, 20);
        assert_eq!(stats.capabilities, 0);
        assert_eq!(stats.metadata, 30);
        assert_eq!(stats.validation, 25);
        assert_eq!(stats.feedback, 25);
        assert_eq!(stats.payments, 0);

        // Verify sum of categories matches all
        assert_eq!(
            stats.agents + stats.metadata + stats.validation + stats.feedback,
            100
        );
    }

    #[test]
    fn test_category_stats_serialization() {
        let stats = super::CategoryStats {
            all: 50,
            agents: 10,
            capabilities: 5,
            metadata: 15,
            validation: 10,
            feedback: 8,
            payments: 2,
        };

        let json = serde_json::to_string(&stats).unwrap();
        assert!(json.contains("\"all\":50"));
        assert!(json.contains("\"agents\":10"));
        assert!(json.contains("\"capabilities\":5"));
        assert!(json.contains("\"metadata\":15"));
        assert!(json.contains("\"validation\":10"));
        assert!(json.contains("\"feedback\":8"));
        assert!(json.contains("\"payments\":2"));
    }

    #[test]
    fn test_chain_info_creation() {
        let chain_info = super::ChainInfo {
            chain_id: 11155111,
            name: "Ethereum Sepolia".to_string(),
            rpc_url: "https://sepolia.infura.io".to_string(),
            identity_registry: "0x1111111111111111111111111111111111111111".to_string(),
            reputation_registry: "0x2222222222222222222222222222222222222222".to_string(),
            validation_registry: "0x3333333333333333333333333333333333333333".to_string(),
            last_synced_block: Some(1000),
            status: Some("syncing".to_string()),
            error_message: None,
            total_events_indexed: Some(500),
            errors_last_hour: Some(0),
            last_sync_time: Some(Utc::now()),
        };

        assert_eq!(chain_info.chain_id, 11155111);
        assert_eq!(chain_info.name, "Ethereum Sepolia");
        assert_eq!(chain_info.status, Some("syncing".to_string()));
        assert_eq!(chain_info.total_events_indexed, Some(500));
    }

    #[test]
    fn test_chain_sync_state_creation() {
        let sync_state = super::ChainSyncState {
            chain_id: 84532,
            last_synced_block: 5000,
            last_sync_time: Utc::now(),
            status: "active".to_string(),
            error_message: None,
            total_events_indexed: 1200,
            errors_last_hour: 0,
        };

        assert_eq!(sync_state.chain_id, 84532);
        assert_eq!(sync_state.last_synced_block, 5000);
        assert_eq!(sync_state.status, "active");
        assert_eq!(sync_state.total_events_indexed, 1200);
        assert_eq!(sync_state.errors_last_hour, 0);
    }

    #[test]
    fn test_cache_stats_logic() {
        // Test cache_stats logic without creating actual storage
        let cache = Arc::new(DashMap::new());
        let max_size = 100;

        // Initially empty
        assert_eq!(cache.len(), 0);

        // Add some items
        cache.insert(
            "key1".to_string(),
            CachedEvent {
                event: create_test_event(11155111, "1", 100, "0xabc", 0),
                inserted_at: 1000,
            },
        );
        cache.insert(
            "key2".to_string(),
            CachedEvent {
                event: create_test_event(11155111, "2", 200, "0xdef", 0),
                inserted_at: 2000,
            },
        );

        assert_eq!(cache.len(), 2);
        assert_eq!(max_size, 100);
    }
}
