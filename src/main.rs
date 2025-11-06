mod api;
mod auth;
mod config;
mod contracts;
mod indexer;
mod metrics;
mod models;
mod rate_limit;
mod retry;
mod storage;

use anyhow::Result;
use config::Config;
use indexer::{Indexer, IndexerConfig};
use sqlx::postgres::PgPoolOptions;
use storage::Storage;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Setup graceful shutdown signal
    let _shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("Shutdown signal received, gracefully shutting down...");
    };
    // Initialize logging (JSON format if LOG_FORMAT=json)
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "erc8004_indexer=info,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "erc8004_indexer=info,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    info!("Starting ERC-8004 Indexer");

    // Initialize metrics
    let metrics_handle = metrics::init_metrics();
    info!("Metrics initialized");

    // Load configuration
    let config = Config::from_env()?;
    info!("Configuration loaded successfully");

    // Connect to database
    info!("Connecting to database...");
    let max_connections = std::env::var("DB_MAX_CONNECTIONS")
        .unwrap_or_else(|_| "10".to_string())
        .parse()
        .unwrap_or(10);

    let min_connections = std::env::var("DB_MIN_CONNECTIONS")
        .unwrap_or_else(|_| "2".to_string())
        .parse()
        .unwrap_or(2);

    let acquire_timeout = std::env::var("DB_ACQUIRE_TIMEOUT_SECS")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30);

    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(min_connections)
        .acquire_timeout(std::time::Duration::from_secs(acquire_timeout))
        .connect(&config.database_url)
        .await?;

    info!(
        "Database connected (pool: {}-{} connections)",
        min_connections, max_connections
    );

    info!("Database connected successfully");

    // Run migrations
    info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("Migrations completed");

    // Create storage
    let storage = Storage::new(pool, config.max_events_in_memory);

    // Create indexer
    let indexer_config = IndexerConfig {
        rpc_url: config.rpc_url.clone(),
        identity_registry: config.identity_registry,
        reputation_registry: config.reputation_registry,
        validation_registry: config.validation_registry,
        starting_block: config.starting_block,
        poll_interval: config.poll_interval,
    };

    let indexer = Indexer::new(indexer_config, storage.clone())?;

    // Start API server in background
    let api_storage = storage.clone();
    let api_host = config.server_host.clone();
    let api_port = config.server_port;
    let api_metrics = metrics_handle.clone();

    tokio::spawn(async move {
        if let Err(e) = api::start_server(api_host, api_port, api_storage, api_metrics).await {
            error!("API server error: {}", e);
        }
    });

    // Start indexer (this blocks)
    info!("Starting indexer...");
    indexer.start().await?;

    Ok(())
}
