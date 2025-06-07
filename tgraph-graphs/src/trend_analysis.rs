//! Trend analysis and statistical calculations for graph data

use crate::{PlayCountDataPoint, TrendConfig};
use chrono::{Datelike, NaiveDate};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tgraph_common::Result;

/// Statistical indicators for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendStatistics {
    /// Minimum value in the dataset
    pub min: f64,
    /// Maximum value in the dataset
    pub max: f64,
    /// Mean (average) value
    pub mean: f64,
    /// Median value
    pub median: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Total sum of values
    pub sum: f64,
    /// Total count of data points
    pub count: usize,
}

/// Growth rate analysis between time periods
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrowthAnalysis {
    /// Growth rate as a percentage
    pub growth_rate: f64,
    /// Absolute change in value
    pub absolute_change: f64,
    /// Period-over-period comparison
    pub period_comparison: String,
    /// Trend direction
    pub trend_direction: TrendDirection,
}

/// Direction of trend movement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

/// Moving average data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MovingAveragePoint {
    pub date: NaiveDate,
    pub value: f64,
    pub window_size: u32,
}

/// Trend line data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendLinePoint {
    pub date: NaiveDate,
    pub value: f64,
    pub confidence_interval_upper: Option<f64>,
    pub confidence_interval_lower: Option<f64>,
}

/// Complete trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysisResult {
    /// Basic statistical indicators
    pub statistics: TrendStatistics,
    /// Growth analysis
    pub growth_analysis: Option<GrowthAnalysis>,
    /// Moving average data points
    pub moving_averages: Vec<MovingAveragePoint>,
    /// Trend line points
    pub trend_line: Vec<TrendLinePoint>,
    /// Seasonal patterns (if detected)
    pub seasonal_patterns: Option<SeasonalPattern>,
}

/// Detected seasonal patterns in the data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPattern {
    /// Pattern type (weekly, monthly, yearly)
    pub pattern_type: SeasonalPatternType,
    /// Strength of the pattern (0.0 to 1.0)
    pub strength: f64,
    /// Peak periods
    pub peak_periods: Vec<String>,
    /// Low periods
    pub low_periods: Vec<String>,
}

/// Types of seasonal patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SeasonalPatternType {
    Weekly,
    Monthly,
    Quarterly,
    Yearly,
}

/// Trend analyzer for processing graph data
pub struct TrendAnalyzer {
    config: TrendConfig,
}

impl TrendAnalyzer {
    pub fn new(config: TrendConfig) -> Self {
        Self { config }
    }

    /// Analyze trends in play count data
    pub fn analyze_play_counts(&self, data: &[PlayCountDataPoint]) -> Result<TrendAnalysisResult> {
        let values: Vec<f64> = data.iter().map(|point| point.count as f64).collect();
        let dates: Vec<NaiveDate> = data.iter().map(|point| point.date).collect();

        let statistics = self.calculate_statistics(&values)?;
        let growth_analysis = if self.config.show_growth_rate {
            Some(self.calculate_growth_analysis(&values)?)
        } else {
            None
        };

        let moving_averages = if self.config.show_moving_average {
            self.calculate_moving_averages(&values, &dates)?
        } else {
            vec![]
        };

        let trend_line = if self.config.show_trend_line {
            self.calculate_trend_line(&values, &dates)?
        } else {
            vec![]
        };

        let seasonal_patterns = self.detect_seasonal_patterns(&values, &dates)?;

        Ok(TrendAnalysisResult {
            statistics,
            growth_analysis,
            moving_averages,
            trend_line,
            seasonal_patterns,
        })
    }

    /// Calculate basic statistical indicators
    fn calculate_statistics(&self, values: &[f64]) -> Result<TrendStatistics> {
        if values.is_empty() {
            return Err(tgraph_common::TGraphError::config("Cannot calculate statistics for empty dataset"));
        }

        let mut sorted_values = values.to_vec();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());

        let min = *sorted_values.first().unwrap();
        let max = *sorted_values.last().unwrap();
        let sum: f64 = values.iter().sum();
        let count = values.len();
        let mean = sum / count as f64;

