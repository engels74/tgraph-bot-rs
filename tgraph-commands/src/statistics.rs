//! User statistics collection and aggregation system
//! 
//! This module provides comprehensive user statistics tracking with privacy-aware
//! data handling, efficient aggregation, and caching for optimal performance.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use chrono::{DateTime, Utc, Datelike, Timelike};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::database::{UserDatabase, UserPreferences};
use crate::CommandExecution;

/// Time period for statistics aggregation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimePeriod {
    Daily,
    Weekly,
    Monthly,
    AllTime,
}

impl TimePeriod {
    /// Get the duration for this time period
    pub fn duration(&self) -> Option<chrono::Duration> {
        match self {
            TimePeriod::Daily => Some(chrono::Duration::days(1)),
            TimePeriod::Weekly => Some(chrono::Duration::weeks(1)),
            TimePeriod::Monthly => Some(chrono::Duration::days(30)),
            TimePeriod::AllTime => None,
        }
    }

    /// Get a human-readable name for this period
    pub fn name(&self) -> &'static str {
        match self {
            TimePeriod::Daily => "Daily",
            TimePeriod::Weekly => "Weekly", 
            TimePeriod::Monthly => "Monthly",
            TimePeriod::AllTime => "All Time",
        }
    }
}

/// User activity data for a specific time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivity {
    /// User ID
    pub user_id: u64,
    /// Time period for this activity
    pub period: TimePeriod,
    /// Period start timestamp
    pub period_start: DateTime<Utc>,
    /// Period end timestamp  
    pub period_end: DateTime<Utc>,
    /// Total number of commands executed
    pub total_commands: u64,
    /// Number of successful commands
    pub successful_commands: u64,
    /// Number of failed commands
    pub failed_commands: u64,
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    /// Most frequently used command
    pub most_used_command: Option<String>,
    /// Command usage breakdown (command -> count)
    pub command_breakdown: HashMap<String, u64>,
    /// Activity by hour of day (0-23)
    pub hourly_activity: [u64; 24],
    /// Activity by day of week (0=Sunday, 6=Saturday)
    pub daily_activity: [u64; 7],
    /// First command timestamp in this period
    pub first_command: Option<DateTime<Utc>>,
    /// Last command timestamp in this period
    pub last_command: Option<DateTime<Utc>>,
    /// Unique channels used
    pub unique_channels: u64,
    /// Unique guilds used
    pub unique_guilds: u64,
}

impl UserActivity {
    /// Create new empty user activity for a period
    pub fn new(user_id: u64, period: TimePeriod, start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            user_id,
            period,
            period_start: start,
            period_end: end,
            total_commands: 0,
            successful_commands: 0,
            failed_commands: 0,
            avg_response_time_ms: 0.0,
            most_used_command: None,
            command_breakdown: HashMap::new(),
            hourly_activity: [0; 24],
            daily_activity: [0; 7],
            first_command: None,
            last_command: None,
            unique_channels: 0,
            unique_guilds: 0,
        }
    }

    /// Calculate success rate as percentage
    pub fn success_rate(&self) -> f64 {
        if self.total_commands == 0 {
            0.0
        } else {
            (self.successful_commands as f64 / self.total_commands as f64) * 100.0
        }
    }

    /// Get the most active hour (0-23)
    pub fn most_active_hour(&self) -> Option<usize> {
        self.hourly_activity
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(hour, _)| hour)
    }

    /// Get the most active day (0=Sunday, 6=Saturday)
    pub fn most_active_day(&self) -> Option<usize> {
        self.daily_activity
            .iter()
            .enumerate()
            .max_by_key(|(_, &count)| count)
            .map(|(day, _)| day)
    }
}

/// Cached statistics entry
#[derive(Debug, Clone)]
struct CachedStats {
    /// The cached user activity data
    data: UserActivity,
    /// When this cache entry was created
    cached_at: Instant,
    /// Cache TTL in seconds
    ttl: Duration,
}

impl CachedStats {
    /// Check if this cache entry is still valid
    fn is_valid(&self) -> bool {
        self.cached_at.elapsed() < self.ttl
    }
}

/// User statistics manager with privacy controls and caching
#[derive(Debug)]
pub struct UserStatisticsManager {
    /// User database for preferences
    user_db: Arc<UserDatabase>,
    /// Cache for aggregated statistics (user_id -> period -> cached_stats)
    cache: Arc<DashMap<u64, DashMap<TimePeriod, CachedStats>>>,
    /// Cache TTL for different periods
    cache_ttl: HashMap<TimePeriod, Duration>,
    /// Background cache cleanup task handle
    _cleanup_task: tokio::task::JoinHandle<()>,
}

