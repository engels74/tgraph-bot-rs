//! DM throttling functionality to prevent abuse of direct message features

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use tracing::debug;

/// DM throttling manager to prevent abuse
#[derive(Debug)]
pub struct DmThrottleManager {
    /// Last DM time for each user ID
    last_dm_times: Arc<RwLock<HashMap<u64, DateTime<Utc>>>>,
    /// Minimum time between DMs for the same user (in seconds)
    throttle_duration: Duration,
}

impl DmThrottleManager {
    /// Create a new DM throttle manager
    pub fn new(throttle_duration: Duration) -> Self {
        Self {
            last_dm_times: Arc::new(RwLock::new(HashMap::new())),
            throttle_duration,
        }
    }

    /// Check if a user can receive a DM now
    pub async fn can_send_dm(&self, user_id: u64) -> bool {
        let now = Utc::now();
        let last_dm_times = self.last_dm_times.read().await;
        
        match last_dm_times.get(&user_id) {
            Some(last_time) => {
                let elapsed = now - *last_time;
                elapsed.num_seconds() >= self.throttle_duration.as_secs() as i64
            }
            None => true, // First DM for this user
        }
    }

    /// Record that a DM was sent to a user
    pub async fn record_dm_sent(&self, user_id: u64) {
        let mut last_dm_times = self.last_dm_times.write().await;
        last_dm_times.insert(user_id, Utc::now());
        debug!("Recorded DM sent to user {} at {}", user_id, Utc::now());
    }

    /// Get remaining throttle time for a user (in seconds)
    pub async fn get_remaining_throttle(&self, user_id: u64) -> Option<Duration> {
        let now = Utc::now();
        let last_dm_times = self.last_dm_times.read().await;
        
        if let Some(last_time) = last_dm_times.get(&user_id) {
            let elapsed = now - *last_time;
            let elapsed_secs = elapsed.num_seconds() as u64;
            
            if elapsed_secs < self.throttle_duration.as_secs() {
                let remaining = self.throttle_duration.as_secs() - elapsed_secs;
                return Some(Duration::from_secs(remaining));
            }
        }
        
        None
    }

    /// Clean up old entries to prevent memory leaks
    pub async fn cleanup_old_entries(&self) {
        let cutoff = Utc::now() - chrono::Duration::seconds(self.throttle_duration.as_secs() as i64 * 2);
        let mut last_dm_times = self.last_dm_times.write().await;
        
        let initial_count = last_dm_times.len();
        last_dm_times.retain(|_, timestamp| *timestamp > cutoff);
        let cleaned_count = initial_count - last_dm_times.len();
        
        if cleaned_count > 0 {
            debug!("Cleaned up {} old DM throttle entries", cleaned_count);
        }
    }

    /// Clear throttle data for a specific user (for GDPR compliance)
    pub async fn clear_user_throttle(&self, user_id: u64) {
        let mut last_dm_times = self.last_dm_times.write().await;
        if last_dm_times.remove(&user_id).is_some() {
            debug!("Cleared DM throttle data for user {} (GDPR compliance)", user_id);
        }
    }
} 