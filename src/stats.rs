use dashmap::DashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

/// Polling statistics tracker for all chains
#[derive(Clone)]
pub struct StatsTracker {
    stats: Arc<DashMap<u64, ChainStats>>, // key: chain_id
}

impl Default for StatsTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl StatsTracker {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(DashMap::new()),
        }
    }

    /// Record a polling event for a specific chain
    pub fn record_poll(&self, chain_id: u64) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        self.stats
            .entry(chain_id)
            .or_insert_with(ChainStats::new)
            .record_poll(now);
    }

    /// Update current block for a chain
    pub fn update_current_block(&self, chain_id: u64, block: u64) {
        self.stats
            .entry(chain_id)
            .or_insert_with(ChainStats::new)
            .update_current_block(block);
    }

    /// Get polling rate (polls per minute) for a specific chain
    pub fn get_polling_rate(&self, chain_id: u64) -> f64 {
        self.stats
            .get(&chain_id)
            .map(|stats| stats.get_polling_rate())
            .unwrap_or(0.0)
    }

    /// Get current block for a chain
    pub fn get_current_block(&self, chain_id: u64) -> Option<u64> {
        self.stats
            .get(&chain_id)
            .and_then(|stats| stats.current_block)
    }

    /// Get all stats for a chain
    #[allow(dead_code)]
    pub fn get_chain_stats(&self, chain_id: u64) -> Option<ChainStatsSnapshot> {
        self.stats.get(&chain_id).map(|stats| stats.snapshot())
    }
}

/// Statistics for a single chain
struct ChainStats {
    poll_timestamps: Vec<u64>, // milliseconds since epoch
    current_block: Option<u64>,
}

impl ChainStats {
    fn new() -> Self {
        Self {
            poll_timestamps: Vec::new(),
            current_block: None,
        }
    }

    /// Record a polling event
    fn record_poll(&mut self, timestamp: u64) {
        // Add new timestamp
        self.poll_timestamps.push(timestamp);

        // Remove timestamps older than 1 minute (60000 ms)
        let cutoff = timestamp.saturating_sub(60_000);
        self.poll_timestamps.retain(|&ts| ts >= cutoff);
    }

    /// Update current block
    fn update_current_block(&mut self, block: u64) {
        self.current_block = Some(block);
    }

    /// Calculate polling rate (polls per minute)
    fn get_polling_rate(&self) -> f64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let cutoff = now.saturating_sub(60_000);
        let polls_last_minute = self
            .poll_timestamps
            .iter()
            .filter(|&&ts| ts >= cutoff)
            .count();

        polls_last_minute as f64
    }

    /// Create a snapshot of current stats
    #[allow(dead_code)]
    fn snapshot(&self) -> ChainStatsSnapshot {
        ChainStatsSnapshot {
            polling_rate: self.get_polling_rate(),
            current_block: self.current_block,
        }
    }
}

/// Snapshot of chain statistics for API responses
#[derive(Debug, Clone, serde::Serialize)]
#[allow(dead_code)]
pub struct ChainStatsSnapshot {
    pub polling_rate: f64,
    pub current_block: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::time::Duration as StdDuration;

    #[test]
    fn test_stats_tracker_creation() {
        let tracker = StatsTracker::new();
        assert_eq!(tracker.get_polling_rate(11155111), 0.0);
        assert_eq!(tracker.get_current_block(11155111), None);
    }

    #[test]
    fn test_record_poll() {
        let tracker = StatsTracker::new();

        // Record 5 polls
        for _ in 0..5 {
            tracker.record_poll(11155111);
            thread::sleep(StdDuration::from_millis(10));
        }

        // Should have recorded 5 polls
        let rate = tracker.get_polling_rate(11155111);
        assert!((4.0..=6.0).contains(&rate)); // Allow some variance
    }

    #[test]
    fn test_update_current_block() {
        let tracker = StatsTracker::new();

        tracker.update_current_block(11155111, 1000);
        assert_eq!(tracker.get_current_block(11155111), Some(1000));

        tracker.update_current_block(11155111, 1001);
        assert_eq!(tracker.get_current_block(11155111), Some(1001));
    }

    #[test]
    fn test_multiple_chains() {
        let tracker = StatsTracker::new();

        tracker.record_poll(11155111);
        tracker.record_poll(84532);
        tracker.update_current_block(11155111, 1000);
        tracker.update_current_block(84532, 2000);

        assert_eq!(tracker.get_current_block(11155111), Some(1000));
        assert_eq!(tracker.get_current_block(84532), Some(2000));
    }

    #[test]
    fn test_chain_stats_snapshot() {
        let tracker = StatsTracker::new();

        tracker.record_poll(11155111);
        tracker.update_current_block(11155111, 5000);

        let snapshot = tracker.get_chain_stats(11155111).unwrap();
        assert_eq!(snapshot.current_block, Some(5000));
        assert!(snapshot.polling_rate >= 0.0);
    }
}
