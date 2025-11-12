use crate::config::IndexerConfig;
use crate::indexer::Indexer;
use crate::models::Event;
use crate::stats::StatsTracker;
use crate::storage::Storage;
use anyhow::Result;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

/// Restart policy for indexer supervisor
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum RestartPolicy {
    Always,
    OnFailure,
    Exponential {
        max_retries: u32,
        base_delay_ms: u64,
        max_delay_ms: u64,
    },
}

/// Chain status for tracking and alerting
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ChainStatus {
    Active,
    Syncing,
    CatchingUp,
    Stalled,
    Failed,
}

impl ChainStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            ChainStatus::Active => "active",
            ChainStatus::Syncing => "syncing",
            ChainStatus::CatchingUp => "catching_up",
            ChainStatus::Stalled => "stalled",
            ChainStatus::Failed => "failed",
        }
    }
}

/// Supervisor that manages a single indexer with auto-restart capability
pub struct IndexerSupervisor {
    config: IndexerConfig,
    storage: Storage,
    event_tx: broadcast::Sender<Event>,
    restart_policy: RestartPolicy,
    stats_tracker: StatsTracker,
}

impl IndexerSupervisor {
    pub fn new(
        config: IndexerConfig,
        storage: Storage,
        event_tx: broadcast::Sender<Event>,
        restart_policy: RestartPolicy,
        stats_tracker: StatsTracker,
    ) -> Self {
        Self {
            config,
            storage,
            event_tx,
            restart_policy,
            stats_tracker,
        }
    }

    /// Start the supervisor loop
    pub async fn start(&self) -> Result<()> {
        let mut retry_count = 0;

        loop {
            info!(
                "[{}] Starting indexer for chain_id {}",
                self.config.name, self.config.chain_id
            );

            // Mark chain as active/syncing
            if let Err(e) = self
                .storage
                .update_chain_status(self.config.chain_id, ChainStatus::Syncing, None)
                .await
            {
                warn!(
                    "[{}] Failed to update chain status: {}",
                    self.config.name, e
                );
            }

            // Create and start indexer
            let indexer = match Indexer::new(
                self.config.clone(),
                self.storage.clone(),
                self.event_tx.clone(),
                self.stats_tracker.clone(),
            )
            .await
            {
                Ok(idx) => idx,
                Err(e) => {
                    error!("[{}] Failed to create indexer: {}", self.config.name, e);
                    self.storage
                        .update_chain_status(
                            self.config.chain_id,
                            ChainStatus::Failed,
                            Some(&e.to_string()),
                        )
                        .await?;
                    return Err(e);
                }
            };

            // Run indexer in isolated task
            let result = tokio::spawn(async move { indexer.start().await }).await;

            match result {
                Ok(Ok(())) => {
                    // Clean exit - indexer stopped gracefully
                    info!("[{}] Indexer exited cleanly", self.config.name);
                    self.storage
                        .update_chain_status(self.config.chain_id, ChainStatus::Active, None)
                        .await?;
                    break;
                }
                Ok(Err(e)) => {
                    // Indexer returned an error
                    error!("[{}] Indexer failed with error: {}", self.config.name, e);

                    // Check restart policy
                    match &self.restart_policy {
                        RestartPolicy::Always => {
                            warn!(
                                "[{}] Restarting immediately (Always policy)",
                                self.config.name
                            );
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        RestartPolicy::OnFailure => {
                            warn!("[{}] Restarting on failure", self.config.name);
                            sleep(Duration::from_secs(1)).await;
                            continue;
                        }
                        RestartPolicy::Exponential {
                            max_retries,
                            base_delay_ms,
                            max_delay_ms,
                        } => {
                            if retry_count >= *max_retries {
                                error!(
                                    "[{}] Max retries ({}) reached. Marking chain as FAILED.",
                                    self.config.name, max_retries
                                );
                                self.storage
                                    .update_chain_status(
                                        self.config.chain_id,
                                        ChainStatus::Failed,
                                        Some(&e.to_string()),
                                    )
                                    .await?;
                                return Err(e);
                            }

                            retry_count += 1;
                            let delay =
                                Self::calculate_backoff(retry_count, *base_delay_ms, *max_delay_ms);

                            warn!(
                                "[{}] Retry {}/{} - Restarting in {:?}...",
                                self.config.name, retry_count, max_retries, delay
                            );

                            // Update chain status to stalled
                            self.storage
                                .update_chain_status(
                                    self.config.chain_id,
                                    ChainStatus::Stalled,
                                    Some(&format!("Retry {}/{}: {}", retry_count, max_retries, e)),
                                )
                                .await?;

                            sleep(delay).await;
                        }
                    }
                }
                Err(e) => {
                    // Task panicked
                    error!("[{}] Indexer task panicked: {}", self.config.name, e);

                    // Always restart on panic
                    self.storage
                        .update_chain_status(
                            self.config.chain_id,
                            ChainStatus::Stalled,
                            Some(&format!("Panic: {}", e)),
                        )
                        .await?;

                    warn!(
                        "[{}] Restarting after panic in 1 second...",
                        self.config.name
                    );
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }

        Ok(())
    }

    /// Calculate exponential backoff delay
    fn calculate_backoff(retry: u32, base_delay_ms: u64, max_delay_ms: u64) -> Duration {
        let multiplier = 2u64.pow(retry);
        let delay_ms = (base_delay_ms * multiplier).min(max_delay_ms);
        Duration::from_millis(delay_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_backoff() {
        // Test exponential backoff (now a static method, no need for supervisor instance)
        assert_eq!(
            IndexerSupervisor::calculate_backoff(1, 1000, 60000),
            Duration::from_millis(2000)
        ); // 1s * 2^1
        assert_eq!(
            IndexerSupervisor::calculate_backoff(2, 1000, 60000),
            Duration::from_millis(4000)
        ); // 1s * 2^2
        assert_eq!(
            IndexerSupervisor::calculate_backoff(3, 1000, 60000),
            Duration::from_millis(8000)
        ); // 1s * 2^3
        assert_eq!(
            IndexerSupervisor::calculate_backoff(10, 1000, 60000),
            Duration::from_millis(60000)
        ); // Capped at max
    }

    #[test]
    fn test_chain_status_as_str() {
        assert_eq!(ChainStatus::Active.as_str(), "active");
        assert_eq!(ChainStatus::Syncing.as_str(), "syncing");
        assert_eq!(ChainStatus::CatchingUp.as_str(), "catching_up");
        assert_eq!(ChainStatus::Stalled.as_str(), "stalled");
        assert_eq!(ChainStatus::Failed.as_str(), "failed");
    }
}
