use crate::config::RpcProvider;
use anyhow::{anyhow, Result};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// State for a single RPC provider
#[derive(Debug, Clone)]
struct ProviderState {
    provider: RpcProvider,
    request_count: u32,           // Requests made in current cycle
    requests_this_minute: u32,    // Requests in current minute window
    minute_window_start: Instant, // Start of current minute window
    last_error: Option<Instant>,
    in_cooldown: bool,
    consecutive_errors: u32,
}

impl ProviderState {
    fn new(provider: RpcProvider) -> Self {
        Self {
            provider,
            request_count: 0,
            requests_this_minute: 0,
            minute_window_start: Instant::now(),
            last_error: None,
            in_cooldown: false,
            consecutive_errors: 0,
        }
    }

    /// Check if provider is available (not in cooldown and under rate limit)
    fn is_available(&self) -> bool {
        if self.in_cooldown {
            return false;
        }

        // Check rate limit (sliding minute window)
        let elapsed = self.minute_window_start.elapsed();
        if elapsed < Duration::from_secs(60) {
            self.requests_this_minute < self.provider.max_requests_per_minute
        } else {
            true
        }
    }

    /// Update minute window if needed
    fn update_minute_window(&mut self) {
        let elapsed = self.minute_window_start.elapsed();
        if elapsed >= Duration::from_secs(60) {
            self.minute_window_start = Instant::now();
            self.requests_this_minute = 0;
        }
    }

    /// Check if should rotate based on weight
    fn should_rotate(&self) -> bool {
        self.request_count >= self.provider.weight
    }

    /// Reset request count for rotation
    fn reset_count(&mut self) {
        self.request_count = 0;
    }
}

/// Manages multiple RPC providers with adaptive smart rotation
pub struct ProviderManager {
    providers: Arc<RwLock<Vec<ProviderState>>>,
    current_index: Arc<RwLock<usize>>,
    chain_name: String,
}

impl ProviderManager {
    /// Create a new ProviderManager from a list of RPC providers
    pub fn new(providers: Vec<RpcProvider>, chain_name: String) -> Result<Self> {
        if providers.is_empty() {
            return Err(anyhow!(
                "No RPC providers configured for chain {}",
                chain_name
            ));
        }

        // Sort providers by priority
        let mut sorted_providers = providers;
        sorted_providers.sort_by_key(|p| p.priority);

        let provider_states: Vec<ProviderState> = sorted_providers
            .into_iter()
            .map(ProviderState::new)
            .collect();

        info!(
            "[{}] Initialized ProviderManager with {} providers",
            chain_name,
            provider_states.len()
        );

        for (i, state) in provider_states.iter().enumerate() {
            debug!(
                "[{}] Provider {}: priority={}, weight={}, max_req/min={}",
                chain_name,
                i,
                state.provider.priority,
                state.provider.weight,
                state.provider.max_requests_per_minute
            );
        }

        Ok(Self {
            providers: Arc::new(RwLock::new(provider_states)),
            current_index: Arc::new(RwLock::new(0)),
            chain_name,
        })
    }

    /// Get the current RPC provider URL
    pub async fn get_current_provider(&self) -> Result<String> {
        let mut providers = self.providers.write().await;
        let mut current_index = self.current_index.write().await;

        // Update minute windows for all providers
        for provider in providers.iter_mut() {
            provider.update_minute_window();

            // Check if cooldown expired
            if provider.in_cooldown {
                if let Some(last_error) = provider.last_error {
                    let cooldown_duration =
                        Duration::from_millis(provider.provider.cooldown_on_error_ms);
                    if last_error.elapsed() >= cooldown_duration {
                        provider.in_cooldown = false;
                        provider.consecutive_errors = 0;
                        info!(
                            "[{}] Provider {} recovered from cooldown",
                            self.chain_name, provider.provider.url
                        );
                    }
                }
            }
        }

        // Find next available provider
        let total_providers = providers.len();
        let mut attempts = 0;

        while attempts < total_providers {
            // Check if current provider should rotate
            let should_rotate = {
                let current = &providers[*current_index];
                current.should_rotate() && total_providers > 1
            };

            if should_rotate {
                let current = &mut providers[*current_index];
                debug!(
                    "[{}] Rotating from provider {} (reached weight {})",
                    self.chain_name, *current_index, current.provider.weight
                );
                current.reset_count();
                *current_index = (*current_index + 1) % total_providers;
                continue;
            }

            // Check if current provider is available
            let is_available = {
                let current = &providers[*current_index];
                current.is_available()
            };

            if is_available {
                let current = &providers[*current_index];
                return Ok(current.provider.url.clone());
            }

            // Try next provider
            {
                let current = &providers[*current_index];
                warn!(
                    "[{}] Provider {} unavailable (cooldown={}, rate_limited={}), trying next",
                    self.chain_name,
                    *current_index,
                    current.in_cooldown,
                    !current.is_available()
                );
            }
            *current_index = (*current_index + 1) % total_providers;
            attempts += 1;
        }

        // All providers unavailable
        Err(anyhow!(
            "[{}] All {} RPC providers are unavailable (rate limited or in cooldown)",
            self.chain_name,
            total_providers
        ))
    }

    /// Mark a successful request
    pub async fn mark_success(&self) {
        let mut providers = self.providers.write().await;
        let current_index = self.current_index.read().await;

        if let Some(provider) = providers.get_mut(*current_index) {
            provider.request_count += 1;
            provider.requests_this_minute += 1;
            provider.consecutive_errors = 0;

            debug!(
                "[{}] Provider {} request #{} successful (weight: {}/{})",
                self.chain_name,
                *current_index,
                provider.request_count,
                provider.request_count,
                provider.provider.weight
            );
        }
    }

