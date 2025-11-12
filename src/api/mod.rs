use crate::auth::{self, Claims, JwtConfig, LoginRequest, LoginResponse};
use crate::models::{Event, EventQuery};
use crate::stats::StatsTracker;
use crate::storage::Storage;
use axum::{
    extract::{
        ws::{Message, WebSocket},
        Query, State, WebSocketUpgrade,
    },
    http::{HeaderValue, Method, StatusCode},
    middleware,
    response::{IntoResponse, Response},
    routing::{get, post},
    Extension, Json, Router,
};
use metrics_exporter_prometheus::PrometheusHandle;
use serde_json::json;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower_http::cors::{Any, CorsLayer};
use tracing::{error, info, warn};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub storage: Storage,
    pub event_tx: broadcast::Sender<Event>,
    pub metrics_handle: PrometheusHandle,
    pub stats_tracker: StatsTracker,
}

/// Configure CORS based on environment variables
fn configure_cors() -> CorsLayer {
    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS").unwrap_or_else(|_| "*".to_string());

    if allowed_origins == "*" {
        warn!("CORS is set to allow all origins (*). This is NOT recommended for production!");
        return CorsLayer::new()
            .allow_origin(Any)
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(Any);
    }

    // Parse comma-separated origins
    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .filter_map(|origin| origin.trim().parse::<HeaderValue>().ok())
        .collect();

    if origins.is_empty() {
        warn!("No valid CORS origins configured. Falling back to permissive CORS");
        return CorsLayer::permissive();
    }

    info!("CORS configured with {} allowed origins", origins.len());

    use axum::http::header;

    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers([header::CONTENT_TYPE, header::AUTHORIZATION, header::ACCEPT])
        .allow_credentials(true)
}

/// Start the API server
pub async fn start_server(
    host: String,
    port: u16,
    storage: Storage,
    event_tx: broadcast::Sender<Event>,
    metrics_handle: PrometheusHandle,
    stats_tracker: StatsTracker,
) -> anyhow::Result<()> {
    let state = Arc::new(AppState {
        storage,
        event_tx,
        metrics_handle,
        stats_tracker,
    });

    // Initialize JWT config
    let jwt_config = JwtConfig::from_env();

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/health/detailed", get(health_check_detailed))
        .route("/chains", get(get_chains))
        .route("/metrics", get(metrics_handler))
        .route("/login", post(login));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/events", get(get_recent_activity))
        .route("/ws", get(websocket_handler))
        .route("/stats", get(get_stats))
        .route("/chains/status", get(get_chains_status))
        .layer(middleware::from_fn(jwt_middleware));

    // Configure CORS
    let cors = configure_cors();

    let app = Router::new()
        .merge(public_routes)
        .merge(protected_routes)
        .layer(Extension(jwt_config))
        .layer(cors)
        .with_state(state);

    let addr = format!("{}:{}", host, port);
    info!("Starting API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

/// JWT middleware to inject JWT config into request extensions
async fn jwt_middleware(
    Extension(jwt_config): Extension<JwtConfig>,
    mut request: axum::http::Request<axum::body::Body>,
    next: middleware::Next,
) -> Result<Response, StatusCode> {
    request.extensions_mut().insert(jwt_config);
    Ok(next.run(request).await)
}

/// Login endpoint
async fn login(
    Extension(jwt_config): Extension<JwtConfig>,
    Json(credentials): Json<LoginRequest>,
) -> Result<Json<LoginResponse>, auth::AuthError> {
    // Validate credentials
    if !auth::validate_credentials(&credentials.username, &credentials.password) {
        return Err(auth::AuthError::WrongCredentials);
    }

    // Create JWT token
    let token = jwt_config.create_token(&credentials.username)?;

    // Calculate expiration time
    let expires_at = (chrono::Utc::now()
        + chrono::Duration::hours(jwt_config.token_expiration_hours))
    .to_rfc3339();

    Ok(Json(LoginResponse { token, expires_at }))
}

/// Health check endpoint (simple)
async fn health_check() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "service": "erc8004-indexer"
    }))
}

