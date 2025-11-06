use axum::{
    body::Body,
    http::{Request, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    Json,
};
use dashmap::DashMap;
use serde_json::json;
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::warn;

/// Simple in-memory rate limiter
#[derive(Clone)]
pub struct RateLimiter {
    requests: Arc<DashMap<IpAddr, Vec<Instant>>>,
    max_requests: usize,
    window: Duration,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window_secs: u64) -> Self {
        Self {
            requests: Arc::new(DashMap::new()),
            max_requests,
            window: Duration::from_secs(window_secs),
        }
    }

    /// Check if request is allowed
    pub fn check_rate_limit(&self, ip: IpAddr) -> bool {
        let now = Instant::now();
        let mut entry = self.requests.entry(ip).or_insert_with(Vec::new);

        // Remove old entries outside the time window
        entry.retain(|&timestamp| now.duration_since(timestamp) < self.window);

        // Check if limit exceeded
        if entry.len() >= self.max_requests {
            warn!("Rate limit exceeded for IP: {}", ip);
            return false;
        }

        // Add current request
        entry.push(now);
        true
    }

    /// Cleanup old entries periodically
    pub fn cleanup(&self) {
        let now = Instant::now();
        self.requests.retain(|_, timestamps| {
            timestamps.retain(|&ts| now.duration_since(ts) < self.window);
            !timestamps.is_empty()
        });
    }
}

/// Rate limit middleware
pub async fn rate_limit_middleware(
    req: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    // Get rate limiter from extensions
    let rate_limiter = req
        .extensions()
        .get::<RateLimiter>()
        .cloned()
        .ok_or(StatusCode::INTERNAL_SERVER_ERROR)?;

    // Extract IP address
    let ip = extract_ip(&req).ok_or(StatusCode::BAD_REQUEST)?;

    // Check rate limit
    if !rate_limiter.check_rate_limit(ip) {
        return Ok((
            StatusCode::TOO_MANY_REQUESTS,
            Json(json!({
                "error": "Rate limit exceeded. Please try again later."
            })),
        )
            .into_response());
    }

    Ok(next.run(req).await)
}

/// Extract IP address from request
fn extract_ip(req: &Request<Body>) -> Option<IpAddr> {
    // Check X-Forwarded-For header (if behind proxy)
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(forwarded_str) = forwarded.to_str() {
            if let Some(ip_str) = forwarded_str.split(',').next() {
                if let Ok(ip) = ip_str.trim().parse::<IpAddr>() {
                    return Some(ip);
                }
            }
        }
    }

    // Check X-Real-IP header
    if let Some(real_ip) = req.headers().get("x-real-ip") {
        if let Ok(ip_str) = real_ip.to_str() {
            if let Ok(ip) = ip_str.parse::<IpAddr>() {
                return Some(ip);
            }
        }
    }

    // Fallback to connection IP (won't work behind proxy)
    // This would need ConnectInfo extractor from axum
    None
}

/// Spawn cleanup task
pub fn spawn_cleanup_task(rate_limiter: RateLimiter) {
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            rate_limiter.cleanup();
        }
    });
}