        let median = if count % 2 == 0 {
            (sorted_values[count / 2 - 1] + sorted_values[count / 2]) / 2.0
        } else {
            sorted_values[count / 2]
        };

        let variance: f64 = values.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let std_dev = variance.sqrt();

        Ok(TrendStatistics {
            min,
            max,
            mean,
            median,
            std_dev,
            sum,
            count,
        })
    }

    /// Calculate growth analysis
    fn calculate_growth_analysis(&self, values: &[f64]) -> Result<GrowthAnalysis> {
        if values.len() < 2 {
            return Err(tgraph_common::TGraphError::config("Need at least 2 data points for growth analysis"));
        }

        let first_value = values[0];
        let last_value = values[values.len() - 1];
        let absolute_change = last_value - first_value;
        
        let growth_rate = if first_value != 0.0 {
            (absolute_change / first_value) * 100.0
        } else {
            0.0
        };

        let trend_direction = self.determine_trend_direction(values);
        
        Ok(GrowthAnalysis {
            growth_rate,
            absolute_change,
            period_comparison: format!("From {} to {}", first_value, last_value),
            trend_direction,
        })
    }

    /// Determine the overall trend direction
    fn determine_trend_direction(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 3 {
            return TrendDirection::Stable;
        }

        let mut increasing_count = 0;
        let mut decreasing_count = 0;

        for window in values.windows(2) {
            if window[1] > window[0] {
                increasing_count += 1;
            } else if window[1] < window[0] {
                decreasing_count += 1;
            }
        }

        let total_comparisons = values.len() - 1;
        let increasing_ratio = increasing_count as f64 / total_comparisons as f64;
        let decreasing_ratio = decreasing_count as f64 / total_comparisons as f64;

        if increasing_ratio > 0.6 {
            TrendDirection::Increasing
        } else if decreasing_ratio > 0.6 {
            TrendDirection::Decreasing
        } else if (increasing_ratio - decreasing_ratio).abs() < 0.2 {
            TrendDirection::Volatile
        } else {
            TrendDirection::Stable
        }
    }

    /// Calculate moving averages
    fn calculate_moving_averages(&self, values: &[f64], dates: &[NaiveDate]) -> Result<Vec<MovingAveragePoint>> {
        let window_size = self.config.moving_average_window as usize;
        
        if values.len() < window_size {
            return Ok(vec![]);
        }

        let mut moving_averages = Vec::new();

        for i in window_size..=values.len() {
            let window = &values[i - window_size..i];
            let average = window.iter().sum::<f64>() / window.len() as f64;
            
            moving_averages.push(MovingAveragePoint {
                date: dates[i - 1],
                value: average,
                window_size: self.config.moving_average_window,
            });
        }

        Ok(moving_averages)
    }

    /// Calculate trend line using linear regression
    fn calculate_trend_line(&self, values: &[f64], dates: &[NaiveDate]) -> Result<Vec<TrendLinePoint>> {
        if values.len() < 2 {
            return Ok(vec![]);
        }

        // Convert dates to numeric values (days since first date)
        let first_date = dates[0];
        let x_values: Vec<f64> = dates.iter()
            .map(|date| (*date - first_date).num_days() as f64)
            .collect();

        // Calculate linear regression
        let n = values.len() as f64;
        let sum_x: f64 = x_values.iter().sum();
        let sum_y: f64 = values.iter().sum();
        let sum_xy: f64 = x_values.iter().zip(values.iter()).map(|(x, y)| x * y).sum();
        let sum_x_squared: f64 = x_values.iter().map(|x| x.powi(2)).sum();

        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x_squared - sum_x.powi(2));
        let intercept = (sum_y - slope * sum_x) / n;

        // Calculate confidence intervals if enabled
        let confidence_interval = if self.config.confidence_interval > 0.0 {
            Some(self.calculate_confidence_interval(values, &x_values, slope, intercept))
        } else {
            None
        };

        let mut trend_points = Vec::new();
        for (i, &date) in dates.iter().enumerate() {
            let predicted_value = slope * x_values[i] + intercept;
            
            let (upper, lower) = if let Some(ci) = confidence_interval {
                (Some(predicted_value + ci), Some(predicted_value - ci))
            } else {
                (None, None)
            };

            trend_points.push(TrendLinePoint {
                date,
                value: predicted_value,
                confidence_interval_upper: upper,
                confidence_interval_lower: lower,
            });
        }

        Ok(trend_points)
    }

    /// Calculate confidence interval for trend line
    fn calculate_confidence_interval(&self, values: &[f64], x_values: &[f64], slope: f64, intercept: f64) -> f64 {
        // Simplified confidence interval calculation
        let n = values.len() as f64;
        let predictions: Vec<f64> = x_values.iter().map(|&x| slope * x + intercept).collect();
        
        let residual_sum_squares: f64 = values.iter()
            .zip(predictions.iter())
            .map(|(actual, predicted)| (actual - predicted).powi(2))
            .sum();

        let standard_error = (residual_sum_squares / (n - 2.0)).sqrt();
        
        // Use t-distribution critical value (approximated for 95% confidence)
        let t_critical = 1.96; // For large samples, approximates t-distribution
        
        t_critical * standard_error
    }

    /// Detect seasonal patterns in the data
    fn detect_seasonal_patterns(&self, values: &[f64], dates: &[NaiveDate]) -> Result<Option<SeasonalPattern>> {
        if values.len() < 14 {  // Need at least 2 weeks of data
            return Ok(None);
        }

        // Try to detect weekly patterns
        if let Some(weekly_pattern) = self.detect_weekly_pattern(values, dates)? {
            return Ok(Some(weekly_pattern));
        }

        // Try to detect monthly patterns if we have enough data
        if values.len() >= 60 {  // Need at least 2 months
            if let Some(monthly_pattern) = self.detect_monthly_pattern(values, dates)? {
                return Ok(Some(monthly_pattern));
            }
        }

        Ok(None)
    }

    /// Detect weekly seasonal patterns
    fn detect_weekly_pattern(&self, values: &[f64], dates: &[NaiveDate]) -> Result<Option<SeasonalPattern>> {
        let mut weekly_data: HashMap<u32, Vec<f64>> = HashMap::new();
        
        for (value, date) in values.iter().zip(dates.iter()) {
            let weekday = date.weekday().num_days_from_monday();
            weekly_data.entry(weekday).or_default().push(*value);
        }

        if weekly_data.len() < 7 {
            return Ok(None);
        }

        // Calculate average for each day of the week
        let mut day_averages: Vec<(u32, f64)> = weekly_data.iter()
            .map(|(&day, values)| {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                (day, avg)
            })
            .collect();
        
        day_averages.sort_by_key(|&(day, _)| day);

        // Calculate pattern strength (coefficient of variation)
        let values_only: Vec<f64> = day_averages.iter().map(|(_, avg)| *avg).collect();
        let mean = values_only.iter().sum::<f64>() / values_only.len() as f64;
        let variance = values_only.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values_only.len() as f64;
        let coefficient_of_variation = variance.sqrt() / mean;

        if coefficient_of_variation > 0.1 {  // Threshold for significant pattern
            let max_day = day_averages.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
            let min_day = day_averages.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();

            let day_names = ["Monday", "Tuesday", "Wednesday", "Thursday", "Friday", "Saturday", "Sunday"];
            
            Ok(Some(SeasonalPattern {
                pattern_type: SeasonalPatternType::Weekly,
                strength: coefficient_of_variation.min(1.0),
                peak_periods: vec![day_names[max_day.0 as usize].to_string()],
                low_periods: vec![day_names[min_day.0 as usize].to_string()],
            }))
        } else {
            Ok(None)
        }
    }

    /// Detect monthly seasonal patterns
    fn detect_monthly_pattern(&self, values: &[f64], dates: &[NaiveDate]) -> Result<Option<SeasonalPattern>> {
        let mut monthly_data: HashMap<u32, Vec<f64>> = HashMap::new();
        
        for (value, date) in values.iter().zip(dates.iter()) {
            let month = date.month();
            monthly_data.entry(month).or_default().push(*value);
        }

        if monthly_data.len() < 3 {
            return Ok(None);
        }

        // Calculate average for each month
        let mut month_averages: Vec<(u32, f64)> = monthly_data.iter()
            .map(|(&month, values)| {
                let avg = values.iter().sum::<f64>() / values.len() as f64;
                (month, avg)
            })
            .collect();
        
        month_averages.sort_by_key(|&(month, _)| month);

        // Calculate pattern strength
        let values_only: Vec<f64> = month_averages.iter().map(|(_, avg)| *avg).collect();
        let mean = values_only.iter().sum::<f64>() / values_only.len() as f64;
        let variance = values_only.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / values_only.len() as f64;
        let coefficient_of_variation = variance.sqrt() / mean;

        if coefficient_of_variation > 0.15 {  // Higher threshold for monthly patterns
            let max_month = month_averages.iter().max_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();
            let min_month = month_averages.iter().min_by(|a, b| a.1.partial_cmp(&b.1).unwrap()).unwrap();

            let month_names = [
                "January", "February", "March", "April", "May", "June",
                "July", "August", "September", "October", "November", "December"
            ];
            
            Ok(Some(SeasonalPattern {
                pattern_type: SeasonalPatternType::Monthly,
                strength: coefficient_of_variation.min(1.0),
                peak_periods: vec![month_names[(max_month.0 - 1) as usize].to_string()],
                low_periods: vec![month_names[(min_month.0 - 1) as usize].to_string()],
            }))
        } else {
            Ok(None)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    fn create_test_data() -> Vec<PlayCountDataPoint> {
        vec![
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), count: 10, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), count: 15, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), count: 12, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(), count: 18, label: None },
            PlayCountDataPoint { date: NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(), count: 20, label: None },
        ]
    }

    #[test]
    fn test_trend_analysis_basic() {
        let config = TrendConfig {
            enabled: true,
            show_moving_average: true,
            moving_average_window: 3,
            show_trend_line: true,
            show_growth_rate: true,
            show_statistics: true,
            confidence_interval: 0.95,
        };

        let analyzer = TrendAnalyzer::new(config);
        let data = create_test_data();
        let result = analyzer.analyze_play_counts(&data).unwrap();

        assert_eq!(result.statistics.count, 5);
        assert_eq!(result.statistics.min, 10.0);
        assert_eq!(result.statistics.max, 20.0);
        assert!(result.growth_analysis.is_some());
        assert!(!result.moving_averages.is_empty());
        assert!(!result.trend_line.is_empty());
    }

    #[test]
    fn test_moving_average_calculation() {
        let values = vec![10.0, 15.0, 12.0, 18.0, 20.0];
        let dates = vec![
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 4).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        ];

        let config = TrendConfig {
            moving_average_window: 3,
            ..Default::default()
        };

        let analyzer = TrendAnalyzer::new(config);
        let result = analyzer.calculate_moving_averages(&values, &dates).unwrap();

        assert_eq!(result.len(), 3);
        assert_eq!(result[0].value, (10.0 + 15.0 + 12.0) / 3.0);
        assert_eq!(result[1].value, (15.0 + 12.0 + 18.0) / 3.0);
        assert_eq!(result[2].value, (12.0 + 18.0 + 20.0) / 3.0);
    }

    #[test]
    fn test_statistics_calculation() {
        let values = vec![10.0, 15.0, 12.0, 18.0, 20.0];
        let config = TrendConfig::default();
        let analyzer = TrendAnalyzer::new(config);
        
        let stats = analyzer.calculate_statistics(&values).unwrap();
        
        assert_eq!(stats.min, 10.0);
        assert_eq!(stats.max, 20.0);
        assert_eq!(stats.mean, 15.0);
        assert_eq!(stats.median, 15.0);
        assert_eq!(stats.sum, 75.0);
        assert_eq!(stats.count, 5);
    }
} 