/// Advanced health check (with DB and multi-chain status)
async fn health_check_detailed(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut overall_status = "healthy";
    let mut checks = serde_json::Map::new();

    // Check database connectivity
    match state.storage.get_enabled_chains().await {
        Ok(chains) => {
            // Check each chain status
            let mut chains_status = serde_json::Map::new();
            let mut failed_count = 0;
            let mut stalled_count = 0;

            for chain in &chains {
                let chain_status = chain.status.as_deref().unwrap_or("unknown");

                if chain_status == "failed" {
                    failed_count += 1;
                    overall_status = "degraded";
                } else if chain_status == "stalled" {
                    stalled_count += 1;
                    if overall_status == "healthy" {
                        overall_status = "degraded";
                    }
                }

                chains_status.insert(
                    chain.name.clone(),
                    json!({
                        "chain_id": chain.chain_id,
                        "status": chain_status,
                        "last_synced_block": chain.last_synced_block,
                        "last_sync_time": chain.last_sync_time,
                        "total_events": chain.total_events_indexed,
                        "errors_last_hour": chain.errors_last_hour,
                        "error_message": chain.error_message
                    }),
                );
            }

            checks.insert(
                "database".to_string(),
                json!({
                    "status": "healthy",
                    "total_chains": chains.len(),
                    "failed_chains": failed_count,
                    "stalled_chains": stalled_count
                }),
            );

            checks.insert("chains".to_string(), json!(chains_status));
        }
        Err(e) => {
            overall_status = "unhealthy";
            checks.insert(
                "database".to_string(),
                json!({
                    "status": "unhealthy",
                    "error": e.to_string()
                }),
            );
        }
    }

    // Check cache
    let (cache_size, cache_max) = state.storage.cache_stats();
    let cache_utilization = (cache_size as f64 / cache_max as f64) * 100.0;

    let cache_status = if cache_utilization > 90.0 {
        "warning"
    } else {
        "healthy"
    };

    checks.insert(
        "cache".to_string(),
        json!({
            "status": cache_status,
            "size": cache_size,
            "max_size": cache_max,
            "utilization_percent": format!("{:.2}", cache_utilization)
        }),
    );

    let health = json!({
        "status": overall_status,
        "service": "erc8004-indexer",
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "checks": checks
    });

    // Overall status
    let status_code = match overall_status {
        "healthy" => StatusCode::OK,
        "degraded" => StatusCode::OK, // Still operational
        _ => StatusCode::SERVICE_UNAVAILABLE,
    };

    (status_code, Json(health))
}

/// Prometheus metrics endpoint
async fn metrics_handler(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    // Render metrics from the handle
    let metrics = state.metrics_handle.render();
    (StatusCode::OK, metrics)
}

/// Get recent activity (REST endpoint)
async fn get_recent_activity(
    claims: Claims,
    State(state): State<Arc<AppState>>,
    Query(query): Query<EventQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("User '{}' requested events", claims.sub);

    // Get total count for pagination metadata
    let total = state.storage.count_events(query.clone()).await?;

    // Get events for current page
    let events = state.storage.get_recent_events(query.clone()).await?;

    // Get category statistics only if requested (to avoid unnecessary DB queries)
    let stats = if query.include_stats {
        Some(
            state
                .storage
                .get_category_stats(query.parse_chain_ids())
                .await?,
        )
    } else {
        None
    };

    // Calculate pagination metadata
    let limit = query.limit.unwrap_or(1000);
    let offset = query.offset.unwrap_or(0);
    let has_more = (offset + events.len() as i64) < total;
    let next_offset = if has_more { Some(offset + limit) } else { None };

    // Build response
    let mut response = json!({
        "success": true,
        "count": events.len(),
        "total": total,
        "pagination": {
            "offset": offset,
            "limit": limit,
            "has_more": has_more,
            "next_offset": next_offset
        },
        "events": events
    });

    // Add chain info if filtering by specific chains
    if let Some(chain_ids) = query.parse_chain_ids() {
        if !chain_ids.is_empty() {
            response["chains_queried"] = json!(chain_ids);
        }
    } else {
        response["chains_queried"] = json!("all");
    }

    // Add stats to response only if they were requested
    if let Some(category_stats) = stats {
        response["stats"] = json!(category_stats);
    }

    Ok(Json(response))
}

