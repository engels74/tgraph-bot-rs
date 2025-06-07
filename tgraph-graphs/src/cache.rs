//! Data caching layer with TTL for graph generation performance optimization

use moka::future::Cache;
use std::collections::HashMap;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, instrument};
use tgraph_common::Result;
use chrono::{NaiveDate, NaiveDateTime};

use crate::{
    DayOfWeekDataPoint, HourlyDataPoint, MonthlyDataPoint, PlayCountDataPoint,
    TopPlatformDataPoint, TopUserDataPoint,
};

/// Configuration for the cache system
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries in cache
    pub max_capacity: u64,
    /// Time-to-live for cache entries
    pub ttl: Duration,
    /// Time-to-idle for cache entries (unused entries expire)
    pub tti: Option<Duration>,
    /// Enable background refresh of frequently accessed items
    pub enable_background_refresh: bool,
    /// Refresh threshold - items accessed more than this get background refresh
    pub refresh_threshold: u32,
    /// Background refresh interval
    pub refresh_interval: Duration,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,
            ttl: Duration::from_secs(3600), // 1 hour
            tti: Some(Duration::from_secs(1800)), // 30 minutes idle
            enable_background_refresh: true,
            refresh_threshold: 5,
            refresh_interval: Duration::from_secs(600), // 10 minutes
        }
    }
}

/// Graph type identifier for cache keys
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum GraphTypeKey {
    DailyPlayCount,
    DayOfWeek,
    HourlyDistribution,
    MonthlyTrends,
    TopPlatforms,
    TopUsers,
}

impl fmt::Display for GraphTypeKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphTypeKey::DailyPlayCount => write!(f, "daily_play_count"),
            GraphTypeKey::DayOfWeek => write!(f, "day_of_week"),
            GraphTypeKey::HourlyDistribution => write!(f, "hourly_distribution"),
            GraphTypeKey::MonthlyTrends => write!(f, "monthly_trends"),
            GraphTypeKey::TopPlatforms => write!(f, "top_platforms"),
            GraphTypeKey::TopUsers => write!(f, "top_users"),
        }
    }
}

/// Cache key for aggregated data with all relevant parameters
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheKey {
    /// Type of graph/aggregation
    pub graph_type: GraphTypeKey,
    /// Date range filters
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    /// Year range filters (for monthly trends)
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
    /// Result limit (for top platforms/users)
    pub limit: Option<usize>,
    /// Additional parameters hash for complex filters
    pub params_hash: u64,
}

impl CacheKey {
    /// Create a cache key for daily play count data
    pub fn daily_play_count(start_date: Option<NaiveDate>, end_date: Option<NaiveDate>) -> Self {
        Self {
            graph_type: GraphTypeKey::DailyPlayCount,
            start_date,
            end_date,
            start_year: None,
            end_year: None,
            limit: None,
            params_hash: 0,
        }
    }

    /// Create a cache key for day of week data
    pub fn day_of_week() -> Self {
        Self {
            graph_type: GraphTypeKey::DayOfWeek,
            start_date: None,
            end_date: None,
            start_year: None,
            end_year: None,
            limit: None,
            params_hash: 0,
        }
    }

    /// Create a cache key for hourly distribution data
    pub fn hourly_distribution() -> Self {
        Self {
            graph_type: GraphTypeKey::HourlyDistribution,
            start_date: None,
            end_date: None,
            start_year: None,
            end_year: None,
            limit: None,
            params_hash: 0,
        }
    }

    /// Create a cache key for monthly trends data
    pub fn monthly_trends(start_year: Option<i32>, end_year: Option<i32>) -> Self {
        Self {
            graph_type: GraphTypeKey::MonthlyTrends,
            start_date: None,
            end_date: None,
            start_year,
            end_year,
            limit: None,
            params_hash: 0,
        }
    }

