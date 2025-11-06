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
                block_number, block_timestamp, transaction_hash, log_index,
                contract_address, event_type, event_data
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (transaction_hash, log_index) DO NOTHING
            "#,
        )
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
        // Build the WHERE clauses
        let mut where_clauses = vec!["1=1".to_string()];

        // Calculate cutoff block if needed
        let cutoff_block = if query.hours.is_some() || query.blocks.is_some() {
            if let Some(hours) = query.hours {
                // Get events within time window
                let cutoff = Utc::now() - Duration::hours(hours as i64);
                where_clauses.push(format!("block_timestamp >= '{}'", cutoff.format("%Y-%m-%d %H:%M:%S")));
                None
            } else if let Some(blocks) = query.blocks {
                // Get current block and calculate cutoff
                let current_block: i64 = sqlx::query_scalar("SELECT COALESCE(MAX(block_number), 0) FROM events")
                    .fetch_one(&self.pool)
                    .await
                    .unwrap_or(0);

                let cutoff = current_block.saturating_sub(blocks as i64);
                Some(cutoff)
            } else {
                None
            }
        } else {
            None
        };

        if let Some(cutoff) = cutoff_block {
            where_clauses.push(format!("block_number >= {}", cutoff));
        }

        // Filter by contract
        if let Some(contract) = &query.contract {
            where_clauses.push(format!("contract_address = '{}'", contract.to_lowercase()));
        }

        // Filter by event type
        if let Some(event_type) = &query.event_type {
            where_clauses.push(format!("event_type = '{}'", event_type));
        }

        // Build final SQL
        let where_clause = where_clauses.join(" AND ");
        let limit_clause = query.limit.map(|l| format!(" LIMIT {}", l)).unwrap_or_default();

        let sql = format!(
            r#"
            SELECT
                id, block_number, block_timestamp, transaction_hash, log_index,
                contract_address, event_type, event_data, created_at
            FROM events
            WHERE {}
            ORDER BY block_number DESC, log_index DESC
            {}
            "#,
            where_clause,
            limit_clause
        );

        // Execute query
        let rows = sqlx::query(&sql).fetch_all(&self.pool).await?;

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
}
