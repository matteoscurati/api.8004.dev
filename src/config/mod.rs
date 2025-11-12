use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::str::FromStr;
use tokio::time::Duration;
use tracing::warn;

/// Configuration for a single RPC provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcProvider {
    pub url: String,
    #[serde(default = "default_provider_weight")]
    pub weight: u32,
    #[serde(default = "default_provider_priority")]
    pub priority: u32,
    #[serde(default = "default_max_requests_per_minute")]
    pub max_requests_per_minute: u32,
    #[serde(default = "default_cooldown_on_error")]
    pub cooldown_on_error_ms: u64,
}

fn default_provider_weight() -> u32 {
    30
}

fn default_provider_priority() -> u32 {
    1
}

fn default_max_requests_per_minute() -> u32 {
    100
}

fn default_cooldown_on_error() -> u64 {
    60000
}

/// Configuration for a single blockchain network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    pub name: String,
    pub chain_id: u64,
    pub enabled: bool,
    #[serde(default)]
    pub rpc_providers: Vec<RpcProvider>,
    // Backward compatibility: single rpc_url (will be converted to providers)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rpc_url: Option<String>,
    pub contracts: ContractAddresses,
    pub starting_block: String, // "latest" or block number
    pub poll_interval_ms: u64,
    #[serde(default = "default_batch_size")]
    pub batch_size: u64,
    #[serde(default = "default_adaptive_polling")]
    pub adaptive_polling: bool,
}

impl ChainConfig {
    /// Get RPC providers, handling backward compatibility with single rpc_url
    pub fn get_providers(&self) -> Vec<RpcProvider> {
        if !self.rpc_providers.is_empty() {
            self.rpc_providers.clone()
        } else if let Some(url) = &self.rpc_url {
            // Backward compatibility: convert single URL to provider
            vec![RpcProvider {
                url: url.clone(),
                weight: default_provider_weight(),
                priority: default_provider_priority(),
                max_requests_per_minute: default_max_requests_per_minute(),
                cooldown_on_error_ms: default_cooldown_on_error(),
            }]
        } else {
            vec![]
        }
    }
}

fn default_batch_size() -> u64 {
    1
}

fn default_adaptive_polling() -> bool {
    true
}

/// Contract addresses for a chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAddresses {
    pub identity_registry: String,
    pub reputation_registry: String,
    pub validation_registry: String,
}

/// Global configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    #[serde(default = "default_max_retries")]
    pub max_indexer_retries: u32,
    #[serde(default = "default_retry_base_delay")]
    pub retry_base_delay_ms: u64,
    #[serde(default = "default_retry_max_delay")]
    pub retry_max_delay_ms: u64,
    #[serde(default = "default_adaptive_polling")]
    pub adaptive_polling_enabled: bool,
    #[serde(default = "default_max_parallel")]
    pub max_parallel_blocks: usize,
    #[serde(default = "default_batch_delay")]
    pub batch_processing_delay_ms: u64,
}

fn default_max_retries() -> u32 {
    5
}

fn default_retry_base_delay() -> u64 {
    1000
}

fn default_retry_max_delay() -> u64 {
    60000
}

fn default_max_parallel() -> usize {
    10
}

fn default_batch_delay() -> u64 {
    50
}

/// Multi-chain configuration from chains.yaml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainsYaml {
    pub chains: Vec<ChainConfig>,
    #[serde(default)]
    pub global: GlobalConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            max_indexer_retries: default_max_retries(),
            retry_base_delay_ms: default_retry_base_delay(),
            retry_max_delay_ms: default_retry_max_delay(),
            adaptive_polling_enabled: default_adaptive_polling(),
            max_parallel_blocks: default_max_parallel(),
            batch_processing_delay_ms: default_batch_delay(),
        }
    }
}

/// Application configuration combining chains.yaml and environment variables
#[derive(Debug, Clone)]
pub struct Config {
    // Chains configuration
    pub chains: Vec<ChainConfig>,
    pub global: GlobalConfig,

    // Database
    pub database_url: String,

    // Server
    pub server_host: String,
    pub server_port: u16,

    // Storage
    pub max_events_in_memory: usize,
}

