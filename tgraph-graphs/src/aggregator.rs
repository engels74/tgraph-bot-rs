//! Data aggregation pipeline for processing Tautulli history into graph data

use crate::{
    DayOfWeekDataPoint, HourlyDataPoint, MonthlyDataPoint, PlayCountDataPoint,
};
use chrono::{Datelike, NaiveDate, Timelike, Weekday};
use std::collections::HashMap;
use tgraph_common::{HistoryEntry, Result};
use tokio::sync::mpsc;
use tracing::{debug, info, instrument, warn};

/// Progress information for aggregation operations
#[derive(Debug, Clone)]
pub struct AggregationProgress {
    pub stage: AggregationStage,
    pub processed: usize,
    pub total: usize,
    pub progress: f32, // 0.0 to 1.0
    pub message: String,
}

/// Stages of data aggregation
#[derive(Debug, Clone)]
pub enum AggregationStage {
    Initializing,
    Processing,
    Finalizing,
    Complete,
}

/// Configuration for aggregation operations
#[derive(Debug, Clone)]
pub struct AggregationConfig {
    /// Maximum chunk size for streaming processing
    pub chunk_size: usize,
    /// Enable progress reporting
    pub enable_progress: bool,
    /// Maximum memory usage in MB before using disk buffering
    pub max_memory_mb: usize,
}

impl Default for AggregationConfig {
    fn default() -> Self {
        Self {
            chunk_size: 1000,
            enable_progress: true,
            max_memory_mb: 256,
        }
    }
}

/// Trait for aggregating data into specific graph types
pub trait DataAggregator<T> {
    /// Process raw history entries and return aggregated data points
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
    ) -> Result<Vec<T>>;

    /// Process entries in streaming fashion with progress reporting
    async fn aggregate_streaming(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<T>> {
        let total = entries.len();
        self.send_progress(&progress_tx, AggregationStage::Initializing, 0, total, "Starting aggregation".to_string());

        let result = if total > config.chunk_size {
            self.aggregate_chunked(entries, config, progress_tx).await?
        } else {
            self.send_progress(&progress_tx, AggregationStage::Processing, 0, total, "Processing data".to_string());
            let result = self.aggregate(entries, config)?;
            self.send_progress(&progress_tx, AggregationStage::Complete, total, total, "Aggregation complete".to_string());
            result
        };

        Ok(result)
    }

    /// Process large datasets in chunks
    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<T>>;

    /// Send progress update if reporting is enabled
    fn send_progress(
        &self,
        progress_tx: &Option<mpsc::UnboundedSender<AggregationProgress>>,
        stage: AggregationStage,
        processed: usize,
        total: usize,
        message: String,
    ) {
        if let Some(ref tx) = progress_tx {
            let progress = if total > 0 { processed as f32 / total as f32 } else { 0.0 };
            let _ = tx.send(AggregationProgress {
                stage,
                processed,
                total,
                progress,
                message,
            });
        }
    }
}

/// Aggregator for daily play count data
#[derive(Debug)]
pub struct DailyPlayCountAggregator {
    /// Optional date range filter
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

impl DailyPlayCountAggregator {
    pub fn new() -> Self {
        Self {
            start_date: None,
            end_date: None,
        }
    }

    pub fn with_date_range(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            start_date: Some(start),
            end_date: Some(end),
        }
    }

    /// Extract date from timestamp
    fn extract_date(&self, timestamp: i64) -> Option<NaiveDate> {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.naive_utc().date())
    }

    /// Check if date is within range
    fn is_in_range(&self, date: NaiveDate) -> bool {
        if let Some(start) = self.start_date {
            if date < start {
                return false;
            }
        }
        if let Some(end) = self.end_date {
            if date > end {
                return false;
            }
        }
        true
    }
}