    /// Create a cache key for top platforms data
    pub fn top_platforms(limit: Option<usize>) -> Self {
        Self {
            graph_type: GraphTypeKey::TopPlatforms,
            start_date: None,
            end_date: None,
            start_year: None,
            end_year: None,
            limit,
            params_hash: 0,
        }
    }

    /// Create a cache key for top users data
    pub fn top_users(limit: Option<usize>) -> Self {
        Self {
            graph_type: GraphTypeKey::TopUsers,
            start_date: None,
            end_date: None,
            start_year: None,
            end_year: None,
            limit,
            params_hash: 0,
        }
    }

    /// Add parameters hash for complex filtering scenarios
    pub fn with_params_hash(mut self, params: &impl Hash) -> Self {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        params.hash(&mut hasher);
        self.params_hash = hasher.finish();
        self
    }
}

impl fmt::Display for CacheKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:", self.graph_type)?;
        
        if let (Some(start), Some(end)) = (self.start_date, self.end_date) {
            write!(f, "{}_{}_", start.format("%Y%m%d"), end.format("%Y%m%d"))?;
        } else if let Some(start) = self.start_date {
            write!(f, "{}_", start.format("%Y%m%d"))?;
        } else if let Some(end) = self.end_date {
            write!(f, "_{}_", end.format("%Y%m%d"))?;
        }

        if let (Some(start_year), Some(end_year)) = (self.start_year, self.end_year) {
            write!(f, "{}_{}:", start_year, end_year)?;
        } else if let Some(start_year) = self.start_year {
            write!(f, "{}_:", start_year)?;
        } else if let Some(end_year) = self.end_year {
            write!(f, "_{}_:", end_year)?;
        }

        if let Some(limit) = self.limit {
            write!(f, "limit_{}_", limit)?;
        }

        if self.params_hash != 0 {
            write!(f, "hash_{}", self.params_hash)?;
        }

        Ok(())
    }
}

/// Cached data variants for different graph types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CachedData {
    DailyPlayCount(Vec<PlayCountDataPoint>),
    DayOfWeek(Vec<DayOfWeekDataPoint>),
    HourlyDistribution(Vec<HourlyDataPoint>),
    MonthlyTrends(Vec<MonthlyDataPoint>),
    TopPlatforms(Vec<TopPlatformDataPoint>),
    TopUsers(Vec<TopUserDataPoint>),
}

/// Cache entry with metadata
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: CachedData,
    pub created_at: NaiveDateTime,
    pub access_count: Arc<AtomicU64>,
    pub last_accessed: Arc<RwLock<NaiveDateTime>>,
}

impl CacheEntry {
    pub fn new(data: CachedData) -> Self {
        let now = chrono::Utc::now().naive_utc();
        Self {
            data,
            created_at: now,
            access_count: Arc::new(AtomicU64::new(1)),
            last_accessed: Arc::new(RwLock::new(now)),
        }
    }

    pub async fn mark_accessed(&self) {
        self.access_count.fetch_add(1, Ordering::Relaxed);
        let mut last_accessed = self.last_accessed.write().await;
        *last_accessed = chrono::Utc::now().naive_utc();
    }

    pub fn access_count(&self) -> u64 {
        self.access_count.load(Ordering::Relaxed)
    }
}

/// Cache performance metrics
#[derive(Debug, Default)]
pub struct CacheMetrics {
    pub hits: AtomicU64,
    pub misses: AtomicU64,
    pub evictions: AtomicU64,
    pub background_refreshes: AtomicU64,
    pub invalidations: AtomicU64,
}