impl Config {
    /// Validate security-related environment variables
    fn validate_security_settings() -> Result<()> {
        // Validate JWT_SECRET
        let jwt_secret = env::var("JWT_SECRET").context("JWT_SECRET must be set")?;
        if jwt_secret.len() < 32 {
            return Err(anyhow!("JWT_SECRET must be at least 32 characters long"));
        }
        if jwt_secret.contains("changeme")
            || jwt_secret.contains("secret")
            || jwt_secret.contains("change-this")
        {
            warn!("⚠️  JWT_SECRET appears to use a default/weak value. Change it in production!");
        }

        // Validate authentication credentials
        env::var("AUTH_USERNAME").context("AUTH_USERNAME must be set")?;

        if env::var("AUTH_PASSWORD_HASH").is_err() {
            if env::var("AUTH_PASSWORD").is_err() {
                return Err(anyhow!(
                    "Either AUTH_PASSWORD_HASH or AUTH_PASSWORD must be set"
                ));
            }
            warn!("⚠️  Using plain text AUTH_PASSWORD. Use AUTH_PASSWORD_HASH in production!");
        }

        // Validate CORS settings
        let cors = env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());
        if cors == "*" {
            warn!("⚠️  CORS is set to allow all origins (*). NOT recommended for production!");
        }

        // Validate rate limiting
        let rate_limit = env::var("RATE_LIMIT_REQUESTS")
            .unwrap_or_else(|_| "100".to_string())
            .parse::<usize>()
            .context("Invalid RATE_LIMIT_REQUESTS")?;

        if rate_limit > 1000 {
            warn!(
                "⚠️  RATE_LIMIT_REQUESTS is set very high ({}). Consider lowering for production.",
                rate_limit
            );
        }

