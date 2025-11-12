mod api;
mod auth;
mod config;
mod contracts;
mod indexer;
mod models;
mod rpc;
mod stats;
mod storage;

use anyhow::Result;
use config::{Config, IndexerConfig};
use indexer::supervisor::{IndexerSupervisor, RestartPolicy};
use sqlx::postgres::PgPoolOptions;
use stats::StatsTracker;
use std::path::Path;
use storage::Storage;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging (JSON format if LOG_FORMAT=json)
    let log_format = std::env::var("LOG_FORMAT").unwrap_or_else(|_| "text".to_string());

    if log_format == "json" {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "api_8004_dev=info,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(
                tracing_subscriber::EnvFilter::try_from_default_env()
                    .unwrap_or_else(|_| "api_8004_dev=info,tower_http=debug".into()),
            )
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    info!("🚀 Starting ERC-8004 Multi-Chain Indexer");

    // Initialize metrics
    let metrics_handle = metrics_exporter_prometheus::PrometheusBuilder::new()
        .install_recorder()
        .expect("Failed to install Prometheus recorder");
    info!("✅ Metrics initialized");

    // Load configuration (try chains.yaml first, fallback to env)
    let config = if Path::new("chains.yaml").exists() {
        info!("📋 Loading configuration from chains.yaml");
        Config::from_yaml_and_env("chains.yaml")?
    } else {
        warn!(
            "⚠️  chains.yaml not found, falling back to environment variables (single-chain mode)"
        );
        Config::from_env()?
    };

    info!("✅ Configuration loaded successfully");
    info!("📊 Enabled chains: {}", config.chains.len());
    for chain in &config.chains {
        info!("   - {} (chain_id: {})", chain.name, chain.chain_id);
    }

    // Connect to database
    info!("🔌 Connecting to database...");
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
        "✅ Database connected (pool: {}-{} connections)",
        min_connections, max_connections
    );

    // Run migrations
    info!("🔄 Running database migrations...");
    sqlx::migrate!("./migrations").run(&pool).await?;
    info!("✅ Migrations completed");

    // Create shared storage
    let storage = Storage::new(pool, config.max_events_in_memory);

    // Create broadcast channel for real-time event streaming
    let (event_tx, _) = tokio::sync::broadcast::channel::<models::Event>(1000);

    // Create stats tracker for monitoring
    let stats_tracker = StatsTracker::new();

    // Spawn supervisor for each enabled chain
    info!(
        "🔧 Starting indexer supervisors for {} chains...",
        config.chains.len()
    );

    let mut supervisor_handles = vec![];

    for chain in &config.chains {
        // Convert ChainConfig to IndexerConfig
        let indexer_config = match IndexerConfig::from_chain_config(chain) {
            Ok(cfg) => cfg,
            Err(e) => {
                error!(
                    "❌ Failed to create indexer config for {}: {}",
                    chain.name, e
                );
                continue;
            }
        };

        // Create supervisor with exponential backoff restart policy
        let supervisor = IndexerSupervisor::new(
            indexer_config,
            storage.clone(),
            event_tx.clone(),
            RestartPolicy::Exponential {
                max_retries: config.global.max_indexer_retries,
                base_delay_ms: config.global.retry_base_delay_ms,
                max_delay_ms: config.global.retry_max_delay_ms,
            },
            stats_tracker.clone(),
        );

        let chain_name = chain.name.clone();

        // Spawn supervisor in its own task
        let handle = tokio::spawn(async move {
            info!("🚀 Starting supervisor for {}", chain_name);
            match supervisor.start().await {
                Ok(()) => {
                    info!("✅ Supervisor {} exited cleanly", chain_name);
                }
                Err(e) => {
                    error!("❌ Supervisor {} failed: {}", chain_name, e);
                }
            }
        });

        supervisor_handles.push((chain.name.clone(), handle));
    }

    info!("✅ All supervisors started");

    // Start API server
    let api_storage = storage.clone();
    let api_host = config.server_host.clone();
    let api_port = config.server_port;
    let api_metrics = metrics_handle.clone();
    let api_stats = stats_tracker.clone();

    info!("🌐 Starting API server on {}:{}", api_host, api_port);

    let api_handle = tokio::spawn(async move {
        if let Err(e) = api::start_server(
            api_host,
            api_port,
            api_storage,
            event_tx,
            api_metrics,
            api_stats,
        )
        .await
        {
            error!("❌ API server error: {}", e);
        }
    });

    // Setup graceful shutdown signal
    let shutdown_signal = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install CTRL+C signal handler");
        info!("🛑 Shutdown signal received, gracefully shutting down...");
    };

    // Wait for either:
    // 1. All supervisors to complete (they shouldn't unless there's an error)
    // 2. API server to crash
    // 3. Shutdown signal
    tokio::select! {
        _ = async {
            for (chain_name, handle) in supervisor_handles {
                if let Err(e) = handle.await {
                    error!("❌ Supervisor {} panicked: {}", chain_name, e);
                }
            }
        } => {
            error!("⚠️  All supervisors terminated");
        }
        _ = api_handle => {
            error!("⚠️  API server terminated");
        }
        _ = shutdown_signal => {
            info!("✅ Graceful shutdown completed");
        }
    }

    Ok(())
}
