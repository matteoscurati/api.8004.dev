pub mod supervisor;

use crate::config::IndexerConfig;
use crate::contracts::{IdentityRegistry, ReputationRegistry, ValidationRegistry};
use crate::models::{
    Event, EventData, EventType, FeedbackRevokedData, MetadataSetData, NewFeedbackData,
    RegisteredData, ResponseAppendedData, UriUpdatedData, ValidationRequestData,
    ValidationResponseData,
};
use crate::rpc::ProviderManager;
use crate::stats::StatsTracker;
use crate::storage::Storage;
use alloy::{
    primitives::{Log as PrimitiveLog, LogData},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{BlockTransactionsKind, Filter, Log},
    sol_types::SolEvent,
    transports::http::{Client, Http},
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// Event indexer that fetches events block by block with adaptive polling
pub struct Indexer {
    config: IndexerConfig,
    provider: Arc<RwLock<RootProvider<Http<Client>>>>,
    provider_manager: Arc<ProviderManager>,
    current_rpc_url: Arc<RwLock<String>>,
    storage: Storage,
    event_tx: broadcast::Sender<Event>,
    stats_tracker: StatsTracker,
}

impl Indexer {
    pub async fn new(
        config: IndexerConfig,
        storage: Storage,
        event_tx: broadcast::Sender<Event>,
        stats_tracker: StatsTracker,
    ) -> Result<Self> {
        // Create provider manager
        let provider_manager = Arc::new(ProviderManager::new(
            config.rpc_providers.clone(),
            config.name.clone(),
        )?);

        // Get initial RPC URL
        let initial_url = provider_manager.get_current_provider().await?;

        // Create initial provider
        let url = initial_url.parse().context("Invalid RPC URL")?;
        let provider = ProviderBuilder::new().on_http(url);

        Ok(Self {
            config,
            provider: Arc::new(RwLock::new(provider)),
            provider_manager,
            current_rpc_url: Arc::new(RwLock::new(initial_url)),
            storage,
            event_tx,
            stats_tracker,
        })
    }

    /// Refresh provider if RPC URL has changed (due to rotation or failover)
    async fn refresh_provider_if_needed(&self) -> Result<()> {
        let new_url = self.provider_manager.get_current_provider().await?;
        let current_url = self.current_rpc_url.read().await;

        if *current_url != new_url {
            drop(current_url); // Release read lock

            debug!(
                "[{}] Switching RPC provider to: {}",
                self.config.name, new_url
            );

            // Create new provider
            let url = new_url.parse().context("Invalid RPC URL")?;
            let new_provider = ProviderBuilder::new().on_http(url);

            // Update provider and URL
            let mut provider_lock = self.provider.write().await;
            *provider_lock = new_provider;

            let mut url_lock = self.current_rpc_url.write().await;
            *url_lock = new_url;

            info!("[{}] RPC provider switched successfully", self.config.name);
        }

        Ok(())
    }

    /// Start the indexer loop with adaptive polling
    pub async fn start(&self) -> Result<()> {
        info!("[{}] Starting ERC-8004 event indexer", self.config.name);
        info!("[{}] Chain ID: {}", self.config.name, self.config.chain_id);
        info!("[{}] Monitoring contracts:", self.config.name);
        info!(
            "[{}]   IdentityRegistry: {}",
            self.config.name, self.config.identity_registry
        );
        info!(
            "[{}]   ReputationRegistry: {}",
            self.config.name, self.config.reputation_registry
        );
        info!(
            "[{}]   ValidationRegistry: {}",
            self.config.name, self.config.validation_registry
        );

        // Get starting block (per-chain)
        // IMPORTANT: Resume from last_synced_block - 1 to ensure no events are missed on crash
        let mut current_block = match self
            .storage
            .get_last_synced_block_for_chain(self.config.chain_id)
            .await
        {
            Ok(block) if block > 1 => {
                let resume_block = block.saturating_sub(1);
                info!("[{}] Resuming from block {} (last synced: {}, replaying last block to ensure no missed events)",
                    self.config.name, resume_block, block);
                resume_block
            }
            _ => {
                let block = if self.config.starting_block == 0 {
                    // Refresh provider before first call
                    self.refresh_provider_if_needed().await?;

                    let result = tokio::time::timeout(Duration::from_secs(30), async {
                        let provider = self.provider.read().await;
                        provider.get_block_number().await
                    })
                    .await;

                    match result {
                        Ok(Ok(block_num)) => {
                            self.provider_manager.mark_success().await;
                            block_num
                        }
                        Ok(Err(e)) => {
                            self.provider_manager
                                .mark_error(&format!("get_block_number failed: {}", e))
                                .await;
                            self.refresh_provider_if_needed().await?;
                            return Err(e).context("Failed to get current block number");
                        }
                        Err(_) => {
                            self.provider_manager
                                .mark_error("get_block_number timeout")
                                .await;
                            self.refresh_provider_if_needed().await?;
                            return Err(anyhow::anyhow!("Timeout getting current block number"));
                        }
                    }
                } else {
                    self.config.starting_block
                };
                info!("[{}] Starting from block {}", self.config.name, block);
                block
            }
        };

        let mut poll_interval = self.config.poll_interval;

        loop {
            // Refresh provider if needed (rotation or recovery)
            if let Err(e) = self.refresh_provider_if_needed().await {
                warn!("[{}] Failed to refresh provider: {}", self.config.name, e);
            }

            // Record polling event for stats
            self.stats_tracker.record_poll(self.config.chain_id);

            // Get latest block to calculate lag (with 30s timeout)
            let latest_block = match tokio::time::timeout(Duration::from_secs(30), async {
                let provider = self.provider.read().await;
                provider.get_block_number().await
            })
            .await
            {
                Ok(Ok(block)) => {
                    self.provider_manager.mark_success().await;
                    // Update current block for stats
                    self.stats_tracker.update_current_block(self.config.chain_id, block);
                    block
                }
                Ok(Err(e)) => {
                    error!("[{}] Failed to get latest block: {}", self.config.name, e);
                    self.provider_manager
                        .mark_error(&format!("get_block_number failed: {}", e))
                        .await;
                    self.refresh_provider_if_needed().await.ok(); // Try to recover
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
                Err(_) => {
                    error!("[{}] Timeout getting latest block (>30s)", self.config.name);
                    self.provider_manager
                        .mark_error("get_block_number timeout")
                        .await;
                    self.refresh_provider_if_needed().await.ok(); // Try to recover
                    sleep(Duration::from_secs(5)).await;
                    continue;
                }
            };

            let blocks_behind = latest_block.saturating_sub(current_block);

            // Adaptive polling: adjust speed based on how far behind we are
            if self.config.adaptive_polling {
                poll_interval = self.calculate_adaptive_interval(blocks_behind);
            }

            // Determine sync strategy based on blocks behind
            match blocks_behind {
                0 => {
                    // Caught up - wait for new blocks
                    debug!(
                        "[{}] Caught up at block {}",
                        self.config.name, current_block
                    );
                    sleep(poll_interval).await;
                }
                1..=10 => {
                    // Near real-time - process one by one
                    match self.sync_block(current_block).await {
                        Ok(events_found) => {
                            if events_found > 0 {
                                info!(
                                    "[{}] Block {}: Found {} events",
                                    self.config.name, current_block, events_found
                                );
                            } else {
                                debug!("[{}] Block {}: No events", self.config.name, current_block);
                            }
                            current_block += 1;

                            // Update last synced block
                            if let Err(e) = self
                                .storage
                                .update_last_synced_block_for_chain(
                                    self.config.chain_id,
                                    current_block,
                                )
                                .await
                            {
                                warn!(
                                    "[{}] Failed to update last synced block: {}",
                                    self.config.name, e
                                );
                            }

                            sleep(poll_interval).await;
                        }
                        Err(e) => {
                            error!(
                                "[{}] Error syncing block {}: {}",
                                self.config.name, current_block, e
                            );
                            warn!("[{}] Retrying in 5 seconds...", self.config.name);
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                11..=100 => {
                    // Moderately behind - batch process
                    info!(
                        "[{}] {} blocks behind, catching up with batches",
                        self.config.name, blocks_behind
                    );
                    let batch_end = (current_block + self.config.batch_size).min(latest_block);

                    match self.sync_block_range(current_block, batch_end).await {
                        Ok(total_events) => {
                            info!(
                                "[{}] Synced blocks {}-{}: {} events",
                                self.config.name, current_block, batch_end, total_events
                            );
                            current_block = batch_end + 1;

                            if let Err(e) = self
                                .storage
                                .update_last_synced_block_for_chain(
                                    self.config.chain_id,
                                    current_block,
                                )
                                .await
                            {
                                warn!(
                                    "[{}] Failed to update last synced block: {}",
                                    self.config.name, e
                                );
                            }

                            // Small delay to avoid overwhelming RPC
                            sleep(Duration::from_millis(50)).await;
                        }
                        Err(e) => {
                            error!(
                                "[{}] Error syncing block range {}-{}: {}",
                                self.config.name, current_block, batch_end, e
                            );
                            warn!("[{}] Retrying in 5 seconds...", self.config.name);
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
                _ => {
                    // Very behind - aggressive catch-up
                    warn!(
                        "[{}] {} blocks behind, aggressive catch-up mode",
                        self.config.name, blocks_behind
                    );
                    let batch_end = (current_block + 100).min(latest_block);

                    match self.sync_block_range(current_block, batch_end).await {
                        Ok(total_events) => {
                            info!(
                                "[{}] Synced blocks {}-{}: {} events",
                                self.config.name, current_block, batch_end, total_events
                            );
                            current_block = batch_end + 1;

                            if let Err(e) = self
                                .storage
                                .update_last_synced_block_for_chain(
                                    self.config.chain_id,
                                    current_block,
                                )
                                .await
                            {
                                warn!(
                                    "[{}] Failed to update last synced block: {}",
                                    self.config.name, e
                                );
                            }

                            // No delay - max speed
                        }
                        Err(e) => {
                            error!(
                                "[{}] Error syncing block range {}-{}: {}",
                                self.config.name, current_block, batch_end, e
                            );
                            warn!("[{}] Retrying in 5 seconds...", self.config.name);
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    }

    /// Calculate adaptive polling interval based on how far behind we are
    fn calculate_adaptive_interval(&self, blocks_behind: u64) -> Duration {
        match blocks_behind {
            0 => self.config.poll_interval,            // Normal speed
            1..=10 => self.config.poll_interval / 2,   // 2x faster
            11..=100 => self.config.poll_interval / 5, // 5x faster
            _ => Duration::from_millis(100),           // Max speed
        }
    }

    /// Sync a range of blocks (for catch-up)
    async fn sync_block_range(&self, from: u64, to: u64) -> Result<usize> {
        let mut total_events = 0;

        for block_num in from..=to {
            match self.sync_block(block_num).await {
                Ok(events) => total_events += events,
                Err(e) => {
                    warn!(
                        "[{}] Failed to sync block {} in range: {}",
                        self.config.name, block_num, e
                    );
                    // Continue with next block instead of failing entire range
                }
            }

            // Small delay to avoid RPC rate limits
            sleep(Duration::from_millis(50)).await;
        }

        Ok(total_events)
    }

    /// Sync a single block and return number of events found
    async fn sync_block(&self, block_number: u64) -> Result<usize> {
        // Get block info for timestamp (with 30s timeout)
        let block_result = tokio::time::timeout(Duration::from_secs(30), async {
            let provider = self.provider.read().await;
            provider
                .get_block_by_number(block_number.into(), BlockTransactionsKind::Hashes)
                .await
        })
        .await;

        let block = match block_result {
            Ok(Ok(Some(b))) => {
                self.provider_manager.mark_success().await;
                b
            }
            Ok(Ok(None)) => {
                self.provider_manager
                    .mark_error(&format!("Block {} not found", block_number))
                    .await;
                return Err(anyhow::anyhow!("Block {} not found", block_number));
            }
            Ok(Err(e)) => {
                self.provider_manager
                    .mark_error(&format!("get_block_by_number failed: {}", e))
                    .await;
                return Err(e).context("Failed to fetch block");
            }
            Err(_) => {
                self.provider_manager
                    .mark_error("get_block_by_number timeout")
                    .await;
                return Err(anyhow::anyhow!("Timeout fetching block"));
            }
        };

        let block_timestamp = chrono::DateTime::from_timestamp(block.header.timestamp as i64, 0)
            .unwrap_or_else(chrono::Utc::now);

        // Fetch logs from all three contracts (with 30s timeout)
        let filter = Filter::new()
            .from_block(block_number)
            .to_block(block_number)
            .address(vec![
                self.config.identity_registry,
                self.config.reputation_registry,
                self.config.validation_registry,
            ]);

        let logs_result = tokio::time::timeout(Duration::from_secs(30), async {
            let provider = self.provider.read().await;
            provider.get_logs(&filter).await
        })
        .await;

        let logs = match logs_result {
            Ok(Ok(l)) => {
                self.provider_manager.mark_success().await;
                l
            }
            Ok(Err(e)) => {
                self.provider_manager
                    .mark_error(&format!("get_logs failed: {}", e))
                    .await;
                return Err(e).context("Failed to fetch logs");
            }
            Err(_) => {
                self.provider_manager.mark_error("get_logs timeout").await;
                return Err(anyhow::anyhow!("Timeout fetching logs"));
            }
        };

        // Process each log
        for log in &logs {
            if let Err(e) = self.process_log(log, block_number, block_timestamp).await {
                warn!(
                    "Failed to process log in tx {}: {}",
                    log.transaction_hash.unwrap_or_default(),
                    e
                );
            }
        }

        Ok(logs.len())
    }

    /// Process a single log entry
    async fn process_log(
        &self,
        log: &Log,
        block_number: u64,
        block_timestamp: chrono::DateTime<chrono::Utc>,
    ) -> Result<()> {
        let contract_address = format!("{:?}", log.address());
        let tx_hash = format!("{:?}", log.transaction_hash.unwrap_or_default());
        let log_index = log.log_index.unwrap_or_default() as u32;

        // Determine which contract and decode the event
        let event = if log.address() == self.config.identity_registry {
            self.decode_identity_event(
                log,
                block_number,
                block_timestamp,
                &contract_address,
                &tx_hash,
                log_index,
            )?
        } else if log.address() == self.config.reputation_registry {
            self.decode_reputation_event(
                log,
                block_number,
                block_timestamp,
                &contract_address,
                &tx_hash,
                log_index,
            )?
        } else if log.address() == self.config.validation_registry {
            self.decode_validation_event(
                log,
                block_number,
                block_timestamp,
                &contract_address,
                &tx_hash,
                log_index,
            )?
        } else {
            return Ok(());
        };

        // Store the event in database
        self.storage.store_event(event.clone()).await?;

        // Broadcast event to WebSocket clients (ignore errors if no receivers)
        let _ = self.event_tx.send(event);

        Ok(())
    }

    /// Convert RPC Log to Primitive Log for event decoding
    fn convert_log(log: &Log) -> PrimitiveLog {
        PrimitiveLog {
            address: log.address(),
            data: LogData::new_unchecked(log.topics().to_vec(), log.data().data.clone()),
        }
    }

    fn decode_identity_event(
        &self,
        log: &Log,
        block_number: u64,
        block_timestamp: chrono::DateTime<chrono::Utc>,
        contract_address: &str,
        tx_hash: &str,
        log_index: u32,
    ) -> Result<Event> {
        let prim_log = Self::convert_log(log);

        // Try Registered
        if let Ok(decoded) = IdentityRegistry::Registered::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::Registered,
                event_data: EventData::Registered(RegisteredData {
                    agent_id: decoded.agentId.to_string(),
                    token_uri: decoded.tokenURI.clone(),
                    owner: format!("{:?}", decoded.owner),
                }),
                created_at: None,
            });
        }

        // Try MetadataSet
        if let Ok(decoded) = IdentityRegistry::MetadataSet::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::MetadataSet,
                event_data: EventData::MetadataSet(MetadataSetData {
                    agent_id: decoded.agentId.to_string(),
                    indexed_key: format!("{:?}", decoded.indexedKey),
                    key: decoded.key.clone(),
                    value: format!("0x{}", hex::encode(&decoded.value)),
                }),
                created_at: None,
            });
        }

        // Try UriUpdated
        if let Ok(decoded) = IdentityRegistry::UriUpdated::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::UriUpdated,
                event_data: EventData::UriUpdated(UriUpdatedData {
                    agent_id: decoded.agentId.to_string(),
                    new_uri: decoded.newUri.clone(),
                    updated_by: format!("{:?}", decoded.updatedBy),
                }),
                created_at: None,
            });
        }

        anyhow::bail!("Unknown IdentityRegistry event")
    }

    fn decode_reputation_event(
        &self,
        log: &Log,
        block_number: u64,
        block_timestamp: chrono::DateTime<chrono::Utc>,
        contract_address: &str,
        tx_hash: &str,
        log_index: u32,
    ) -> Result<Event> {
        let prim_log = Self::convert_log(log);

        // Try NewFeedback
        if let Ok(decoded) = ReputationRegistry::NewFeedback::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::NewFeedback,
                event_data: EventData::NewFeedback(NewFeedbackData {
                    agent_id: decoded.agentId.to_string(),
                    client: format!("{:?}", decoded.client),
                    score: decoded.score,
                    tag1: format!("{:?}", decoded.tag1),
                    tag2: format!("{:?}", decoded.tag2),
                    feedback_uri: decoded.feedbackURI.clone(),
                    feedback_hash: format!("{:?}", decoded.feedbackHash),
                }),
                created_at: None,
            });
        }

        // Try FeedbackRevoked
        if let Ok(decoded) = ReputationRegistry::FeedbackRevoked::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::FeedbackRevoked,
                event_data: EventData::FeedbackRevoked(FeedbackRevokedData {
                    agent_id: decoded.agentId.to_string(),
                    client: format!("{:?}", decoded.client),
                    feedback_index: decoded.feedbackIndex.to_string(),
                }),
                created_at: None,
            });
        }

        // Try ResponseAppended
        if let Ok(decoded) = ReputationRegistry::ResponseAppended::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::ResponseAppended,
                event_data: EventData::ResponseAppended(ResponseAppendedData {
                    agent_id: decoded.agentId.to_string(),
                    client: format!("{:?}", decoded.client),
                    feedback_index: decoded.feedbackIndex.to_string(),
                    responder: format!("{:?}", decoded.responder),
                    response_uri: decoded.responseURI.clone(),
                    response_hash: format!("{:?}", decoded.responseHash),
                }),
                created_at: None,
            });
        }

        anyhow::bail!("Unknown ReputationRegistry event")
    }

    fn decode_validation_event(
        &self,
        log: &Log,
        block_number: u64,
        block_timestamp: chrono::DateTime<chrono::Utc>,
        contract_address: &str,
        tx_hash: &str,
        log_index: u32,
    ) -> Result<Event> {
        let prim_log = Self::convert_log(log);

        // Try ValidationRequest
        if let Ok(decoded) = ValidationRegistry::ValidationRequest::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::ValidationRequest,
                event_data: EventData::ValidationRequest(ValidationRequestData {
                    validator_address: format!("{:?}", decoded.validatorAddress),
                    agent_id: decoded.agentId.to_string(),
                    request_uri: decoded.requestUri.clone(),
                    request_hash: format!("{:?}", decoded.requestHash),
                }),
                created_at: None,
            });
        }

        // Try ValidationResponse
        if let Ok(decoded) = ValidationRegistry::ValidationResponse::decode_log(&prim_log, true) {
            return Ok(Event {
                id: None,
                chain_id: self.config.chain_id,
                block_number,
                block_timestamp,
                transaction_hash: tx_hash.to_string(),
                log_index,
                contract_address: contract_address.to_string(),
                event_type: EventType::ValidationResponse,
                event_data: EventData::ValidationResponse(ValidationResponseData {
                    validator_address: format!("{:?}", decoded.validatorAddress),
                    agent_id: decoded.agentId.to_string(),
                    request_hash: format!("{:?}", decoded.requestHash),
                    response: decoded.response,
                    response_uri: decoded.responseUri.clone(),
                    response_hash: format!("{:?}", decoded.responseHash),
                    tag: format!("{:?}", decoded.tag),
                }),
                created_at: None,
            });
        }

        anyhow::bail!("Unknown ValidationRegistry event")
    }
}