impl DataAggregator<PlayCountDataPoint> for DailyPlayCountAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<PlayCountDataPoint>> {
        let mut daily_counts: HashMap<NaiveDate, u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some(date) = self.extract_date(timestamp) {
                    if self.is_in_range(date) {
                        *daily_counts.entry(date).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut result: Vec<PlayCountDataPoint> = daily_counts
            .into_iter()
            .map(|(date, count)| PlayCountDataPoint {
                date,
                count,
                label: Some(format!("{} plays", count)),
            })
            .collect();

        // Sort by date
        result.sort_by_key(|point| point.date);

        debug!("Aggregated {} daily play count data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<PlayCountDataPoint>> {
        let total = entries.len();
        let mut daily_counts: HashMap<NaiveDate, u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let Some(timestamp) = entry.date {
                    if let Some(date) = self.extract_date(timestamp) {
                        if self.is_in_range(date) {
                            *daily_counts.entry(date).or_insert(0) += 1;
                        }
                    }
                }
                processed += 1;
            }

            // Yield control to allow other tasks to run
            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing results".to_string());

        let mut result: Vec<PlayCountDataPoint> = daily_counts
            .into_iter()
            .map(|(date, count)| PlayCountDataPoint {
                date,
                count,
                label: Some(format!("{} plays", count)),
            })
            .collect();

        result.sort_by_key(|point| point.date);

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Daily aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} daily play count data points", result.len());
        Ok(result)
    }
}

/// Aggregator for day of week data
#[derive(Debug)]
pub struct DayOfWeekAggregator;

impl DayOfWeekAggregator {
    pub fn new() -> Self {
        Self
    }

    /// Extract weekday from timestamp
    fn extract_weekday(&self, timestamp: i64) -> Option<Weekday> {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.weekday())
    }
}

impl DataAggregator<DayOfWeekDataPoint> for DayOfWeekAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<DayOfWeekDataPoint>> {
        let mut weekday_counts: HashMap<Weekday, u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some(weekday) = self.extract_weekday(timestamp) {
                    *weekday_counts.entry(weekday).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<DayOfWeekDataPoint> = weekday_counts
            .into_iter()
            .map(|(weekday, count)| DayOfWeekDataPoint {
                weekday,
                count,
                label: Some(format!("{} plays", count)),
            })
            .collect();

        // Sort by weekday (Monday = 0, Sunday = 6)
        result.sort_by_key(|point| match point.weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        });

        debug!("Aggregated {} day of week data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<DayOfWeekDataPoint>> {
        let total = entries.len();
        let mut weekday_counts: HashMap<Weekday, u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing weekday chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let Some(timestamp) = entry.date {
                    if let Some(weekday) = self.extract_weekday(timestamp) {
                        *weekday_counts.entry(weekday).or_insert(0) += 1;
                    }
                }
                processed += 1;
            }

            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing weekday results".to_string());

        let mut result: Vec<DayOfWeekDataPoint> = weekday_counts
            .into_iter()
            .map(|(weekday, count)| DayOfWeekDataPoint {
                weekday,
                count,
                label: Some(format!("{} plays", count)),
            })
            .collect();

        result.sort_by_key(|point| match point.weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        });

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Day of week aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} day of week data points", result.len());
        Ok(result)
    }
}

/// Aggregator for hourly distribution data
#[derive(Debug)]
pub struct HourlyDistributionAggregator;

impl HourlyDistributionAggregator {
    pub fn new() -> Self {
        Self
    }

    /// Extract hour from timestamp
    fn extract_hour(&self, timestamp: i64) -> Option<u8> {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| dt.hour() as u8)
    }
}

impl DataAggregator<HourlyDataPoint> for HourlyDistributionAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<HourlyDataPoint>> {
        let mut hourly_counts: HashMap<u8, u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some(hour) = self.extract_hour(timestamp) {
                    *hourly_counts.entry(hour).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<HourlyDataPoint> = hourly_counts
            .into_iter()
            .map(|(hour, count)| HourlyDataPoint {
                hour,
                count,
                label: Some(format!("{}:00 - {} plays", hour, count)),
            })
            .collect();

        // Sort by hour
        result.sort_by_key(|point| point.hour);

        debug!("Aggregated {} hourly distribution data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<HourlyDataPoint>> {
        let total = entries.len();
        let mut hourly_counts: HashMap<u8, u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing hourly chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let Some(timestamp) = entry.date {
                    if let Some(hour) = self.extract_hour(timestamp) {
                        *hourly_counts.entry(hour).or_insert(0) += 1;
                    }
                }
                processed += 1;
            }

            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing hourly results".to_string());

        let mut result: Vec<HourlyDataPoint> = hourly_counts
            .into_iter()
            .map(|(hour, count)| HourlyDataPoint {
                hour,
                count,
                label: Some(format!("{}:00 - {} plays", hour, count)),
            })
            .collect();

        result.sort_by_key(|point| point.hour);

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Hourly aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} hourly distribution data points", result.len());
        Ok(result)
    }
}