impl CacheMetrics {
    pub fn record_hit(&self) {
        self.hits.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_miss(&self) {
        self.misses.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_eviction(&self) {
        self.evictions.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_background_refresh(&self) {
        self.background_refreshes.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_invalidation(&self) {
        self.invalidations.fetch_add(1, Ordering::Relaxed);
    }

    pub fn hit_rate(&self) -> f64 {
        let hits = self.hits.load(Ordering::Relaxed) as f64;
        let total = hits + self.misses.load(Ordering::Relaxed) as f64;
        if total > 0.0 {
            hits / total
        } else {
            0.0
        }
    }

    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        stats.insert("hits".to_string(), self.hits.load(Ordering::Relaxed));
        stats.insert("misses".to_string(), self.misses.load(Ordering::Relaxed));
        stats.insert("evictions".to_string(), self.evictions.load(Ordering::Relaxed));
        stats.insert("background_refreshes".to_string(), self.background_refreshes.load(Ordering::Relaxed));
        stats.insert("invalidations".to_string(), self.invalidations.load(Ordering::Relaxed));
        stats
    }
}

/// Main cache manager for graph data
pub struct GraphDataCache {
    cache: Cache<CacheKey, CacheEntry>,
    config: CacheConfig,
    metrics: Arc<CacheMetrics>,
}

impl GraphDataCache {
    /// Create a new cache with the given configuration
    pub fn new(config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .time_to_idle(config.tti.unwrap_or(config.ttl))
            .eviction_listener(|_key, _value, _cause| {
                // Could add eviction logging here
            })
            .build();

        Self {
            cache,
            config,
            metrics: Arc::new(CacheMetrics::default()),
        }
    }

    /// Get data from cache if available
    #[instrument(skip(self), fields(key = %key))]
    pub async fn get(&self, key: &CacheKey) -> Option<CachedData> {
        if let Some(entry) = self.cache.get(key).await {
            debug!("Cache hit for key: {}", key);
            self.metrics.record_hit();
            entry.mark_accessed().await;
            Some(entry.data.clone())
        } else {
            debug!("Cache miss for key: {}", key);
            self.metrics.record_miss();
            None
        }
    }

    /// Store data in cache
    #[instrument(skip(self, data), fields(key = %key))]
    pub async fn put(&self, key: CacheKey, data: CachedData) {
        debug!("Storing data in cache for key: {}", key);
        let entry = CacheEntry::new(data);
        self.cache.insert(key, entry).await;
    }

    /// Invalidate cache entries by pattern
    #[instrument(skip(self))]
    pub async fn invalidate_by_graph_type(&self, graph_type: GraphTypeKey) {
        info!("Invalidating cache entries for graph type: {}", graph_type);
        let mut invalidated_count = 0;

        // Collect keys to invalidate (to avoid borrowing issues)
        let keys_to_invalidate: Vec<CacheKey> = self.cache
            .iter()
            .filter_map(|(key, _)| {
                if key.graph_type == graph_type {
                    Some((*key).clone())
                } else {
                    None
                }
            })
            .collect();

        // Invalidate collected keys
        for key in keys_to_invalidate {
            self.cache.invalidate(&key).await;
            invalidated_count += 1;
        }

        self.metrics.invalidations.fetch_add(invalidated_count, Ordering::Relaxed);
        info!("Invalidated {} cache entries for graph type: {}", invalidated_count, graph_type);
    }

    /// Invalidate all cache entries
    #[instrument(skip(self))]
    pub async fn invalidate_all(&self) {
        info!("Invalidating all cache entries");
        let entry_count = self.cache.entry_count();
        self.cache.invalidate_all();
        self.metrics.invalidations.fetch_add(entry_count, Ordering::Relaxed);
        info!("Invalidated {} cache entries", entry_count);
    }

    /// Get cache metrics
    pub fn metrics(&self) -> Arc<CacheMetrics> {
        Arc::clone(&self.metrics)
    }

    /// Get cache statistics
    pub async fn stats(&self) -> HashMap<String, u64> {
        let mut stats = self.metrics.get_stats();
        stats.insert("entry_count".to_string(), self.cache.entry_count());
        stats.insert("weighted_size".to_string(), self.cache.weighted_size());
        stats
    }

    /// Start background refresh task for frequently accessed items
    pub async fn start_background_refresh_task(&self) -> Result<()> {
        if !self.config.enable_background_refresh {
            return Ok(());
        }

        info!("Starting background refresh task with interval: {:?}", self.config.refresh_interval);
        
        // This would typically spawn a background task
        // For now, we'll just log that it would be started
        warn!("Background refresh task not yet implemented - would run every {:?}", self.config.refresh_interval);
        
        Ok(())
    }

    /// Force refresh of a specific cache entry
    pub async fn refresh_entry(&self, key: &CacheKey) -> Result<()> {
        info!("Refreshing cache entry for key: {}", key);
        self.cache.invalidate(key).await;
        self.metrics.record_background_refresh();
        Ok(())
    }

    /// Get entries that should be refreshed based on access patterns
    pub async fn get_refresh_candidates(&self) -> Vec<CacheKey> {
        let threshold = self.config.refresh_threshold as u64;
        let candidates: Vec<CacheKey> = self.cache
            .iter()
            .filter_map(|(key, entry)| {
                if entry.access_count() >= threshold {
                    Some((*key).clone())
                } else {
                    None
                }
            })
            .collect();

        debug!("Found {} cache entries for background refresh", candidates.len());
        candidates
    }
}

impl Default for GraphDataCache {
    fn default() -> Self {
        Self::new(CacheConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_cache_key_creation() {
        let key1 = CacheKey::daily_play_count(
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
        );
        assert_eq!(key1.graph_type, GraphTypeKey::DailyPlayCount);
        assert_eq!(key1.start_date, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));

        let key2 = CacheKey::top_platforms(Some(10));
        assert_eq!(key2.graph_type, GraphTypeKey::TopPlatforms);
        assert_eq!(key2.limit, Some(10));
    }

    #[test]
    fn test_cache_key_display() {
        let key = CacheKey::daily_play_count(
            Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
            Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap())
        );
        let display = format!("{}", key);
        assert!(display.contains("daily_play_count"));
        assert!(display.contains("20240101"));
        assert!(display.contains("20240131"));
    }

    #[test]
    fn test_cache_key_with_params_hash() {
        let params = ("user_filter", "platform_filter");
        let key = CacheKey::top_users(Some(5)).with_params_hash(&params);
        assert_ne!(key.params_hash, 0);
    }

    #[tokio::test]
    async fn test_cache_basic_operations() {
        let cache = GraphDataCache::new(CacheConfig::default());
        
        let key = CacheKey::day_of_week();
        let data = CachedData::DayOfWeek(vec![]);

        // Test miss
        assert!(cache.get(&key).await.is_none());

        // Test put and hit
        cache.put(key.clone(), data.clone()).await;
        assert!(cache.get(&key).await.is_some());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = GraphDataCache::new(CacheConfig::default());
        
        let key1 = CacheKey::day_of_week();
        let key2 = CacheKey::hourly_distribution();
        let data = CachedData::DayOfWeek(vec![]);

        cache.put(key1.clone(), data.clone()).await;
        cache.put(key2.clone(), CachedData::HourlyDistribution(vec![])).await;

        // Verify both entries exist
        assert!(cache.get(&key1).await.is_some());
        assert!(cache.get(&key2).await.is_some());

        // Invalidate by graph type
        cache.invalidate_by_graph_type(GraphTypeKey::DayOfWeek).await;

        // Verify selective invalidation
        assert!(cache.get(&key1).await.is_none());
        assert!(cache.get(&key2).await.is_some());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = GraphDataCache::new(CacheConfig::default());
        let key = CacheKey::day_of_week();

        // Test miss
        cache.get(&key).await;
        
        let stats = cache.stats().await;
        assert_eq!(stats.get("misses"), Some(&1));
        assert_eq!(stats.get("hits"), Some(&0));

        // Test hit
        cache.put(key.clone(), CachedData::DayOfWeek(vec![])).await;
        cache.get(&key).await;

        let stats = cache.stats().await;
        assert_eq!(stats.get("hits"), Some(&1));
        assert!(cache.metrics().hit_rate() > 0.0);
    }
} 