#[cfg(test)]
mod tests {
    use crate::models::{Event, EventData, EventType, RegisteredData};
    use chrono::Utc;
    use tokio::sync::broadcast;

    fn create_test_event() -> Event {
        Event {
            id: None,
            chain_id: 11155111,
            block_number: 1000,
            block_timestamp: Utc::now(),
            transaction_hash: "0xabc".to_string(),
            log_index: 0,
            contract_address: "0x1234".to_string(),
            event_type: EventType::Registered,
            event_data: EventData::Registered(RegisteredData {
                agent_id: "1".to_string(),
                token_uri: "https://example.com".to_string(),
                owner: "0x5678".to_string(),
            }),
            created_at: None,
        }
    }

    #[test]
    fn test_event_broadcast_channel_creation() {
        // Test that broadcast channel can be created and used for events
        let (tx, _rx) = broadcast::channel::<Event>(100);

        // Send an event
        let event = create_test_event();
        let result = tx.send(event.clone());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 1); // 1 receiver
    }

    #[test]
    fn test_event_broadcast_multiple_receivers() {
        // Test that multiple receivers can subscribe to the same channel
        let (tx, _rx1) = broadcast::channel::<Event>(100);
        let _rx2 = tx.subscribe();
        let _rx3 = tx.subscribe();

        // Send event
        let event = create_test_event();
        let result = tx.send(event);

        // Should have 3 receivers
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 3);
    }

    #[test]
    fn test_metrics_collection() {
        // Test that metrics macros don't panic when called
        metrics::counter!("test_events_indexed", "chain_id" => "11155111").increment(1);
        metrics::gauge!("test_last_synced_block", "chain_id" => "11155111").set(1000.0);

        // If we reach here without panic, test passes
    }
}