/// Aggregator for monthly trends data
#[derive(Debug)]
pub struct MonthlyTrendsAggregator {
    /// Optional year range filter
    pub start_year: Option<i32>,
    pub end_year: Option<i32>,
}

impl MonthlyTrendsAggregator {
    pub fn new() -> Self {
        Self {
            start_year: None,
            end_year: None,
        }
    }

    pub fn with_year_range(start_year: i32, end_year: i32) -> Self {
        Self {
            start_year: Some(start_year),
            end_year: Some(end_year),
        }
    }

    /// Extract year and month from timestamp
    fn extract_year_month(&self, timestamp: i64) -> Option<(i32, u32)> {
        chrono::DateTime::from_timestamp(timestamp, 0)
            .map(|dt| (dt.year(), dt.month()))
    }

    /// Check if year is within range
    fn is_year_in_range(&self, year: i32) -> bool {
        if let Some(start) = self.start_year {
            if year < start {
                return false;
            }
        }
        if let Some(end) = self.end_year {
            if year > end {
                return false;
            }
        }
        true
    }
}

impl DataAggregator<MonthlyDataPoint> for MonthlyTrendsAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<MonthlyDataPoint>> {
        let mut monthly_counts: HashMap<(i32, u32), u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some((year, month)) = self.extract_year_month(timestamp) {
                    if self.is_year_in_range(year) {
                        *monthly_counts.entry((year, month)).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut result: Vec<MonthlyDataPoint> = monthly_counts
            .into_iter()
            .map(|((year, month), count)| MonthlyDataPoint {
                year,
                month,
                count,
                label: Some(format!("{}/{:02} - {} plays", year, month, count)),
            })
            .collect();

        // Sort by year then month
        result.sort_by_key(|point| (point.year, point.month));

        debug!("Aggregated {} monthly trends data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<MonthlyDataPoint>> {
        let total = entries.len();
        let mut monthly_counts: HashMap<(i32, u32), u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing monthly chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let Some(timestamp) = entry.date {
                    if let Some((year, month)) = self.extract_year_month(timestamp) {
                        if self.is_year_in_range(year) {
                            *monthly_counts.entry((year, month)).or_insert(0) += 1;
                        }
                    }
                }
                processed += 1;
            }

            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing monthly results".to_string());

        let mut result: Vec<MonthlyDataPoint> = monthly_counts
            .into_iter()
            .map(|((year, month), count)| MonthlyDataPoint {
                year,
                month,
                count,
                label: Some(format!("{}/{:02} - {} plays", year, month, count)),
            })
            .collect();

        result.sort_by_key(|point| (point.year, point.month));

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Monthly trends aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} monthly trends data points", result.len());
        Ok(result)
    }
}

// Data structures for top platforms and users need to be defined
// These are placeholders based on the existing graph patterns

/// Data point for top platforms
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopPlatformDataPoint {
    pub platform: String,
    pub count: u32,
    pub percentage: f64,
    pub label: Option<String>,
}

/// Data point for top users
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TopUserDataPoint {
    pub user_id: i32,
    pub username: String,
    pub friendly_name: Option<String>,
    pub count: u32,
    pub label: Option<String>,
}

/// Aggregator for top platforms data
#[derive(Debug)]
pub struct TopPlatformsAggregator {
    /// Maximum number of platforms to return
    pub limit: usize,
}

impl TopPlatformsAggregator {
    pub fn new() -> Self {
        Self { limit: 10 }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self { limit }
    }
}

impl DataAggregator<TopPlatformDataPoint> for TopPlatformsAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<TopPlatformDataPoint>> {
        let mut platform_counts: HashMap<String, u32> = HashMap::new();
        let total_plays = entries.len() as f64;

        for entry in entries {
            if let Some(platform) = entry.platform {
                if !platform.is_empty() {
                    *platform_counts.entry(platform).or_insert(0) += 1;
                }
            }
        }

        let mut result: Vec<TopPlatformDataPoint> = platform_counts
            .into_iter()
            .map(|(platform, count)| TopPlatformDataPoint {
                platform: platform.clone(),
                count,
                percentage: (count as f64 / total_plays) * 100.0,
                label: Some(format!("{} - {} plays ({:.1}%)", platform, count, (count as f64 / total_plays) * 100.0)),
            })
            .collect();

