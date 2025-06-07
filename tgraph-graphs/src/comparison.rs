//! Graph comparison functionality for overlaying multiple time periods

use crate::{
    ComparisonConfig, ComparisonDisplayMode, ComparisonPeriod, DateRange, PlayCountDataPoint,
    DayOfWeekDataPoint,
};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tgraph_common::{HistoryEntry, Result};

/// Comparison result containing data for multiple periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonResult<T> {
    /// Primary period data
    pub primary: ComparisonPeriodData<T>,
    /// Secondary period data sets
    pub secondary: Vec<ComparisonPeriodData<T>>,
    /// Comparison configuration used
    pub config: ComparisonConfig,
    /// Calculated differences between periods
    pub differences: Option<Vec<ComparisonDifference>>,
}

/// Data for a single comparison period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonPeriodData<T> {
    /// Label for this period
    pub label: String,
    /// Date range for this period
    pub date_range: DateRange,
    /// Aggregated data points
    pub data: Vec<T>,
    /// Color for this period
    pub color: Option<String>,
    /// Summary statistics
    pub summary: ComparisonSummary,
}

/// Summary statistics for a comparison period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonSummary {
    /// Total count/sum for the period
    pub total: f64,
    /// Average value per data point
    pub average: f64,
    /// Peak value in the period
    pub peak: f64,
    /// Number of data points
    pub data_points: usize,
}

/// Difference calculation between periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonDifference {
    /// Label identifying what's being compared
    pub label: String,
    /// Absolute difference
    pub absolute_difference: f64,
    /// Percentage difference
    pub percentage_difference: f64,
    /// Growth direction
    pub growth_direction: GrowthDirection,
}

/// Direction of growth between periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GrowthDirection {
    Positive,
    Negative,
    Neutral,
}

/// Manager for handling graph comparisons
pub struct ComparisonManager {
    config: ComparisonConfig,
}

impl ComparisonManager {
    pub fn new(config: ComparisonConfig) -> Self {
        Self { config }
    }

    /// Compare daily play count data across multiple periods
    pub async fn compare_daily_play_counts(
        &self,
        primary_data: Vec<HistoryEntry>,
        comparison_data: Vec<Vec<HistoryEntry>>,
    ) -> Result<ComparisonResult<PlayCountDataPoint>> {
        let primary_aggregated = self.aggregate_daily_play_counts(&primary_data, &self.config.primary_range)?;
        let primary_period = ComparisonPeriodData {
            label: "Primary Period".to_string(),
            date_range: self.config.primary_range.clone(),
            color: None,
            summary: self.calculate_summary_play_counts(&primary_aggregated),
            data: primary_aggregated,
        };

        let mut secondary_periods = Vec::new();
        for (i, data) in comparison_data.iter().enumerate() {
            if let Some(period_config) = self.config.comparison_ranges.get(i) {
                if period_config.enabled {
                    let aggregated = self.aggregate_daily_play_counts(data, &period_config.date_range)?;
                    secondary_periods.push(ComparisonPeriodData {
                        label: period_config.label.clone(),
                        date_range: period_config.date_range.clone(),
                        color: period_config.color.clone(),
                        summary: self.calculate_summary_play_counts(&aggregated),
                        data: aggregated,
                    });
                }
            }
        }

        let differences = if self.config.show_differences {
            Some(self.calculate_play_count_differences(&primary_period, &secondary_periods)?)
        } else {
            None
        };

        Ok(ComparisonResult {
            primary: primary_period,
            secondary: secondary_periods,
            config: self.config.clone(),
            differences,
        })
    }

    /// Compare day of week data across multiple periods
    pub async fn compare_day_of_week(
        &self,
        primary_data: Vec<HistoryEntry>,
        comparison_data: Vec<Vec<HistoryEntry>>,
    ) -> Result<ComparisonResult<DayOfWeekDataPoint>> {
        let primary_aggregated = self.aggregate_day_of_week(&primary_data)?;
        let primary_period = ComparisonPeriodData {
            label: "Primary Period".to_string(),
            date_range: self.config.primary_range.clone(),
            color: None,
            summary: self.calculate_summary_day_of_week(&primary_aggregated),
            data: primary_aggregated,
        };

        let mut secondary_periods = Vec::new();
        for (i, data) in comparison_data.iter().enumerate() {
            if let Some(period_config) = self.config.comparison_ranges.get(i) {
                if period_config.enabled {
                    let aggregated = self.aggregate_day_of_week(data)?;
                    secondary_periods.push(ComparisonPeriodData {
                        label: period_config.label.clone(),
                        date_range: period_config.date_range.clone(),
                        color: period_config.color.clone(),
                        summary: self.calculate_summary_day_of_week(&aggregated),
                        data: aggregated,
                    });
                }
            }
        }

        let differences = if self.config.show_differences {
            Some(self.calculate_day_of_week_differences(&primary_period, &secondary_periods)?)
        } else {
            None
        };

        Ok(ComparisonResult {
            primary: primary_period,
            secondary: secondary_periods,
            config: self.config.clone(),
            differences,
        })
    }