        Ok(())
    }

    /// Load configuration from chains.yaml and environment variables
    pub fn from_yaml_and_env(yaml_path: &str) -> Result<Self> {
        dotenvy::dotenv().ok();

        // Validate security settings first
        Self::validate_security_settings()?;

        // Load chains.yaml
        let yaml_content =
            fs::read_to_string(yaml_path).context(format!("Failed to read {}", yaml_path))?;
        let chains_yaml: ChainsYaml =
            serde_yaml::from_str(&yaml_content).context("Failed to parse chains.yaml")?;

        // Filter enabled chains
        let enabled_chains: Vec<ChainConfig> = chains_yaml
            .chains
            .into_iter()
            .filter(|chain| chain.enabled)
            .collect();

        if enabled_chains.is_empty() {
            return Err(anyhow!("No enabled chains found in chains.yaml"));
        }

        // Load environment variables for database and server
        let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;
        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("Invalid SERVER_PORT")?;

        let max_events_in_memory: usize = env::var("MAX_EVENTS_IN_MEMORY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .context("Invalid MAX_EVENTS_IN_MEMORY")?;

        Ok(Self {
            chains: enabled_chains,
            global: chains_yaml.global,
            database_url,
            server_host,
            server_port,
            max_events_in_memory,
        })
    }

    /// Legacy: Load configuration from environment variables only (for backward compatibility)
    /// This is used when chains.yaml doesn't exist
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        // Validate security settings first
        Self::validate_security_settings()?;

        let rpc_url = env::var("RPC_URL").context("RPC_URL not set")?;

        let chain_id = env::var("CHAIN_ID")
            .unwrap_or_else(|_| "11155111".to_string()) // Default to Sepolia
            .parse()
            .context("Invalid CHAIN_ID")?;

        let identity_registry =
            env::var("IDENTITY_REGISTRY_ADDRESS").context("IDENTITY_REGISTRY_ADDRESS not set")?;
        let reputation_registry = env::var("REPUTATION_REGISTRY_ADDRESS")
            .context("REPUTATION_REGISTRY_ADDRESS not set")?;
        let validation_registry = env::var("VALIDATION_REGISTRY_ADDRESS")
            .context("VALIDATION_REGISTRY_ADDRESS not set")?;

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("Invalid SERVER_PORT")?;

        let starting_block_str =
            env::var("STARTING_BLOCK").unwrap_or_else(|_| "latest".to_string());

        let poll_interval_ms: u64 = env::var("POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "12000".to_string())
            .parse()
            .context("Invalid POLL_INTERVAL_MS")?;

        let max_events_in_memory: usize = env::var("MAX_EVENTS_IN_MEMORY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .context("Invalid MAX_EVENTS_IN_MEMORY")?;

        // Create a single-chain configuration (backward compatibility with single RPC_URL)
        let chain = ChainConfig {
            name: format!("Chain {}", chain_id),
            chain_id,
            enabled: true,
            rpc_providers: vec![RpcProvider {
                url: rpc_url,
                weight: default_provider_weight(),
                priority: default_provider_priority(),
                max_requests_per_minute: default_max_requests_per_minute(),
                cooldown_on_error_ms: default_cooldown_on_error(),
            }],
            rpc_url: None,
            contracts: ContractAddresses {
                identity_registry,
                reputation_registry,
                validation_registry,
            },
            starting_block: starting_block_str,
            poll_interval_ms,
            batch_size: 1,
            adaptive_polling: true,
        };

        Ok(Self {
            chains: vec![chain],
            global: GlobalConfig::default(),
            database_url,
            server_host,
            server_port,
            max_events_in_memory,
        })
    }
}

/// Indexer-specific configuration (converted from ChainConfig)
#[derive(Debug, Clone)]
pub struct IndexerConfig {
    pub name: String,
    pub rpc_providers: Vec<RpcProvider>,
    pub chain_id: u64,
    pub identity_registry: Address,
    pub reputation_registry: Address,
    pub validation_registry: Address,
    pub starting_block: u64,
    pub poll_interval: Duration,
    pub batch_size: u64,
    pub adaptive_polling: bool,
}

impl IndexerConfig {
    /// Convert ChainConfig to IndexerConfig (with address parsing)
    pub fn from_chain_config(chain: &ChainConfig) -> Result<Self> {
        let identity_registry = Address::from_str(&chain.contracts.identity_registry)
            .context("Invalid identity_registry address")?;
        let reputation_registry = Address::from_str(&chain.contracts.reputation_registry)
            .context("Invalid reputation_registry address")?;
        let validation_registry = Address::from_str(&chain.contracts.validation_registry)
            .context("Invalid validation_registry address")?;

        // Parse starting_block (will be resolved to actual block number at runtime if "latest")
        let starting_block = if chain.starting_block == "latest" {
            0 // Will be resolved later
        } else {
            chain
                .starting_block
                .parse()
                .context("Invalid starting_block")?
        };

        let providers = chain.get_providers();
        if providers.is_empty() {
            return Err(anyhow!(
                "No RPC providers configured for chain {}",
                chain.name
            ));
        }

        Ok(Self {
            name: chain.name.clone(),
            rpc_providers: providers,
            chain_id: chain.chain_id,
            identity_registry,
            reputation_registry,
            validation_registry,
            starting_block,
            poll_interval: Duration::from_millis(chain.poll_interval_ms),
            batch_size: chain.batch_size,
            adaptive_polling: chain.adaptive_polling,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;

    #[test]
    #[serial]
    fn test_validate_security_settings_valid() {
        env::set_var(
            "JWT_SECRET",
            "this-is-a-very-long-and-secure-secret-key-for-jwt",
        );
        env::set_var("AUTH_USERNAME", "admin");
        env::set_var("AUTH_PASSWORD", "testpassword");
        env::set_var("CORS_ALLOWED_ORIGINS", "http://localhost:3000");
        env::set_var("RATE_LIMIT_REQUESTS", "100");

        let result = Config::validate_security_settings();
        assert!(result.is_ok());
    }

    #[test]
    #[serial]
    fn test_validate_security_settings_short_jwt_secret() {
        env::set_var("JWT_SECRET", "short");
        env::set_var("AUTH_USERNAME", "admin");
        env::set_var("AUTH_PASSWORD", "password");

        let result = Config::validate_security_settings();
        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("at least 32 characters"));
    }

    #[test]
    fn test_chain_config_deserialization() {
        let yaml = r#"
chains:
  - name: "Test Chain"
    chain_id: 123
    enabled: true
    rpc_url: "https://test.rpc"
    contracts:
      identity_registry: "0x1111111111111111111111111111111111111111"
      reputation_registry: "0x2222222222222222222222222222222222222222"
      validation_registry: "0x3333333333333333333333333333333333333333"
    starting_block: "latest"
    poll_interval_ms: 5000
    batch_size: 2
    adaptive_polling: true
global:
  max_indexer_retries: 3
"#;

        let config: ChainsYaml = serde_yaml::from_str(yaml).unwrap();
        assert_eq!(config.chains.len(), 1);
        assert_eq!(config.chains[0].name, "Test Chain");
        assert_eq!(config.chains[0].chain_id, 123);
        assert_eq!(config.global.max_indexer_retries, 3);
    }
}
