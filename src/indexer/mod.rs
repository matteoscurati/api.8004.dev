use crate::contracts::{
    IdentityRegistry, ReputationRegistry, ValidationRegistry,
};
use crate::models::{
    Event, EventData, EventType, FeedbackRevokedData, MetadataSetData, NewFeedbackData,
    RegisteredData, ResponseAppendedData, UriUpdatedData, ValidationRequestData,
    ValidationResponseData,
};
use crate::storage::Storage;
use alloy::{
    primitives::{Address, Log as PrimitiveLog, LogData},
    providers::{Provider, ProviderBuilder, RootProvider},
    rpc::types::{BlockTransactionsKind, Filter, Log},
    sol_types::SolEvent,
    transports::http::{Client, Http},
};
use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

/// Configuration for the indexer
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub rpc_url: String,
    pub identity_registry: Address,
    pub reputation_registry: Address,
    pub validation_registry: Address,
    pub starting_block: u64,
    pub poll_interval: Duration,
}

/// Event indexer that fetches events block by block
pub struct Indexer {
    config: IndexerConfig,
    provider: Arc<RootProvider<Http<Client>>>,
    storage: Storage,
}

impl Indexer {
    pub fn new(config: IndexerConfig, storage: Storage) -> Result<Self> {
        let provider = ProviderBuilder::new()
            .on_http(config.rpc_url.parse().context("Invalid RPC URL")?);

        Ok(Self {
            config,
            provider: Arc::new(provider),
            storage,
        })
    }

    /// Start the indexer loop
    pub async fn start(&self) -> Result<()> {
        info!("Starting ERC-8004 event indexer");
        info!("Monitoring contracts:");
        info!("  IdentityRegistry: {}", self.config.identity_registry);
        info!("  ReputationRegistry: {}", self.config.reputation_registry);
        info!("  ValidationRegistry: {}", self.config.validation_registry);

        // Get starting block
        let mut current_block = match self.storage.get_last_synced_block().await {
            Ok(block) if block > 0 => {
                info!("Resuming from block {}", block);
                block
            }
            _ => {
                let block = if self.config.starting_block == 0 {
                    self.provider
                        .get_block_number()
                        .await
                        .context("Failed to get current block number")?
                } else {
                    self.config.starting_block
                };
                info!("Starting from block {}", block);
                block
            }
        };

        loop {
            match self.sync_block(current_block).await {
                Ok(events_found) => {
                    if events_found > 0 {
                        info!("Block {}: Found {} events", current_block, events_found);
                    } else {
                        info!("Block {}: No events", current_block);
                    }

                    // Update last synced block
                    if let Err(e) = self.storage.update_last_synced_block(current_block).await {
                        warn!("Failed to update last synced block: {}", e);
                    }

                    current_block += 1;

                    // Add delay between blocks to respect rate limits
                    sleep(self.config.poll_interval).await;
                }
                Err(e) => {
                    // Check if we've caught up to the chain head
                    match self.provider.get_block_number().await {
                        Ok(latest_block) if current_block > latest_block => {
                            debug!(
                                "Caught up to chain head at block {}. Waiting for new blocks...",
                                latest_block
                            );
                            sleep(self.config.poll_interval).await;
                        }
                        _ => {
                            error!("Error syncing block {}: {}", current_block, e);
                            error!("Error details: {:?}", e);
                            warn!("Retrying in 5 seconds...");
                            sleep(Duration::from_secs(5)).await;
                        }
                    }
                }
            }
        }
    }

    /// Sync a single block and return number of events found
    async fn sync_block(&self, block_number: u64) -> Result<usize> {
        // Get block info for timestamp
        let block = self
            .provider
            .get_block_by_number(block_number.into(), BlockTransactionsKind::Hashes)
            .await
            .context("Failed to fetch block")?
            .context("Block not found")?;

        let block_timestamp = chrono::DateTime::from_timestamp(block.header.timestamp as i64, 0)
            .unwrap_or_else(|| chrono::Utc::now());

        // Fetch logs from all three contracts
        let filter = Filter::new()
            .from_block(block_number)
            .to_block(block_number)
            .address(vec![
                self.config.identity_registry,
                self.config.reputation_registry,
                self.config.validation_registry,
            ]);

        let logs = self
            .provider
            .get_logs(&filter)
            .await
            .context("Failed to fetch logs")?;

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
            self.decode_identity_event(log, block_number, block_timestamp, &contract_address, &tx_hash, log_index)?
        } else if log.address() == self.config.reputation_registry {
            self.decode_reputation_event(log, block_number, block_timestamp, &contract_address, &tx_hash, log_index)?
        } else if log.address() == self.config.validation_registry {
            self.decode_validation_event(log, block_number, block_timestamp, &contract_address, &tx_hash, log_index)?
        } else {
            return Ok(());
        };

        // Store the event
        self.storage.store_event(event).await?;

        Ok(())
    }

    /// Convert RPC Log to Primitive Log for event decoding
    fn convert_log(log: &Log) -> PrimitiveLog {
        PrimitiveLog {
            address: log.address(),
            data: LogData::new_unchecked(
                log.topics().to_vec(),
                log.data().data.clone().into(),
            ),
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
