use alloy::primitives::Address;
use anyhow::{anyhow, Context, Result};
use std::env;
use std::str::FromStr;
use tokio::time::Duration;
use tracing::warn;

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    // Ethereum
    pub rpc_url: String,
    pub chain_id: u64,
    pub identity_registry: Address,
    pub reputation_registry: Address,
    pub validation_registry: Address,
    pub starting_block: u64,
    pub poll_interval: Duration,

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
        if jwt_secret.contains("changeme") || jwt_secret.contains("secret") || jwt_secret.contains("change-this") {
            warn!("⚠️  JWT_SECRET appears to use a default/weak value. Change it in production!");
        }

        // Validate authentication credentials
        env::var("AUTH_USERNAME").context("AUTH_USERNAME must be set")?;

        if env::var("AUTH_PASSWORD_HASH").is_err() {
            if env::var("AUTH_PASSWORD").is_err() {
                return Err(anyhow!("Either AUTH_PASSWORD_HASH or AUTH_PASSWORD must be set"));
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
            warn!("⚠️  RATE_LIMIT_REQUESTS is set very high ({}). Consider lowering for production.", rate_limit);
        }

        Ok(())
    }

    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        dotenvy::dotenv().ok();

        // Validate security settings first
        Self::validate_security_settings()?;

        let rpc_url = env::var("RPC_URL").context("RPC_URL not set")?;

        let chain_id = env::var("CHAIN_ID")
            .unwrap_or_else(|_| "11155111".to_string()) // Default to Sepolia
            .parse()
            .context("Invalid CHAIN_ID")?;

        let identity_registry = Address::from_str(
            &env::var("IDENTITY_REGISTRY_ADDRESS")
                .context("IDENTITY_REGISTRY_ADDRESS not set")?,
        )
        .context("Invalid IDENTITY_REGISTRY_ADDRESS")?;

        let reputation_registry = Address::from_str(
            &env::var("REPUTATION_REGISTRY_ADDRESS")
                .context("REPUTATION_REGISTRY_ADDRESS not set")?,
        )
        .context("Invalid REPUTATION_REGISTRY_ADDRESS")?;

        let validation_registry = Address::from_str(
            &env::var("VALIDATION_REGISTRY_ADDRESS")
                .context("VALIDATION_REGISTRY_ADDRESS not set")?,
        )
        .context("Invalid VALIDATION_REGISTRY_ADDRESS")?;

        let database_url = env::var("DATABASE_URL").context("DATABASE_URL not set")?;

        let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
        let server_port = env::var("SERVER_PORT")
            .unwrap_or_else(|_| "8080".to_string())
            .parse()
            .context("Invalid SERVER_PORT")?;

        let starting_block_str = env::var("STARTING_BLOCK").unwrap_or_else(|_| "latest".to_string());
        let starting_block = if starting_block_str == "latest" {
            0 // Will be resolved to latest block at runtime
        } else {
            starting_block_str
                .parse()
                .context("Invalid STARTING_BLOCK")?
        };

        let poll_interval_ms: u64 = env::var("POLL_INTERVAL_MS")
            .unwrap_or_else(|_| "12000".to_string())
            .parse()
            .context("Invalid POLL_INTERVAL_MS")?;

        let max_events_in_memory: usize = env::var("MAX_EVENTS_IN_MEMORY")
            .unwrap_or_else(|_| "10000".to_string())
            .parse()
            .context("Invalid MAX_EVENTS_IN_MEMORY")?;

        Ok(Self {
            rpc_url,
            chain_id,
            identity_registry,
            reputation_registry,
            validation_registry,
            starting_block,
            poll_interval: Duration::from_millis(poll_interval_ms),
            database_url,
            server_host,
            server_port,
            max_events_in_memory,
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
        env::set_var("JWT_SECRET", "this-is-a-very-long-and-secure-secret-key-for-jwt");
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
        assert!(result.unwrap_err().to_string().contains("at least 32 characters"));
    }

    #[test]
    #[serial]
    fn test_validate_security_settings_missing_username() {
        env::set_var("JWT_SECRET", "this-is-a-very-long-and-secure-secret-key");
        env::remove_var("AUTH_USERNAME");
        env::set_var("AUTH_PASSWORD", "password");

        let result = Config::validate_security_settings();
        assert!(result.is_err());
    }

    #[test]
    #[serial]
    fn test_validate_security_settings_no_password() {
        env::set_var("JWT_SECRET", "this-is-a-very-long-and-secure-secret-key");
        env::set_var("AUTH_USERNAME", "admin");
        env::remove_var("AUTH_PASSWORD");
        env::remove_var("AUTH_PASSWORD_HASH");

        let result = Config::validate_security_settings();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("AUTH_PASSWORD"));
    }

    #[test]
    #[serial]
    fn test_config_loads_successfully() {
        // Just verify that config can be loaded with all required vars set
        env::set_var("RPC_URL", "https://rpc.example.com");
        env::set_var("IDENTITY_REGISTRY_ADDRESS", "0x1111111111111111111111111111111111111111");
        env::set_var("REPUTATION_REGISTRY_ADDRESS", "0x2222222222222222222222222222222222222222");
        env::set_var("VALIDATION_REGISTRY_ADDRESS", "0x3333333333333333333333333333333333333333");
        env::set_var("DATABASE_URL", "postgresql://localhost/test");
        env::set_var("JWT_SECRET", "this-is-a-very-long-and-secure-secret-key-for-testing-ok");
        env::set_var("AUTH_USERNAME", "admin");
        env::set_var("AUTH_PASSWORD", "password");

        let result = Config::from_env();
        assert!(result.is_ok(), "Config should load successfully");
    }
}