    /// Mark a failed request (triggers cooldown)
    pub async fn mark_error(&self, error_msg: &str) {
        let mut providers = self.providers.write().await;
        let mut current_index_lock = self.current_index.write().await;
        let current_index = *current_index_lock;

        if let Some(provider) = providers.get_mut(current_index) {
            provider.last_error = Some(Instant::now());
            provider.consecutive_errors += 1;
            provider.in_cooldown = true;

            warn!(
                "[{}] Provider {} failed: {} (consecutive errors: {}, cooldown: {}ms)",
                self.chain_name,
                current_index,
                error_msg,
                provider.consecutive_errors,
                provider.provider.cooldown_on_error_ms
            );

            // Reset count and rotate to next provider
            provider.reset_count();

            // Find next available provider
            let start_index = current_index;
            let total_providers = providers.len();
            let mut next_index = (current_index + 1) % total_providers;
            let mut attempts = 0;

            while attempts < total_providers && next_index != start_index {
                if providers[next_index].is_available() {
                    info!(
                        "[{}] Rotating to provider {} after error",
                        self.chain_name, next_index
                    );
                    *current_index_lock = next_index;
                    return;
                }
                next_index = (next_index + 1) % total_providers;
                attempts += 1;
            }

            warn!(
                "[{}] No other providers available after error, staying on provider {}",
                self.chain_name, current_index
            );
        }
    }

    /// Get statistics for monitoring
    #[allow(dead_code)]
    pub async fn get_stats(&self) -> ProviderStats {
        let mut providers = self.providers.write().await;
        let current_index = *self.current_index.read().await;

        // Update cooldown status before counting
        for provider in providers.iter_mut() {
            if provider.in_cooldown {
                if let Some(last_error) = provider.last_error {
                    let cooldown_duration =
                        Duration::from_millis(provider.provider.cooldown_on_error_ms);
                    if last_error.elapsed() >= cooldown_duration {
                        provider.in_cooldown = false;
                        provider.consecutive_errors = 0;
                    }
                }
            }
        }

        let total = providers.len();
        let available = providers.iter().filter(|p| p.is_available()).count();
        let in_cooldown = providers.iter().filter(|p| p.in_cooldown).count();

        ProviderStats {
            total_providers: total,
            available_providers: available,
            cooldown_providers: in_cooldown,
            current_provider_index: current_index,
            current_provider_url: providers
                .get(current_index)
                .map(|p| p.provider.url.clone())
                .unwrap_or_default(),
        }
    }
}

/// Statistics about provider manager status
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ProviderStats {
    pub total_providers: usize,
    pub available_providers: usize,
    pub cooldown_providers: usize,
    pub current_provider_index: usize,
    pub current_provider_url: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_provider_rotation() {
        let providers = vec![
            RpcProvider {
                url: "http://provider1.com".to_string(),
                weight: 2,
                priority: 1,
                max_requests_per_minute: 10,
                cooldown_on_error_ms: 1000,
            },
            RpcProvider {
                url: "http://provider2.com".to_string(),
                weight: 2,
                priority: 2,
                max_requests_per_minute: 10,
                cooldown_on_error_ms: 1000,
            },
        ];

        let manager = ProviderManager::new(providers, "test".to_string()).unwrap();

        // First provider
        let url1 = manager.get_current_provider().await.unwrap();
        assert_eq!(url1, "http://provider1.com");

        // Mark 2 successful requests (reaches weight)
        manager.mark_success().await;
        manager.mark_success().await;

        // Should rotate to provider2
        let url2 = manager.get_current_provider().await.unwrap();
        assert_eq!(url2, "http://provider2.com");
    }

    #[tokio::test]
    async fn test_provider_failover() {
        let providers = vec![
            RpcProvider {
                url: "http://provider1.com".to_string(),
                weight: 10,
                priority: 1,
                max_requests_per_minute: 10,
                cooldown_on_error_ms: 100,
            },
            RpcProvider {
                url: "http://provider2.com".to_string(),
                weight: 10,
                priority: 2,
                max_requests_per_minute: 10,
                cooldown_on_error_ms: 100,
            },
        ];

        let manager = ProviderManager::new(providers, "test".to_string()).unwrap();

        // First provider
        let url1 = manager.get_current_provider().await.unwrap();
        assert_eq!(url1, "http://provider1.com");

        // Mark error - should failover to provider2
        manager.mark_error("test error").await;
        let url2 = manager.get_current_provider().await.unwrap();
        assert_eq!(url2, "http://provider2.com");

        // After cooldown, provider1 should be available again
        tokio::time::sleep(Duration::from_millis(150)).await;
        let stats = manager.get_stats().await;
        assert_eq!(stats.available_providers, 2);
    }

    #[tokio::test]
    async fn test_rate_limiting() {
        let providers = vec![RpcProvider {
            url: "http://provider1.com".to_string(),
            weight: 100,
            priority: 1,
            max_requests_per_minute: 2, // Very low limit
            cooldown_on_error_ms: 1000,
        }];

        let manager = ProviderManager::new(providers, "test".to_string()).unwrap();

        // Make 2 requests (hits limit)
        manager.mark_success().await;
        manager.mark_success().await;

        // Third request should fail due to rate limit
        let result = manager.get_current_provider().await;
        assert!(result.is_err());
    }
}