        // Sort by count descending, take top N
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result.truncate(self.limit);

        debug!("Aggregated {} top platform data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopPlatformDataPoint>> {
        let total = entries.len();
        let mut platform_counts: HashMap<String, u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing platforms chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let Some(platform) = &entry.platform {
                    if !platform.is_empty() {
                        *platform_counts.entry(platform.clone()).or_insert(0) += 1;
                    }
                }
                processed += 1;
            }

            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing platform results".to_string());

        let total_plays = total as f64;
        let mut result: Vec<TopPlatformDataPoint> = platform_counts
            .into_iter()
            .map(|(platform, count)| TopPlatformDataPoint {
                platform: platform.clone(),
                count,
                percentage: (count as f64 / total_plays) * 100.0,
                label: Some(format!("{} - {} plays ({:.1}%)", platform, count, (count as f64 / total_plays) * 100.0)),
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));
        result.truncate(self.limit);

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Top platforms aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} top platform data points", result.len());
        Ok(result)
    }
}

/// Aggregator for top users data
#[derive(Debug)]
pub struct TopUsersAggregator {
    /// Maximum number of users to return
    pub limit: usize,
}

impl TopUsersAggregator {
    pub fn new() -> Self {
        Self { limit: 10 }
    }

    pub fn with_limit(limit: usize) -> Self {
        Self { limit }
    }
}

impl DataAggregator<TopUserDataPoint> for TopUsersAggregator {
    #[instrument(skip(self, entries))]
    fn aggregate(
        &self,
        entries: Vec<HistoryEntry>,
        _config: &AggregationConfig,
    ) -> Result<Vec<TopUserDataPoint>> {
        let mut user_counts: HashMap<(i32, String, Option<String>), u32> = HashMap::new();

        for entry in entries {
            if let (Some(user_id), Some(username)) = (entry.user_id, entry.username) {
                let key = (user_id, username.clone(), entry.friendly_name.clone());
                *user_counts.entry(key).or_insert(0) += 1;
            }
        }

        let mut result: Vec<TopUserDataPoint> = user_counts
            .into_iter()
            .map(|((user_id, username, friendly_name), count)| TopUserDataPoint {
                user_id,
                username: username.clone(),
                friendly_name: friendly_name.clone(),
                count,
                label: Some(format!("{} - {} plays", 
                    friendly_name.as_deref().unwrap_or(&username), count)),
            })
            .collect();

        // Sort by count descending, take top N
        result.sort_by(|a, b| b.count.cmp(&a.count));
        result.truncate(self.limit);

        debug!("Aggregated {} top user data points", result.len());
        Ok(result)
    }

    async fn aggregate_chunked(
        &self,
        entries: Vec<HistoryEntry>,
        config: &AggregationConfig,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopUserDataPoint>> {
        let total = entries.len();
        let mut user_counts: HashMap<(i32, String, Option<String>), u32> = HashMap::new();
        let mut processed = 0;

        for chunk in entries.chunks(config.chunk_size) {
            self.send_progress(
                &progress_tx,
                AggregationStage::Processing,
                processed,
                total,
                format!("Processing users chunk {}/{}", processed / config.chunk_size + 1, (total + config.chunk_size - 1) / config.chunk_size),
            );

            for entry in chunk {
                if let (Some(user_id), Some(username)) = (entry.user_id, &entry.username) {
                    let key = (user_id, username.clone(), entry.friendly_name.clone());
                    *user_counts.entry(key).or_insert(0) += 1;
                }
                processed += 1;
            }

            tokio::task::yield_now().await;
        }

        self.send_progress(&progress_tx, AggregationStage::Finalizing, processed, total, "Finalizing user results".to_string());

        let mut result: Vec<TopUserDataPoint> = user_counts
            .into_iter()
            .map(|((user_id, username, friendly_name), count)| TopUserDataPoint {
                user_id,
                username: username.clone(),
                friendly_name: friendly_name.clone(),
                count,
                label: Some(format!("{} - {} plays", 
                    friendly_name.as_deref().unwrap_or(&username), count)),
            })
            .collect();

        result.sort_by(|a, b| b.count.cmp(&a.count));
        result.truncate(self.limit);

        self.send_progress(&progress_tx, AggregationStage::Complete, processed, total, "Top users aggregation complete".to_string());
        
        info!("Chunked aggregation completed: {} top user data points", result.len());
        Ok(result)
    }
}

/// Convenience aggregation manager for all graph types
#[derive(Debug)]
pub struct AggregationManager {
    config: AggregationConfig,
}

impl AggregationManager {
    pub fn new(config: AggregationConfig) -> Self {
        Self { config }
    }