/// Get indexer statistics (DEPRECATED - use /health/detailed or /chains instead)
async fn get_stats(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    warn!("User '{}' requested deprecated /stats endpoint", claims.sub);

    // Get multi-chain stats
    let chains = state.storage.get_enabled_chains().await?;
    let (cache_size, cache_max) = state.storage.cache_stats();

    Ok(Json(json!({
        "deprecated": true,
        "message": "This endpoint is deprecated. Use /health/detailed or /chains for multi-chain statistics.",
        "alternatives": {
            "/health/detailed": "Complete health check with per-chain status",
            "/chains": "List of all chains with indexing status"
        },
        "legacy_data": {
            "cache_size": cache_size,
            "cache_max_size": cache_max,
            "total_chains": chains.len(),
            "chains": chains.iter().map(|c| json!({
                "name": c.name,
                "chain_id": c.chain_id,
                "last_synced_block": c.last_synced_block,
                "total_events": c.total_events_indexed
            })).collect::<Vec<_>>()
        }
    })))
}

/// WebSocket handler
async fn websocket_handler(
    claims: Claims,
    ws: WebSocketUpgrade,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    info!("User '{}' connected to WebSocket", claims.sub);
    ws.on_upgrade(|socket| handle_websocket(socket, state))
}

/// Handle WebSocket connection
async fn handle_websocket(socket: WebSocket, state: Arc<AppState>) {
    let (mut sender, mut receiver) = socket.split();

    let mut event_rx = state.event_tx.subscribe();

    // Send welcome message
    let welcome = json!({
        "type": "connected",
        "message": "Connected to ERC-8004 event stream"
    });

    if let Ok(msg) = serde_json::to_string(&welcome) {
        if sender.send(Message::Text(msg)).await.is_err() {
            return;
        }
    }

    // Spawn task to forward events to WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            let msg = json!({
                "type": "event",
                "data": event
            });

            if let Ok(text) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(text)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (mostly for keep-alive pings)
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Ping(_) => {
                    // Pong is automatically sent by axum
                }
                _ => {}
            }
        }
    });

    // Wait for either task to complete
    tokio::select! {
        _ = &mut send_task => {
            recv_task.abort();
        }
        _ = &mut recv_task => {
            send_task.abort();
        }
    }

    info!("WebSocket connection closed");
}

/// Broadcast an event to all connected WebSocket clients
#[allow(dead_code)]
pub fn broadcast_event(event_tx: &broadcast::Sender<Event>, event: Event) {
    if let Err(e) = event_tx.send(event) {
        error!("Failed to broadcast event: {}", e);
    }
}

/// API error wrapper
struct ApiError(anyhow::Error);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        error!("API error: {}", self.0);

        let body = Json(json!({
            "success": false,
            "error": self.0.to_string()
        }));

        (StatusCode::INTERNAL_SERVER_ERROR, body).into_response()
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

// Required for axum WebSocket
use futures::stream::StreamExt;
use futures::SinkExt;

