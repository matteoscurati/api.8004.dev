use metrics::{counter, gauge, histogram};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::time::Instant;

/// Initialize Prometheus metrics exporter
pub fn init_metrics() -> PrometheusHandle {
    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_request_duration_seconds".to_string()),
            &[0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0],
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

/// Record an event being indexed
pub fn record_event_indexed(event_type: &str, contract: &str) {
    counter!("events_indexed_total", "event_type" => event_type.to_string(), "contract" => contract.to_string()).increment(1);
}

/// Record a block being synced
pub fn record_block_synced(block_number: u64) {
    gauge!("last_synced_block").set(block_number as f64);
    counter!("blocks_synced_total").increment(1);
}

/// Record RPC request
pub fn record_rpc_request(method: &str, success: bool) {
    let status = if success { "success" } else { "error" };
    counter!("rpc_requests_total", "method" => method.to_string(), "status" => status.to_string()).increment(1);
}

/// Record HTTP request duration
pub fn record_http_request(method: &str, path: &str, status: u16, duration: std::time::Duration) {
    histogram!(
        "http_request_duration_seconds",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    )
    .record(duration.as_secs_f64());

    counter!(
        "http_requests_total",
        "method" => method.to_string(),
        "path" => path.to_string(),
        "status" => status.to_string()
    )
    .increment(1);
}

/// Record database query
pub fn record_db_query(query_type: &str, duration: std::time::Duration) {
    histogram!("db_query_duration_seconds", "query_type" => query_type.to_string())
        .record(duration.as_secs_f64());

    counter!("db_queries_total", "query_type" => query_type.to_string()).increment(1);
}

/// Record cache statistics
pub fn record_cache_stats(size: usize, max_size: usize) {
    gauge!("cache_size").set(size as f64);
    gauge!("cache_max_size").set(max_size as f64);
    gauge!("cache_utilization").set((size as f64 / max_size as f64) * 100.0);
}

/// Timer helper for measuring durations
pub struct Timer {
    start: Instant,
}

impl Timer {
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for Timer {
    fn default() -> Self {
        Self::new()
    }
}
