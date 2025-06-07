//! Cached aggregator that wraps the existing aggregation system with caching

use crate::{
    AggregationConfig, AggregationManager, AggregationProgress,
    CacheConfig, CacheKey, CachedData, GraphDataCache,
    DayOfWeekDataPoint, HourlyDataPoint, MonthlyDataPoint, PlayCountDataPoint,
    TopPlatformDataPoint, TopUserDataPoint,
};
use chrono::NaiveDate;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};
use tgraph_common::{HistoryEntry, Result};

/// Cached aggregator that provides transparent caching for all aggregation operations
pub struct CachedAggregationManager {
    /// Underlying aggregation manager
    aggregator: AggregationManager,
    /// Cache for storing aggregated data
    cache: Arc<GraphDataCache>,
    /// Whether caching is enabled
    cache_enabled: bool,
}

impl CachedAggregationManager {
    /// Create a new cached aggregation manager
    pub fn new(aggregation_config: AggregationConfig, cache_config: CacheConfig) -> Self {
        Self {
            aggregator: AggregationManager::new(aggregation_config),
            cache: Arc::new(GraphDataCache::new(cache_config)),
            cache_enabled: true,
        }
    }

    /// Create with default configurations
    pub fn default() -> Self {
        Self {
            aggregator: AggregationManager::default(),
            cache: Arc::new(GraphDataCache::default()),
            cache_enabled: true,
        }
    }

    /// Create without caching (pass-through mode)
    pub fn without_cache(aggregation_config: AggregationConfig) -> Self {
        Self {
            aggregator: AggregationManager::new(aggregation_config),
            cache: Arc::new(GraphDataCache::default()),
            cache_enabled: false,
        }
    }

    /// Enable or disable caching
    pub fn set_cache_enabled(&mut self, enabled: bool) {
        self.cache_enabled = enabled;
        if enabled {
            info!("Cache enabled for aggregation operations");
        } else {
            warn!("Cache disabled - all operations will bypass cache");
        }
    }

    /// Get cache reference for direct access
    pub fn cache(&self) -> Arc<GraphDataCache> {
        Arc::clone(&self.cache)
    }

    /// Get cache statistics
    pub async fn cache_stats(&self) -> std::collections::HashMap<String, u64> {
        self.cache.stats().await
    }

    /// Invalidate cache for specific graph type
    pub async fn invalidate_cache(&self, graph_type: crate::GraphTypeKey) -> Result<()> {
        if self.cache_enabled {
            self.cache.invalidate_by_graph_type(graph_type).await;
        }
        Ok(())
    }

    /// Invalidate all cache entries
    pub async fn invalidate_all_cache(&self) -> Result<()> {
        if self.cache_enabled {
            self.cache.invalidate_all().await;
        }
        Ok(())
    }

