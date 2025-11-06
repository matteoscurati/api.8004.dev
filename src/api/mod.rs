use crate::auth::{self, Claims, JwtConfig, LoginRequest, LoginResponse};
use crate::models::{Event, EventQuery};
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
}

/// Configure CORS based on environment variables
fn configure_cors() -> CorsLayer {
    let allowed_origins = std::env::var("CORS_ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "*".to_string());

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
        .filter_map(|origin| {
            origin.trim().parse::<HeaderValue>().ok()
        })
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
        .allow_headers([
            header::CONTENT_TYPE,
            header::AUTHORIZATION,
            header::ACCEPT,
        ])
        .allow_credentials(true)
}

/// Start the API server
pub async fn start_server(
    host: String,
    port: u16,
    storage: Storage,
    metrics_handle: PrometheusHandle,
) -> anyhow::Result<()> {
    let (event_tx, _) = broadcast::channel::<Event>(1000);

    let state = Arc::new(AppState {
        storage,
        event_tx,
        metrics_handle,
    });

    // Initialize JWT config
    let jwt_config = JwtConfig::from_env();

    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/", get(health_check))
        .route("/health", get(health_check))
        .route("/health/detailed", get(health_check_detailed))
        .route("/metrics", get(metrics_handler))
        .route("/login", post(login));

    // Protected routes (authentication required)
    let protected_routes = Router::new()
        .route("/events", get(get_recent_activity))
        .route("/ws", get(websocket_handler))
        .route("/stats", get(get_stats))
        .layer(Extension(jwt_config.clone()))
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

/// Advanced health check (with DB and RPC checks)
async fn health_check_detailed(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let mut overall_status = "healthy";
    let mut checks = serde_json::Map::new();

    // Check database
    match state.storage.get_last_synced_block().await {
        Ok(block) => {
            checks.insert(
                "database".to_string(),
                json!({
                    "status": "healthy",
                    "last_synced_block": block
                }),
            );
        }
        Err(e) => {
            overall_status = "degraded";
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
    let events = state.storage.get_recent_events(query).await?;

    Ok(Json(json!({
        "success": true,
        "count": events.len(),
        "events": events
    })))
}

/// Get indexer statistics
async fn get_stats(
    claims: Claims,
    State(state): State<Arc<AppState>>,
) -> Result<Json<serde_json::Value>, ApiError> {
    info!("User '{}' requested stats", claims.sub);
    let last_block = state.storage.get_last_synced_block().await.unwrap_or(0);
    let (cache_size, cache_max) = state.storage.cache_stats();

    Ok(Json(json!({
        "last_synced_block": last_block,
        "cache_size": cache_size,
        "cache_max_size": cache_max
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