    pub fn default() -> Self {
        Self {
            config: AggregationConfig::default(),
        }
    }

    /// Aggregate data for daily play counts
    pub async fn aggregate_daily_play_counts(
        &self,
        entries: Vec<HistoryEntry>,
        date_range: Option<(NaiveDate, NaiveDate)>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<PlayCountDataPoint>> {
        let aggregator = if let Some((start, end)) = date_range {
            DailyPlayCountAggregator::with_date_range(start, end)
        } else {
            DailyPlayCountAggregator::new()
        };

        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }

    /// Aggregate data for day of week analysis
    pub async fn aggregate_day_of_week(
        &self,
        entries: Vec<HistoryEntry>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<DayOfWeekDataPoint>> {
        let aggregator = DayOfWeekAggregator::new();
        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }

    /// Aggregate data for hourly distribution
    pub async fn aggregate_hourly_distribution(
        &self,
        entries: Vec<HistoryEntry>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<HourlyDataPoint>> {
        let aggregator = HourlyDistributionAggregator::new();
        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }

    /// Aggregate data for monthly trends
    pub async fn aggregate_monthly_trends(
        &self,
        entries: Vec<HistoryEntry>,
        year_range: Option<(i32, i32)>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<MonthlyDataPoint>> {
        let aggregator = if let Some((start, end)) = year_range {
            MonthlyTrendsAggregator::with_year_range(start, end)
        } else {
            MonthlyTrendsAggregator::new()
        };

        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }

    /// Aggregate data for top platforms
    pub async fn aggregate_top_platforms(
        &self,
        entries: Vec<HistoryEntry>,
        limit: Option<usize>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopPlatformDataPoint>> {
        let aggregator = if let Some(limit) = limit {
            TopPlatformsAggregator::with_limit(limit)
        } else {
            TopPlatformsAggregator::new()
        };

        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }

    /// Aggregate data for top users
    pub async fn aggregate_top_users(
        &self,
        entries: Vec<HistoryEntry>,
        limit: Option<usize>,
        progress_tx: Option<mpsc::UnboundedSender<AggregationProgress>>,
    ) -> Result<Vec<TopUserDataPoint>> {
        let aggregator = if let Some(limit) = limit {
            TopUsersAggregator::with_limit(limit)
        } else {
            TopUsersAggregator::new()
        };

        aggregator.aggregate_streaming(entries, &self.config, progress_tx).await
    }
}

impl Default for DailyPlayCountAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DayOfWeekAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for HourlyDistributionAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MonthlyTrendsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TopPlatformsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TopUsersAggregator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveDate, Weekday};
    use tgraph_common::HistoryEntry;

    fn create_test_history_entry(date: i64, user_id: i32, username: &str, platform: &str) -> HistoryEntry {
        HistoryEntry {
            date: Some(date),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            platform: Some(platform.to_string()),
            friendly_name: Some(format!("Friendly {}", username)),
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
    async fn test_daily_play_count_aggregation() {
        let entries = vec![
            create_test_history_entry(1640995200, 1, "user1", "Web"), // 2022-01-01
            create_test_history_entry(1640995200, 2, "user2", "Mobile"), // 2022-01-01 
            create_test_history_entry(1641081600, 1, "user1", "Web"), // 2022-01-02
        ];

        let aggregator = DailyPlayCountAggregator::new();
        let config = AggregationConfig::default();
        let result = aggregator.aggregate(entries, &config).unwrap();

        assert_eq!(result.len(), 2);
        
        // Check first day
        assert_eq!(result[0].date, NaiveDate::from_ymd_opt(2022, 1, 1).unwrap());
        assert_eq!(result[0].count, 2);
        
        // Check second day
        assert_eq!(result[1].date, NaiveDate::from_ymd_opt(2022, 1, 2).unwrap());
        assert_eq!(result[1].count, 1);
    }

    #[tokio::test] 
    async fn test_day_of_week_aggregation() {
        let entries = vec![
            create_test_history_entry(1640995200, 1, "user1", "Web"), // Saturday
            create_test_history_entry(1641081600, 2, "user2", "Mobile"), // Sunday
            create_test_history_entry(1641168000, 1, "user1", "Web"), // Monday
        ];

        let aggregator = DayOfWeekAggregator::new();
        let config = AggregationConfig::default();
        let result = aggregator.aggregate(entries, &config).unwrap();

        assert_eq!(result.len(), 3);
        
        // Should be sorted by weekday
        assert_eq!(result[0].weekday, Weekday::Mon);
        assert_eq!(result[0].count, 1);
        assert_eq!(result[1].weekday, Weekday::Sat);
        assert_eq!(result[1].count, 1);
        assert_eq!(result[2].weekday, Weekday::Sun);
        assert_eq!(result[2].count, 1);
    }

    #[tokio::test]
    async fn test_top_platforms_aggregation() {
        let entries = vec![
            create_test_history_entry(1640995200, 1, "user1", "Web"),
            create_test_history_entry(1640995200, 2, "user2", "Web"),
            create_test_history_entry(1640995200, 3, "user3", "Mobile"),
            create_test_history_entry(1640995200, 4, "user4", "TV"),
        ];

        let aggregator = TopPlatformsAggregator::with_limit(2);
        let config = AggregationConfig::default();
        let result = aggregator.aggregate(entries, &config).unwrap();

        assert_eq!(result.len(), 2);
        
        // Should be sorted by count descending
        assert_eq!(result[0].platform, "Web");
        assert_eq!(result[0].count, 2);
        assert_eq!(result[0].percentage, 50.0);
        
        // Second place could be either Mobile or TV (both have 1 count)
        assert!(result[1].platform == "Mobile" || result[1].platform == "TV");
        assert_eq!(result[1].count, 1);
        assert_eq!(result[1].percentage, 25.0);
    }

    #[tokio::test]
    async fn test_aggregation_manager() {
        let manager = AggregationManager::default();
        let entries = vec![
            create_test_history_entry(1640995200, 1, "user1", "Web"),
            create_test_history_entry(1640995200, 2, "user2", "Mobile"),
        ];

        // Test daily aggregation
        let daily_result = manager.aggregate_daily_play_counts(entries.clone(), None, None).await.unwrap();
        assert_eq!(daily_result.len(), 1);
        assert_eq!(daily_result[0].count, 2);

        // Test day of week aggregation
        let weekday_result = manager.aggregate_day_of_week(entries.clone(), None).await.unwrap();
        assert_eq!(weekday_result.len(), 1);
        assert_eq!(weekday_result[0].weekday, Weekday::Sat);

        // Test top platforms aggregation
        let platforms_result = manager.aggregate_top_platforms(entries, None, None).await.unwrap();
        assert_eq!(platforms_result.len(), 2);
    }

    #[tokio::test]
    async fn test_chunked_processing() {
        let mut entries = Vec::new();
        for i in 0..2500 { // More than default chunk size
            entries.push(create_test_history_entry(1640995200 + i, 1, "user1", "Web"));
        }

        let aggregator = DailyPlayCountAggregator::new();
        let config = AggregationConfig {
            chunk_size: 1000, // Force chunked processing
            ..AggregationConfig::default()
        };

        let (tx, mut rx) = mpsc::unbounded_channel();
        
        // Start aggregation in background
        let handle = tokio::spawn(async move {
            aggregator.aggregate_streaming(entries, &config, Some(tx)).await
        });

        // Collect progress updates
        let mut progress_updates = Vec::new();
        while let Some(progress) = rx.recv().await {
            progress_updates.push(progress.clone());
            if matches!(progress.stage, AggregationStage::Complete) {
                break;
            }
        }

        let result = handle.await.unwrap().unwrap();
        
        // Should have processed all entries into one daily count
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].count, 2500);
        
        // Should have received progress updates
        assert!(!progress_updates.is_empty());
        assert!(progress_updates.iter().any(|p| matches!(p.stage, AggregationStage::Processing)));
        assert!(progress_updates.iter().any(|p| matches!(p.stage, AggregationStage::Complete)));
    }
} 