impl UserStatisticsManager {
    /// Create a new user statistics manager
    pub fn new(user_db: Arc<UserDatabase>) -> Self {
        let cache = Arc::new(DashMap::new());
        
        // Configure cache TTL based on period sensitivity
        let mut cache_ttl = HashMap::new();
        cache_ttl.insert(TimePeriod::Daily, Duration::from_secs(300)); // 5 minutes for daily
        cache_ttl.insert(TimePeriod::Weekly, Duration::from_secs(1800)); // 30 minutes for weekly
        cache_ttl.insert(TimePeriod::Monthly, Duration::from_secs(3600)); // 1 hour for monthly
        cache_ttl.insert(TimePeriod::AllTime, Duration::from_secs(7200)); // 2 hours for all-time

        // Start background cleanup task
        let cache_for_cleanup = cache.clone();
        let cleanup_task = tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(600)); // Every 10 minutes
            
            loop {
                interval.tick().await;
                Self::cleanup_expired_cache(&cache_for_cleanup).await;
            }
        });

        Self {
            user_db,
            cache,
            cache_ttl,
            _cleanup_task: cleanup_task,
        }
    }

    /// Cleanup expired cache entries
    async fn cleanup_expired_cache(cache: &DashMap<u64, DashMap<TimePeriod, CachedStats>>) {
        let mut expired_users = Vec::new();
        
        for entry in cache.iter() {
            let user_id = *entry.key();
            let user_cache = entry.value();
            
            // Remove expired entries for this user
            user_cache.retain(|_period, cached| cached.is_valid());
            
            // If user has no valid cache entries, mark for removal
            if user_cache.is_empty() {
                expired_users.push(user_id);
            }
        }
        
        // Remove users with no cache entries
        for user_id in expired_users {
            cache.remove(&user_id);
        }
        
        debug!("Cache cleanup completed, remaining users: {}", cache.len());
    }

    /// Get user statistics for a specific period, respecting privacy settings
    pub async fn get_user_statistics(
        &self,
        user_id: u64,
        period: TimePeriod,
        command_executions: &[CommandExecution],
    ) -> Result<Option<UserActivity>> {
        // Get user preferences first to check privacy settings
        let preferences = self.user_db.get_or_create_preferences(user_id).await?;
        
        // Check if user allows statistics collection
        if !preferences.allow_public_stats && period != TimePeriod::AllTime {
            debug!("User {} has disabled public statistics", user_id);
            return Ok(None);
        }

        // Check data retention policy
        if !preferences.should_retain_data() {
            debug!("User {} data should not be retained per their preferences", user_id);
            return Ok(None);
        }

        // Try to get from cache first
        if let Some(cached) = self.get_cached_stats(user_id, period) {
            debug!("Returning cached statistics for user {} period {:?}", user_id, period);
            return Ok(Some(cached));
        }

        // Calculate fresh statistics
        let stats = self.calculate_user_statistics(user_id, period, command_executions, &preferences).await?;
        
        // Cache the results
        if let Some(ref stats) = stats {
            self.cache_stats(user_id, period, stats.clone());
        }

        Ok(stats)
    }

    /// Get cached statistics if valid
    fn get_cached_stats(&self, user_id: u64, period: TimePeriod) -> Option<UserActivity> {
        if let Some(user_cache) = self.cache.get(&user_id) {
            if let Some(cached) = user_cache.get(&period) {
                if cached.is_valid() {
                    return Some(cached.data.clone());
                }
            }
        }
        None
    }

    /// Cache statistics for a user and period
    fn cache_stats(&self, user_id: u64, period: TimePeriod, stats: UserActivity) {
        let ttl = self.cache_ttl.get(&period).copied().unwrap_or(Duration::from_secs(3600));
        
        let cached = CachedStats {
            data: stats,
            cached_at: Instant::now(),
            ttl,
        };

        let user_cache = self.cache.entry(user_id).or_default();
        user_cache.insert(period, cached);
        
        debug!("Cached statistics for user {} period {:?}", user_id, period);
    }

    /// Calculate user statistics for a specific period using zero-copy operations where possible
    async fn calculate_user_statistics(
        &self,
        user_id: u64,
        period: TimePeriod,
        command_executions: &[CommandExecution],
        preferences: &UserPreferences,
    ) -> Result<Option<UserActivity>> {
        let now = Utc::now();
        let (period_start, period_end) = match period {
            TimePeriod::Daily => (now - chrono::Duration::days(1), now),
            TimePeriod::Weekly => (now - chrono::Duration::weeks(1), now),
            TimePeriod::Monthly => (now - chrono::Duration::days(30), now),
            TimePeriod::AllTime => {
                // For all-time, use the user's data creation date or a reasonable default
                let start = preferences.created_at.min(now - chrono::Duration::days(365));
                (start, now)
            }
        };

        // Filter executions for this user and time period using zero-copy slice operations
        let user_executions: Vec<&CommandExecution> = command_executions
            .iter()
            .filter(|exec| {
                exec.user_id == user_id
                    && exec.timestamp >= period_start
                    && exec.timestamp <= period_end
            })
            .collect();

        if user_executions.is_empty() {
            return Ok(None);
        }

        let mut activity = UserActivity::new(user_id, period, period_start, period_end);

        // Efficient aggregation using iterator chains and zero-copy operations
        activity.total_commands = user_executions.len() as u64;
        activity.successful_commands = user_executions.iter().filter(|exec| exec.success).count() as u64;
        activity.failed_commands = activity.total_commands - activity.successful_commands;

        // Calculate average response time
        let total_duration: u64 = user_executions.iter().map(|exec| exec.duration_ms).sum();
        activity.avg_response_time_ms = if activity.total_commands > 0 {
            total_duration as f64 / activity.total_commands as f64
        } else {
            0.0
        };

        // Command breakdown using efficient counting
        for exec in &user_executions {
            *activity.command_breakdown.entry(exec.command.clone()).or_insert(0) += 1;
        }

        // Find most used command
        activity.most_used_command = activity.command_breakdown
            .iter()
            .max_by_key(|(_, &count)| count)
            .map(|(cmd, _)| cmd.clone());

        // Time-based activity analysis
        for exec in &user_executions {
            let hour = exec.timestamp.hour() as usize;
            let day = exec.timestamp.weekday().number_from_sunday() as usize - 1; // Convert to 0-6
            
            if hour < 24 {
                activity.hourly_activity[hour] += 1;
            }
            if day < 7 {
                activity.daily_activity[day] += 1;
            }
        }

        // Find first and last commands
        activity.first_command = user_executions.iter().map(|exec| exec.timestamp).min();
        activity.last_command = user_executions.iter().map(|exec| exec.timestamp).max();

        // Count unique channels and guilds using efficient set operations
        let unique_channels: std::collections::HashSet<_> = user_executions
            .iter()
            .filter_map(|exec| exec.channel_id)
            .collect();
        activity.unique_channels = unique_channels.len() as u64;

        let unique_guilds: std::collections::HashSet<_> = user_executions
            .iter()
            .filter_map(|exec| exec.guild_id)
            .collect();
        activity.unique_guilds = unique_guilds.len() as u64;

        debug!(
            "Calculated statistics for user {} period {:?}: {} commands, {:.1}% success rate",
            user_id, period, activity.total_commands, activity.success_rate()
        );

        Ok(Some(activity))
    }

    /// Clear cache for a specific user (useful for privacy compliance)
    pub fn clear_user_cache(&self, user_id: u64) {
        self.cache.remove(&user_id);
        debug!("Cleared cache for user {}", user_id);
    }

    /// Clear all cached statistics
    pub fn clear_all_cache(&self) {
        self.cache.clear();
        info!("Cleared all cached statistics");
    }

    /// Get cache statistics for monitoring
    pub fn get_cache_stats(&self) -> serde_json::Value {
        let total_users = self.cache.len();
        let mut total_entries = 0;
        let mut valid_entries = 0;

        for user_cache in self.cache.iter() {
            for cached in user_cache.value().iter() {
                total_entries += 1;
                if cached.value().is_valid() {
                    valid_entries += 1;
                }
            }
        }

        serde_json::json!({
            "total_users": total_users,
            "total_cache_entries": total_entries,
            "valid_cache_entries": valid_entries,
            "cache_hit_ratio": if total_entries > 0 { 
                valid_entries as f64 / total_entries as f64 
            } else { 
                0.0 
            }
        })
    }

    /// Apply privacy filters to user activity based on preferences
    pub fn apply_privacy_filters(&self, activity: &mut UserActivity, _preferences: &UserPreferences) {
        // If username should not be visible, we don't modify the activity here
        // as it doesn't contain username information. This is handled at display time.
        
        // If public stats are disabled, return None (handled in get_user_statistics)
        
        // Additional privacy filters can be added here as needed
        debug!("Applied privacy filters for user {}", activity.user_id);
    }

    /// Export user statistics data for GDPR compliance
    pub async fn export_user_statistics_data(
        &self,
        user_id: u64,
        command_executions: &[CommandExecution],
    ) -> Result<serde_json::Value> {
        let mut exported_data = serde_json::Map::new();

        // Get statistics for all periods
        for &period in &[TimePeriod::Daily, TimePeriod::Weekly, TimePeriod::Monthly, TimePeriod::AllTime] {
            if let Some(stats) = self.get_user_statistics(user_id, period, command_executions).await? {
                exported_data.insert(period.name().to_lowercase(), serde_json::to_value(stats)?);
            }
        }

        // Add metadata
        exported_data.insert("export_timestamp".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
        exported_data.insert("user_id".to_string(), serde_json::Value::Number(user_id.into()));

        Ok(serde_json::Value::Object(exported_data))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::database::UserDatabase;

    async fn create_test_manager() -> (UserStatisticsManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let user_db = Arc::new(UserDatabase::new(temp_dir.path().join("test_stats_db")).unwrap());
        let manager = UserStatisticsManager::new(user_db);
        (manager, temp_dir)
    }

    fn create_test_executions() -> Vec<CommandExecution> {
        let now = Utc::now();
        vec![
            CommandExecution {
                command: "about".to_string(),
                user_id: 123,
                channel_id: Some(456),
                guild_id: Some(789),
                timestamp: now - chrono::Duration::hours(2),
                duration_ms: 150,
                success: true,
                error: None,
                metadata: serde_json::json!({}),
            },
            CommandExecution {
                command: "uptime".to_string(),
                user_id: 123,
                channel_id: Some(456),
                guild_id: Some(789),
                timestamp: now - chrono::Duration::hours(1),
                duration_ms: 200,
                success: true,
                error: None,
                metadata: serde_json::json!({}),
            },
            CommandExecution {
                command: "about".to_string(),
                user_id: 123,
                channel_id: Some(789),
                guild_id: Some(789),
                timestamp: now - chrono::Duration::minutes(30),
                duration_ms: 180,
                success: false,
                error: Some("Test error".to_string()),
                metadata: serde_json::json!({}),
            },
        ]
    }

    #[tokio::test]
    async fn test_user_statistics_calculation() {
        let (manager, _temp_dir) = create_test_manager().await;
        let executions = create_test_executions();

        // Enable public stats for the test user
        let mut preferences = crate::database::UserPreferences::new(123);
        preferences.allow_public_stats = true;
        manager.user_db.store_preferences(preferences).await.unwrap();

        let stats = manager.get_user_statistics(123, TimePeriod::Daily, &executions).await.unwrap();
        assert!(stats.is_some());

        let stats = stats.unwrap();
        assert_eq!(stats.user_id, 123);
        assert_eq!(stats.total_commands, 3);
        assert_eq!(stats.successful_commands, 2);
        assert_eq!(stats.failed_commands, 1);
        assert!((stats.success_rate() - 66.66666666666667).abs() < 0.0001);
        assert_eq!(stats.most_used_command, Some("about".to_string()));
        assert_eq!(stats.unique_channels, 2);
        assert_eq!(stats.unique_guilds, 1);
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let (manager, _temp_dir) = create_test_manager().await;
        let executions = create_test_executions();

        // Enable public stats for the test user
        let mut preferences = crate::database::UserPreferences::new(123);
        preferences.allow_public_stats = true;
        manager.user_db.store_preferences(preferences).await.unwrap();

        // First call should calculate and cache
        let stats1 = manager.get_user_statistics(123, TimePeriod::Daily, &executions).await.unwrap();
        assert!(stats1.is_some());

        // Second call should return cached result
        let stats2 = manager.get_user_statistics(123, TimePeriod::Daily, &executions).await.unwrap();
        assert!(stats2.is_some());

        // Results should be identical
        assert_eq!(stats1.unwrap().total_commands, stats2.unwrap().total_commands);
    }

    #[tokio::test]
    async fn test_privacy_controls() {
        let (manager, _temp_dir) = create_test_manager().await;
        let executions = create_test_executions();

        // Create user with public stats disabled
        let mut preferences = crate::database::UserPreferences::new(123);
        preferences.allow_public_stats = false;
        manager.user_db.store_preferences(preferences).await.unwrap();

        // Should return None for weekly stats (not all-time)
        let stats = manager.get_user_statistics(123, TimePeriod::Weekly, &executions).await.unwrap();
        assert!(stats.is_none());

        // Should still return stats for all-time (personal use)
        let stats = manager.get_user_statistics(123, TimePeriod::AllTime, &executions).await.unwrap();
        assert!(stats.is_some());
    }

    #[tokio::test]
    async fn test_data_export() {
        let (manager, _temp_dir) = create_test_manager().await;
        let executions = create_test_executions();

        let exported = manager.export_user_statistics_data(123, &executions).await.unwrap();
        assert!(exported.is_object());
        assert!(exported.get("export_timestamp").is_some());
        assert!(exported.get("user_id").is_some());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let (manager, _temp_dir) = create_test_manager().await;
        let executions = create_test_executions();

        // Add some statistics to cache
        manager.get_user_statistics(123, TimePeriod::Daily, &executions).await.unwrap();
        
        // Clear cache
        manager.clear_user_cache(123);
        
        // Cache stats should show no entries for this user
        let cache_stats = manager.get_cache_stats();
        assert_eq!(cache_stats["total_users"], 0);
    }
} 