    /// Aggregate daily play counts with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_daily_play_counts(
        &self,
        entries: Vec<HistoryEntry>,
        date_range: Option<(NaiveDate, NaiveDate)>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<PlayCountDataPoint>> {
        let cache_key = CacheKey::daily_play_count(
            date_range.map(|(start, _)| start),
            date_range.map(|(_, end)| end),
        );

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::DailyPlayCount(data) = cached_data {
                    debug!("Cache hit for daily play counts");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for daily play counts - computing data");
        let data = self.aggregator
            .aggregate_daily_play_counts(entries, date_range, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::DailyPlayCount(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate day of week data with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_day_of_week(
        &self,
        entries: Vec<HistoryEntry>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<DayOfWeekDataPoint>> {
        let cache_key = CacheKey::day_of_week();

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::DayOfWeek(data) = cached_data {
                    debug!("Cache hit for day of week data");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for day of week data - computing data");
        let data = self.aggregator
            .aggregate_day_of_week(entries, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::DayOfWeek(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate hourly distribution with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_hourly_distribution(
        &self,
        entries: Vec<HistoryEntry>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<HourlyDataPoint>> {
        let cache_key = CacheKey::hourly_distribution();

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::HourlyDistribution(data) = cached_data {
                    debug!("Cache hit for hourly distribution data");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for hourly distribution data - computing data");
        let data = self.aggregator
            .aggregate_hourly_distribution(entries, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::HourlyDistribution(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate monthly trends with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_monthly_trends(
        &self,
        entries: Vec<HistoryEntry>,
        year_range: Option<(i32, i32)>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<MonthlyDataPoint>> {
        let cache_key = CacheKey::monthly_trends(
            year_range.map(|(start, _)| start),
            year_range.map(|(_, end)| end),
        );

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::MonthlyTrends(data) = cached_data {
                    debug!("Cache hit for monthly trends data");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for monthly trends data - computing data");
        let data = self.aggregator
            .aggregate_monthly_trends(entries, year_range, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::MonthlyTrends(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate top platforms with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_top_platforms(
        &self,
        entries: Vec<HistoryEntry>,
        limit: Option<usize>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopPlatformDataPoint>> {
        let cache_key = CacheKey::top_platforms(limit);

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::TopPlatforms(data) = cached_data {
                    debug!("Cache hit for top platforms data");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for top platforms data - computing data");
        let data = self.aggregator
            .aggregate_top_platforms(entries, limit, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::TopPlatforms(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate top users with caching
    #[instrument(skip(self, entries, progress_tx))]
    pub async fn aggregate_top_users(
        &self,
        entries: Vec<HistoryEntry>,
        limit: Option<usize>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopUserDataPoint>> {
        let cache_key = CacheKey::top_users(limit);

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                if let CachedData::TopUsers(data) = cached_data {
                    debug!("Cache hit for top users data");
                    return Ok(data);
                }
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for top users data - computing data");
        let data = self.aggregator
            .aggregate_top_users(entries, limit, progress_tx)
            .await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache
                .put(cache_key, CachedData::TopUsers(data.clone()))
                .await;
        }

        Ok(data)
    }

    /// Aggregate with custom parameters and caching
    #[instrument(skip(self, entries, progress_tx, params, aggregation_fn))]
    pub async fn aggregate_with_params<T, F>(
        &self,
        cache_key: CacheKey,
        entries: Vec<HistoryEntry>,
        params: T,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
        aggregation_fn: F,
    ) -> Result<CachedData>
    where
        T: std::hash::Hash,
        F: std::future::Future<Output = Result<CachedData>>,
    {
        // Add params hash to cache key
        let cache_key = cache_key.with_params_hash(&params);

        // Try cache first if enabled
        if self.cache_enabled {
            if let Some(cached_data) = self.cache.get(&cache_key).await {
                debug!("Cache hit for custom aggregation");
                return Ok(cached_data);
            }
        }

        // Cache miss or disabled - compute data
        debug!("Cache miss for custom aggregation - computing data");
        let data = aggregation_fn.await?;

        // Store in cache if enabled
        if self.cache_enabled {
            self.cache.put(cache_key, data.clone()).await;
        }

        Ok(data)
    }

    /// Start background refresh task
    pub async fn start_background_refresh(&self) -> Result<()> {
        if self.cache_enabled {
            self.cache.start_background_refresh_task().await?;
        }
        Ok(())
    }

    /// Manually refresh cache entries that are frequently accessed
    pub async fn refresh_popular_entries(&self) -> Result<()> {
        if !self.cache_enabled {
            return Ok(());
        }

        let candidates = self.cache.get_refresh_candidates().await;
        info!("Found {} cache entries for background refresh", candidates.len());

        for key in candidates {
            if let Err(e) = self.cache.refresh_entry(&key).await {
                warn!("Failed to refresh cache entry {}: {}", key, e);
            }
        }

        Ok(())
    }

    /// Preload cache with common queries
    pub async fn preload_cache(&self, entries: Vec<HistoryEntry>) -> Result<()> {
        if !self.cache_enabled {
            return Ok(());
        }

        info!("Preloading cache with common aggregations");

        // Preload common aggregations
        let _ = self.aggregate_day_of_week(entries.clone(), None).await;
        let _ = self.aggregate_hourly_distribution(entries.clone(), None).await;
        let _ = self.aggregate_top_platforms(entries.clone(), Some(10), None).await;
        let _ = self.aggregate_top_users(entries.clone(), Some(10), None).await;

        info!("Cache preloading completed");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn create_test_entry(date: i64, user_id: i32, username: &str, platform: &str) -> HistoryEntry {
        HistoryEntry {
            date: Some(date),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            friendly_name: Some(format!("User {}", user_id)),
            platform: Some(platform.to_string()),
            media_type: None,
            rating_key: None,
            parent_rating_key: None,
            grandparent_rating_key: None,
            title: None,
            parent_title: None,
            grandparent_title: None,
            year: None,
            watched_status: None,
            percent_complete: None,
            duration: None,
            transcode_decision: None,
            player: None,
            ip_address: None,
        }
    }

    #[tokio::test]
    async fn test_cached_aggregator_creation() {
        let aggregator = CachedAggregationManager::default();
        assert!(aggregator.cache_enabled);
    }

    #[tokio::test]
    async fn test_cache_enable_disable() {
        let mut aggregator = CachedAggregationManager::default();
        
        aggregator.set_cache_enabled(false);
        assert!(!aggregator.cache_enabled);
        
        aggregator.set_cache_enabled(true);
        assert!(aggregator.cache_enabled);
    }

    #[tokio::test]
    async fn test_daily_play_count_caching() {
        let aggregator = CachedAggregationManager::default();
        
        let entries = vec![
            create_test_entry(1640995200, 1, "user1", "web"), // 2022-01-01
            create_test_entry(1641081600, 2, "user2", "mobile"), // 2022-01-02
        ];

        // First call should compute and cache
        let result1 = aggregator
            .aggregate_daily_play_counts(entries.clone(), None, None)
            .await
            .unwrap();

        // Second call should hit cache
        let result2 = aggregator
            .aggregate_daily_play_counts(entries, None, None)
            .await
            .unwrap();

        assert_eq!(result1.len(), result2.len());
        
        // Verify cache hit
        let stats = aggregator.cache_stats().await;
        assert!(stats.get("hits").unwrap_or(&0) > &0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let aggregator = CachedAggregationManager::default();
        
        let entries = vec![
            create_test_entry(1640995200, 1, "user1", "web"),
        ];

        // Cache some data
        let _ = aggregator
            .aggregate_day_of_week(entries.clone(), None)
            .await
            .unwrap();

        // Verify cache has data
        let stats_before = aggregator.cache_stats().await;
        assert!(stats_before.get("entry_count").unwrap_or(&0) > &0);

        // Invalidate cache
        aggregator
            .invalidate_cache(crate::GraphTypeKey::DayOfWeek)
            .await
            .unwrap();

        // Verify cache entry was removed
        let stats_after = aggregator.cache_stats().await;
        assert!(stats_after.get("invalidations").unwrap_or(&0) > &0);
    }

    #[tokio::test]
    async fn test_cache_disabled_mode() {
        let aggregator = CachedAggregationManager::without_cache(AggregationConfig::default());
        
        let entries = vec![
            create_test_entry(1640995200, 1, "user1", "web"),
        ];

        // Multiple calls should not use cache
        let _ = aggregator
            .aggregate_day_of_week(entries.clone(), None)
            .await
            .unwrap();
        let _ = aggregator
            .aggregate_day_of_week(entries, None)
            .await
            .unwrap();

        // Verify no cache hits
        let stats = aggregator.cache_stats().await;
        assert_eq!(stats.get("hits").unwrap_or(&0), &0);
    }

    #[tokio::test]
    async fn test_cache_preloading() {
        let aggregator = CachedAggregationManager::default();
        
        let entries = vec![
            create_test_entry(1640995200, 1, "user1", "web"),
            create_test_entry(1641081600, 2, "user2", "mobile"),
        ];

        // Preload cache
        aggregator.preload_cache(entries).await.unwrap();

        // Verify cache has entries
        let stats = aggregator.cache_stats().await;
        assert!(stats.get("entry_count").unwrap_or(&0) > &0);
    }

    #[tokio::test]
    async fn test_custom_cache_config() {
        let cache_config = CacheConfig {
            max_capacity: 100,
            ttl: Duration::from_secs(300),
            tti: Some(Duration::from_secs(150)),
            enable_background_refresh: false,
            refresh_threshold: 3,
            refresh_interval: Duration::from_secs(60),
        };

        let aggregator = CachedAggregationManager::new(
            AggregationConfig::default(),
            cache_config,
        );

        assert!(aggregator.cache_enabled);
    }
} 