/// GET /chains - List all enabled chains with status
async fn get_chains(
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    // Get all enabled chains from database
    let chains = state.storage.get_enabled_chains().await?;

    // Calculate overall status
    let total_chains = chains.len();
    let healthy_chains = chains
        .iter()
        .filter(|c| c.status.as_deref() == Some("active") || c.status.as_deref() == Some("syncing"))
        .count();
    let failed_chains = chains
        .iter()
        .filter(|c| c.status.as_deref() == Some("failed"))
        .count();

    let overall_status = if failed_chains > 0 {
        "degraded"
    } else if healthy_chains == total_chains {
        "healthy"
    } else {
        "syncing"
    };

    Ok(Json(json!({
        "status": overall_status,
        "total_chains": total_chains,
        "healthy_chains": healthy_chains,
        "failed_chains": failed_chains,
        "chains": chains
    })))
}

/// GET /chains/status - Get detailed status for all chains with monitoring data
async fn get_chains_status(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("User '{}' requested chains status", claims.sub);

    // Get all enabled chains from database
    let chains = state.storage.get_enabled_chains().await?;

    let mut chain_statuses = vec![];

    for chain in chains {
        // Get current block from stats tracker
        let current_block = state.stats_tracker.get_current_block(chain.chain_id);

        // Get indexer block from database
        let indexer_block = chain.last_synced_block.unwrap_or(0);

        // Calculate blocks behind
        let blocks_behind = if let Some(current) = current_block {
            current.saturating_sub(indexer_block)
        } else {
            0
        };

        // Get polling rate
        let polling_rate = state.stats_tracker.get_polling_rate(chain.chain_id);

        // Get event counts by type
        let event_counts = state
            .storage
            .get_event_counts_by_type(chain.chain_id)
            .await
            .unwrap_or_default();

        chain_statuses.push(json!({
            "chain_id": chain.chain_id,
            "name": chain.name,
            "status": chain.status.unwrap_or_else(|| "unknown".to_string()),
            "blocks": {
                "current": current_block,
                "indexed": indexer_block,
                "behind": blocks_behind
            },
            "polling": {
                "rate_per_minute": format!("{:.2}", polling_rate)
            },
            "events": {
                "total": chain.total_events_indexed.unwrap_or(0),
                "by_type": event_counts
            },
            "last_sync_time": chain.last_sync_time
        }));
    }

    Ok(Json(json!({
        "success": true,
        "timestamp": chrono::Utc::now().to_rfc3339(),
        "chains": chain_statuses
    })))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Event, EventData, EventType, RegisteredData};
    use chrono::Utc;

    fn create_test_event() -> Event {
        Event {
            id: Some(1),
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
            created_at: Some(Utc::now()),
        }
    }

    #[test]
    fn test_configure_cors_permissive() {
        // Test that CORS allows all origins when set to *
        std::env::set_var("CORS_ALLOWED_ORIGINS", "*");
        let cors = configure_cors();
        // Can't easily test CorsLayer behavior, but we verify it doesn't panic
        drop(cors);
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_configure_cors_specific_origins() {
        // Test that CORS parses multiple origins
        std::env::set_var(
            "CORS_ALLOWED_ORIGINS",
            "http://localhost:3000,https://example.com",
        );
        let cors = configure_cors();
        drop(cors);
        std::env::remove_var("CORS_ALLOWED_ORIGINS");
    }

    #[test]
    fn test_api_error_conversion() {
        // Test that errors can be converted to ApiError
        let error = anyhow::anyhow!("Test error");
        let api_error: ApiError = error.into();

        // Verify the error can be converted to response
        let response = api_error.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[test]
    fn test_broadcast_event() {
        // Test that broadcast_event doesn't panic
        let (tx, _rx) = broadcast::channel::<Event>(10);
        let event = create_test_event();

        broadcast_event(&tx, event);
        // If we get here without panic, the test passes
    }

    #[test]
    fn test_event_broadcast_channel() {
        // Test that broadcast channel can be created for events
        let (tx, rx1) = broadcast::channel::<Event>(10);
        let rx2 = tx.subscribe();

        // Send an event
        let event = create_test_event();
        tx.send(event.clone()).unwrap();

        // Both receivers should get the event
        // Note: We can't actually receive in sync test, but we verify channel creation works
        drop(rx1);
        drop(rx2);
    }
}