    /// Aggregate daily play count data for a specific date range
    fn aggregate_daily_play_counts(&self, entries: &[HistoryEntry], date_range: &DateRange) -> Result<Vec<PlayCountDataPoint>> {
        let mut daily_counts: HashMap<NaiveDate, u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some(date) = chrono::DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.naive_utc().date()) {
                    if date >= date_range.start && date <= date_range.end {
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

        result.sort_by_key(|point| point.date);
        Ok(result)
    }

    /// Aggregate day of week data
    fn aggregate_day_of_week(&self, entries: &[HistoryEntry]) -> Result<Vec<DayOfWeekDataPoint>> {
        let mut weekday_counts: HashMap<chrono::Weekday, u32> = HashMap::new();

        for entry in entries {
            if let Some(timestamp) = entry.date {
                if let Some(date) = chrono::DateTime::from_timestamp(timestamp, 0)
                    .map(|dt| dt.naive_utc().date()) {
                    let weekday = date.weekday();
                    *weekday_counts.entry(weekday).or_insert(0) += 1;
                }
            }
        }

        let total_count: u32 = weekday_counts.values().sum();
        let mut result: Vec<DayOfWeekDataPoint> = weekday_counts
            .into_iter()
            .map(|(weekday, count)| {
                let percentage = if total_count > 0 {
                    (count as f64 / total_count as f64) * 100.0
                } else {
                    0.0
                };
                DayOfWeekDataPoint {
                    weekday,
                    count,
                    label: Some(format!("{}: {} plays ({:.1}%)", weekday, count, percentage)),
                }
            })
            .collect();

        result.sort_by_key(|point| point.weekday.num_days_from_monday());
        Ok(result)
    }

    /// Calculate summary statistics for play count data
    fn calculate_summary_play_counts(&self, data: &[PlayCountDataPoint]) -> ComparisonSummary {
        if data.is_empty() {
            return ComparisonSummary {
                total: 0.0,
                average: 0.0,
                peak: 0.0,
                data_points: 0,
            };
        }

        let total = data.iter().map(|p| p.count as f64).sum();
        let peak = data.iter().map(|p| p.count as f64).fold(0.0, f64::max);
        let average = total / data.len() as f64;

        ComparisonSummary {
            total,
            average,
            peak,
            data_points: data.len(),
        }
    }

    /// Calculate summary statistics for day of week data
    fn calculate_summary_day_of_week(&self, data: &[DayOfWeekDataPoint]) -> ComparisonSummary {
        if data.is_empty() {
            return ComparisonSummary {
                total: 0.0,
                average: 0.0,
                peak: 0.0,
                data_points: 0,
            };
        }

        let total = data.iter().map(|p| p.count as f64).sum();
        let peak = data.iter().map(|p| p.count as f64).fold(0.0, f64::max);
        let average = total / data.len() as f64;

        ComparisonSummary {
            total,
            average,
            peak,
            data_points: data.len(),
        }
    }

    /// Calculate differences between play count periods
    fn calculate_play_count_differences(
        &self,
        primary: &ComparisonPeriodData<PlayCountDataPoint>,
        secondary: &[ComparisonPeriodData<PlayCountDataPoint>],
    ) -> Result<Vec<ComparisonDifference>> {
        let mut differences = Vec::new();

        for period in secondary {
            let absolute_diff = period.summary.total - primary.summary.total;
            let percentage_diff = if primary.summary.total != 0.0 {
                (absolute_diff / primary.summary.total) * 100.0
            } else {
                0.0
            };

            let growth_direction = if absolute_diff > 0.0 {
                GrowthDirection::Positive
            } else if absolute_diff < 0.0 {
                GrowthDirection::Negative
            } else {
                GrowthDirection::Neutral
            };

            differences.push(ComparisonDifference {
                label: format!("{} vs Primary", period.label),
                absolute_difference: absolute_diff,
                percentage_difference: percentage_diff,
                growth_direction,
            });
        }

        Ok(differences)
    }

    /// Calculate differences between day of week periods
    fn calculate_day_of_week_differences(
        &self,
        primary: &ComparisonPeriodData<DayOfWeekDataPoint>,
        secondary: &[ComparisonPeriodData<DayOfWeekDataPoint>],
    ) -> Result<Vec<ComparisonDifference>> {
        let mut differences = Vec::new();

        for period in secondary {
            let absolute_diff = period.summary.total - primary.summary.total;
            let percentage_diff = if primary.summary.total != 0.0 {
                (absolute_diff / primary.summary.total) * 100.0
            } else {
                0.0
            };

            let growth_direction = if absolute_diff > 0.0 {
                GrowthDirection::Positive
            } else if absolute_diff < 0.0 {
                GrowthDirection::Negative
            } else {
                GrowthDirection::Neutral
            };

            differences.push(ComparisonDifference {
                label: format!("{} vs Primary", period.label),
                absolute_difference: absolute_diff,
                percentage_difference: percentage_diff,
                growth_direction,
            });
        }

        Ok(differences)
    }

    /// Create a comparison configuration with common time periods
    pub fn create_year_over_year_comparison(base_date_range: DateRange) -> ComparisonConfig {
        let days_in_range = (base_date_range.end - base_date_range.start).num_days();
        
        // Create previous year comparison
        let previous_year_start = base_date_range.start - chrono::Duration::days(365);
        let previous_year_end = previous_year_start + chrono::Duration::days(days_in_range);

        ComparisonConfig {
            enabled: true,
            primary_range: base_date_range,
            comparison_ranges: vec![
                ComparisonPeriod {
                    label: "Previous Year".to_string(),
                    date_range: DateRange::new(previous_year_start, previous_year_end),
                    color: Some("#FF6B6B".to_string()),
                    enabled: true,
                },
            ],
            display_mode: ComparisonDisplayMode::Overlay,
            show_differences: true,
            show_growth_percentages: true,
            comparison_colors: vec![
                "#FF6B6B".to_string(),
                "#4ECDC4".to_string(),
            ],
        }
    }

    /// Create a comparison configuration for month-over-month
    pub fn create_month_over_month_comparison(base_date_range: DateRange) -> ComparisonConfig {
        let days_in_range = (base_date_range.end - base_date_range.start).num_days();
        
        // Create previous month comparison
        let previous_month_start = base_date_range.start - chrono::Duration::days(30);
        let previous_month_end = previous_month_start + chrono::Duration::days(days_in_range);

        ComparisonConfig {
            enabled: true,
            primary_range: base_date_range,
            comparison_ranges: vec![
                ComparisonPeriod {
                    label: "Previous Month".to_string(),
                    date_range: DateRange::new(previous_month_start, previous_month_end),
                    color: Some("#4ECDC4".to_string()),
                    enabled: true,
                },
            ],
            display_mode: ComparisonDisplayMode::Overlay,
            show_differences: true,
            show_growth_percentages: true,
            comparison_colors: vec![
                "#4ECDC4".to_string(),
                "#45B7D1".to_string(),
            ],
        }
    }

    /// Create a comparison configuration for week-over-week
    pub fn create_week_over_week_comparison(base_date_range: DateRange) -> ComparisonConfig {
        let days_in_range = (base_date_range.end - base_date_range.start).num_days();
        
        // Create previous week comparison
        let previous_week_start = base_date_range.start - chrono::Duration::days(7);
        let previous_week_end = previous_week_start + chrono::Duration::days(days_in_range);

        ComparisonConfig {
            enabled: true,
            primary_range: base_date_range,
            comparison_ranges: vec![
                ComparisonPeriod {
                    label: "Previous Week".to_string(),
                    date_range: DateRange::new(previous_week_start, previous_week_end),
                    color: Some("#45B7D1".to_string()),
                    enabled: true,
                },
            ],
            display_mode: ComparisonDisplayMode::Overlay,
            show_differences: true,
            show_growth_percentages: true,
            comparison_colors: vec![
                "#45B7D1".to_string(),
                "#FFA07A".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tgraph_common::HistoryEntry;

    fn create_test_history_entry(date: i64, user_id: i32, username: &str) -> HistoryEntry {
        HistoryEntry {
            date: Some(date),
            user_id: Some(user_id),
            username: Some(username.to_string()),
            platform: Some("Test Platform".to_string()),
            friendly_name: Some("Test User".to_string()),
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

    #[test]
    fn test_comparison_config_creation() {
        let base_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );

        let config = ComparisonManager::create_year_over_year_comparison(base_range);
        assert!(config.enabled);
        assert_eq!(config.comparison_ranges.len(), 1);
        assert_eq!(config.comparison_ranges[0].label, "Previous Year");
    }

    #[test]
    fn test_play_count_aggregation() {
        let date_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
        );

        let config = ComparisonConfig::default();
        let manager = ComparisonManager::new(config);

        let entries = vec![
            create_test_history_entry(1704067200, 1, "user1"), // 2024-01-01
            create_test_history_entry(1704067200, 2, "user2"), // 2024-01-01  
            create_test_history_entry(1704153600, 1, "user1"), // 2024-01-02
        ];

        let result = manager.aggregate_daily_play_counts(&entries, &date_range).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].count, 2); // Two plays on 2024-01-01
        assert_eq!(result[1].count, 1); // One play on 2024-01-02
    }

    #[test]
    fn test_summary_calculation() {
        let data = vec![
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), count: 10, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), count: 20, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), count: 15, label: None },
        ];

        let config = ComparisonConfig::default();
        let manager = ComparisonManager::new(config);
        let summary = manager.calculate_summary_play_counts(&data);

        assert_eq!(summary.total, 45.0);
        assert_eq!(summary.average, 15.0);
        assert_eq!(summary.peak, 20.0);
        assert_eq!(summary.data_points, 3);
    }
} 