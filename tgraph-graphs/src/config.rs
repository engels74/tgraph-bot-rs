//! Graph configuration and customization system

use crate::{ColorScheme, GraphConfig, StyleConfig};
use chrono::{DateTime, Datelike, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tgraph_common::Result;

/// Configuration specific to different graph types
pub trait GraphSpecificConfig: Clone + Serialize + for<'de> Deserialize<'de> {
    /// Get default configuration for this graph type
    fn default_config() -> Self;
    
    /// Validate the configuration settings
    fn validate(&self) -> Result<()>;
    
    /// Apply configuration to base GraphConfig
    fn apply_to_graph_config(&self, config: &mut GraphConfig);
    
    /// Get configuration display name
    fn display_name(&self) -> &'static str;
}

/// Time range presets for easy selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimeRangePreset {
    LastWeek,
    LastMonth,
    LastQuarter,
    LastYear,
    LastSevenDays,
    LastThirtyDays,
    LastNinetyDays,
    ThisMonth,
    ThisYear,
    AllTime,
    Custom(DateRange),
}

impl TimeRangePreset {
    pub fn to_date_range(&self) -> DateRange {
        let now = chrono::Utc::now().date_naive();
        
        match self {
            TimeRangePreset::LastWeek => DateRange::last_days(7),
            TimeRangePreset::LastMonth => DateRange::last_days(30),
            TimeRangePreset::LastQuarter => DateRange::last_days(90),
            TimeRangePreset::LastYear => DateRange::last_days(365),
            TimeRangePreset::LastSevenDays => DateRange::last_days(7),
            TimeRangePreset::LastThirtyDays => DateRange::last_days(30),
            TimeRangePreset::LastNinetyDays => DateRange::last_days(90),
            TimeRangePreset::ThisMonth => {
                let start = now.with_day(1).unwrap();
                DateRange::new(start, now)
            },
            TimeRangePreset::ThisYear => {
                let start = now.with_month(1).unwrap().with_day(1).unwrap();
                DateRange::new(start, now)
            },
            TimeRangePreset::AllTime => {
                // Default to last 2 years for "all time"
                DateRange::last_days(730)
            },
            TimeRangePreset::Custom(range) => range.clone(),
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            TimeRangePreset::LastWeek => "Last Week",
            TimeRangePreset::LastMonth => "Last Month",
            TimeRangePreset::LastQuarter => "Last Quarter",
            TimeRangePreset::LastYear => "Last Year",
            TimeRangePreset::LastSevenDays => "Last 7 Days",
            TimeRangePreset::LastThirtyDays => "Last 30 Days",
            TimeRangePreset::LastNinetyDays => "Last 90 Days",
            TimeRangePreset::ThisMonth => "This Month",
            TimeRangePreset::ThisYear => "This Year",
            TimeRangePreset::AllTime => "All Time",
            TimeRangePreset::Custom(_) => "Custom Range",
        }
    }
}

/// Configuration for graph comparison features
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonConfig {
    /// Enable comparison mode
    pub enabled: bool,
    /// Primary time range for comparison
    pub primary_range: DateRange,
    /// Secondary time range(s) for comparison
    pub comparison_ranges: Vec<ComparisonPeriod>,
    /// Comparison display mode
    pub display_mode: ComparisonDisplayMode,
    /// Show difference indicators
    pub show_differences: bool,
    /// Show growth percentages
    pub show_growth_percentages: bool,
    /// Colors for different comparison periods
    pub comparison_colors: Vec<String>,
}

/// Individual comparison period configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparisonPeriod {
    /// Name/label for this comparison period
    pub label: String,
    /// Date range for this comparison
    pub date_range: DateRange,
    /// Color for this comparison (optional, uses default palette if None)
    pub color: Option<String>,
    /// Whether this comparison is enabled
    pub enabled: bool,
}

/// How to display comparison data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComparisonDisplayMode {
    /// Overlay all periods on the same graph
    Overlay,
    /// Side-by-side subplots
    SideBySide,
    /// Stacked display
    Stacked,
    /// Show as difference/delta from primary
    Difference,
}

/// Configuration for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendConfig {
    /// Enable trend analysis
    pub enabled: bool,
    /// Show moving averages
    pub show_moving_average: bool,
    /// Moving average window size (in days)
    pub moving_average_window: u32,
    /// Show linear trend line
    pub show_trend_line: bool,
    /// Show growth rate indicators
    pub show_growth_rate: bool,
    /// Show statistical indicators (min, max, mean, median)
    pub show_statistics: bool,
    /// Confidence interval for trend predictions
    pub confidence_interval: f64,
}

impl Default for ComparisonConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            primary_range: DateRange::last_days(30),
            comparison_ranges: vec![],
            display_mode: ComparisonDisplayMode::Overlay,
            show_differences: true,
            show_growth_percentages: true,
            comparison_colors: vec![
                "#FF6B6B".to_string(),
                "#4ECDC4".to_string(),
                "#45B7D1".to_string(),
                "#FFA07A".to_string(),
                "#98D8C8".to_string(),
            ],
        }
    }
}

impl Default for TrendConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            show_moving_average: false,
            moving_average_window: 7,
            show_trend_line: false,
            show_growth_rate: false,
            show_statistics: false,
            confidence_interval: 0.95,
        }
    }
}

/// Data filtering options for graph generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    /// Date range filter
    pub date_range: Option<DateRange>,
    /// Time range preset for easy selection
    pub time_range_preset: Option<TimeRangePreset>,
    /// Platform filter (include only these platforms)
    pub platforms: Option<Vec<String>>,
    /// User filter (include only these users)
    pub users: Option<Vec<String>>,
    /// Maximum number of data points to display
    pub data_point_limit: Option<u32>,
    /// Minimum threshold for data inclusion
    pub minimum_threshold: Option<f64>,
    /// Custom data filters
    pub custom_filters: HashMap<String, String>,
    /// Comparison configuration
    pub comparison: Option<ComparisonConfig>,
    /// Trend analysis configuration
    pub trend_analysis: Option<TrendConfig>,
}

impl Default for FilterConfig {
    fn default() -> Self {
        Self {
            date_range: None,
            time_range_preset: None,
            platforms: None,
            users: None,
            data_point_limit: Some(100), // Default limit
            minimum_threshold: None,
            custom_filters: HashMap::new(),
            comparison: None,
            trend_analysis: None,
        }
    }
}

/// Date range specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: NaiveDate,
    pub end: NaiveDate,
}

impl DateRange {
    pub fn new(start: NaiveDate, end: NaiveDate) -> Self {
        Self { start, end }
    }
    
    pub fn last_days(days: u32) -> Self {
        let end = chrono::Utc::now().date_naive();
        let start = end - chrono::Duration::days(days as i64);
        Self { start, end }
    }
    
    pub fn last_months(months: u32) -> Self {
        let end = chrono::Utc::now().date_naive();
        let start = end - chrono::Duration::days((months * 30) as i64);
        Self { start, end }
    }
}

/// Sort order options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortOrder {
    Ascending,
    Descending,
    Alphabetical,
    ReverseAlphabetical,
    Custom(Vec<String>),
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::Descending
    }
}

/// Display preferences for graph rendering
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    /// Show data labels on points
    pub show_data_labels: bool,
    /// Show legend
    pub show_legend: bool,
    /// Show grid lines
    pub show_grid: bool,
    /// Show trend lines
    pub show_trends: bool,
    /// Animation settings (for future frontend integration)
    pub enable_animations: bool,
    /// Sort order for data
    pub sort_order: SortOrder,
}

impl Default for DisplayConfig {
    fn default() -> Self {
        Self {
            show_data_labels: false,
            show_legend: true,
            show_grid: true,
            show_trends: false,
            enable_animations: true,
            sort_order: SortOrder::default(),
        }
    }
}

/// Configuration for daily play count graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPlayCountConfig {
    /// Show weekend highlighting
    pub highlight_weekends: bool,
    /// Show moving average line
    pub show_moving_average: bool,
    /// Moving average window size
    pub moving_average_days: u32,
    /// Show growth trends
    pub show_growth_trends: bool,
    /// Color for weekends
    pub weekend_color: Option<String>,
    /// Line thickness
    pub line_thickness: u32,
    /// Show data point markers
    pub show_markers: bool,
}

impl Default for DailyPlayCountConfig {
    fn default() -> Self {
        Self {
            highlight_weekends: true,
            show_moving_average: false,
            moving_average_days: 7,
            show_growth_trends: false,
            weekend_color: Some("#ff6b6b".to_string()),
            line_thickness: 2,
            show_markers: true,
        }
    }
}

impl GraphSpecificConfig for DailyPlayCountConfig {
    fn default_config() -> Self {
        Self::default()
    }
    
    fn validate(&self) -> Result<()> {
        if self.moving_average_days == 0 || self.moving_average_days > 365 {
            return Err(tgraph_common::TGraphError::config(
                "Moving average days must be between 1 and 365"
            ));
        }
        if self.line_thickness == 0 || self.line_thickness > 10 {
            return Err(tgraph_common::TGraphError::config(
                "Line thickness must be between 1 and 10"
            ));
        }
        Ok(())
    }
    
    fn apply_to_graph_config(&self, _config: &mut GraphConfig) {
        // Apply weekend highlighting if enabled
        if self.highlight_weekends && self.weekend_color.is_some() {
            // This would require extending GraphConfig to support weekend highlighting
            // For now, we can store this in custom properties
        }
    }
    
    fn display_name(&self) -> &'static str {
        "Daily Play Count"
    }
}

/// Configuration for day of week graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayOfWeekConfig {
    /// Start week on Monday vs Sunday
    pub start_week_monday: bool,
    /// Show percentages instead of counts
    pub show_percentages: bool,
    /// Highlight weekends
    pub highlight_weekends: bool,
    /// Bar width adjustment
    pub bar_width_ratio: f64,
    /// Show average line
    pub show_average_line: bool,
}

impl Default for DayOfWeekConfig {
    fn default() -> Self {
        Self {
            start_week_monday: true,
            show_percentages: false,
            highlight_weekends: true,
            bar_width_ratio: 0.8,
            show_average_line: false,
        }
    }
}

impl GraphSpecificConfig for DayOfWeekConfig {
    fn default_config() -> Self {
        Self::default()
    }
    
    fn validate(&self) -> Result<()> {
        if self.bar_width_ratio <= 0.0 || self.bar_width_ratio > 1.0 {
            return Err(tgraph_common::TGraphError::config(
                "Bar width ratio must be between 0.0 and 1.0"
            ));
        }
        Ok(())
    }
    
    fn apply_to_graph_config(&self, _config: &mut GraphConfig) {
        // Graph-specific settings would be applied here
    }
    
    fn display_name(&self) -> &'static str {
        "Day of Week"
    }
}

/// Configuration for hourly distribution graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HourlyDistributionConfig {
    /// Use 24-hour format vs 12-hour
    pub use_24_hour_format: bool,
    /// Group hours into time periods
    pub group_time_periods: bool,
    /// Show peak hours highlighting
    pub highlight_peak_hours: bool,
    /// Peak hours threshold percentage
    pub peak_threshold_percent: f64,
    /// Smooth the distribution curve
    pub smooth_curve: bool,
}

impl Default for HourlyDistributionConfig {
    fn default() -> Self {
        Self {
            use_24_hour_format: true,
            group_time_periods: false,
            highlight_peak_hours: true,
            peak_threshold_percent: 80.0,
            smooth_curve: false,
        }
    }
}

impl GraphSpecificConfig for HourlyDistributionConfig {
    fn default_config() -> Self {
        Self::default()
    }
    
    fn validate(&self) -> Result<()> {
        if self.peak_threshold_percent < 0.0 || self.peak_threshold_percent > 100.0 {
            return Err(tgraph_common::TGraphError::config(
                "Peak threshold percentage must be between 0 and 100"
            ));
        }
        Ok(())
    }
    
    fn apply_to_graph_config(&self, _config: &mut GraphConfig) {
        // Graph-specific settings would be applied here
    }
    
    fn display_name(&self) -> &'static str {
        "Hourly Distribution"
    }
}

/// Configuration for monthly trends graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonthlyTrendsConfig {
    /// Show year-over-year comparison
    pub show_yoy_comparison: bool,
    /// Show seasonal trends
    pub show_seasonal_trends: bool,
    /// Aggregate by quarter instead of month
    pub quarterly_aggregation: bool,
    /// Show forecast projection
    pub show_forecast: bool,
    /// Forecast months ahead
    pub forecast_months: u32,
    /// Show growth percentage labels
    pub show_growth_labels: bool,
}

impl Default for MonthlyTrendsConfig {
    fn default() -> Self {
        Self {
            show_yoy_comparison: false,
            show_seasonal_trends: true,
            quarterly_aggregation: false,
            show_forecast: false,
            forecast_months: 3,
            show_growth_labels: true,
        }
    }
}

impl GraphSpecificConfig for MonthlyTrendsConfig {
    fn default_config() -> Self {
        Self::default()
    }
    
    fn validate(&self) -> Result<()> {
        if self.forecast_months > 12 {
            return Err(tgraph_common::TGraphError::config(
                "Forecast months cannot exceed 12"
            ));
        }
        Ok(())
    }
    
    fn apply_to_graph_config(&self, _config: &mut GraphConfig) {
        // Graph-specific settings would be applied here
    }
    
    fn display_name(&self) -> &'static str {
        "Monthly Trends"
    }
}

/// Configuration for top platforms/users graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopItemsConfig {
    /// Maximum number of items to show
    pub max_items: u32,
    /// Show percentages instead of raw counts
    pub show_percentages: bool,
    /// Show "Others" category for remaining items
    pub show_others_category: bool,
    /// Horizontal vs vertical bar orientation
    pub horizontal_bars: bool,
    /// Sort by count vs alphabetical
    pub sort_by_count: bool,
    /// Show data labels on bars
    pub show_data_labels: bool,
    /// Minimum threshold for inclusion
    pub minimum_count: u32,
}

impl Default for TopItemsConfig {
    fn default() -> Self {
        Self {
            max_items: 10,
            show_percentages: false,
            show_others_category: true,
            horizontal_bars: true,
            sort_by_count: true,
            show_data_labels: true,
            minimum_count: 0,
        }
    }
}

impl GraphSpecificConfig for TopItemsConfig {
    fn default_config() -> Self {
        Self::default()
    }
    
    fn validate(&self) -> Result<()> {
        if self.max_items == 0 || self.max_items > 100 {
            return Err(tgraph_common::TGraphError::config(
                "Max items must be between 1 and 100"
            ));
        }
        Ok(())
    }
    
    fn apply_to_graph_config(&self, _config: &mut GraphConfig) {
        // Graph-specific settings would be applied here
    }
    
    fn display_name(&self) -> &'static str {
        "Top Items"
    }
}

/// Complete graph configuration combining all aspects
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteGraphConfig {
    /// Base graph configuration (styling, dimensions)
    pub base: GraphConfig,
    /// Data filtering options
    pub filters: FilterConfig,
    /// Display preferences
    pub display: DisplayConfig,
    /// Graph-specific configuration (serialized as JSON)
    pub graph_specific: Option<serde_json::Value>,
    /// Configuration metadata
    pub metadata: ConfigMetadata,
}

/// Configuration metadata for tracking and management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Configuration name/title
    pub name: String,
    /// Description of the configuration
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last modified timestamp
    pub modified_at: DateTime<Utc>,
    /// Version for migration support
    pub version: String,
    /// Tags for organization
    pub tags: Vec<String>,
    /// Author/creator
    pub author: Option<String>,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        let now = Utc::now();
        Self {
            name: "Default Configuration".to_string(),
            description: None,
            created_at: now,
            modified_at: now,
            version: "1.0.0".to_string(),
            tags: vec![],
            author: None,
        }
    }
}

/// Configuration preset templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPreset {
    /// Preset metadata
    pub metadata: ConfigMetadata,
    /// The complete configuration
    pub config: CompleteGraphConfig,
}

/// Configuration manager for persistence and sharing
#[derive(Debug)]
pub struct ConfigurationManager {
    /// Default configurations per graph type
    defaults: HashMap<String, CompleteGraphConfig>,
    /// User-saved configurations
    saved_configs: HashMap<String, CompleteGraphConfig>,
    /// Available presets
    presets: Vec<ConfigPreset>,
}

impl ConfigurationManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            defaults: HashMap::new(),
            saved_configs: HashMap::new(),
            presets: Vec::new(),
        }
    }

    /// Load configuration from file
    pub async fn load_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let content = tokio::fs::read_to_string(path).await?;
        let config: CompleteGraphConfig = serde_json::from_str(&content)?;
        self.saved_configs.insert(config.metadata.name.clone(), config);
        Ok(())
    }

    /// Save configuration to file
    pub async fn save_to_file<P: AsRef<Path>>(&self, config: &CompleteGraphConfig, path: P) -> Result<()> {
        let content = serde_json::to_string_pretty(config)?;
        tokio::fs::write(path, content).await?;
        Ok(())
    }

    /// Export configuration as JSON string
    pub fn export_config(&self, config: &CompleteGraphConfig) -> Result<String> {
        Ok(serde_json::to_string_pretty(config)?)
    }

    /// Import configuration from JSON string
    pub fn import_config(&mut self, json: &str) -> Result<CompleteGraphConfig> {
        let config: CompleteGraphConfig = serde_json::from_str(json)?;
        self.saved_configs.insert(config.metadata.name.clone(), config.clone());
        Ok(config)
    }

    /// Get default configuration for graph type
    pub fn get_default_config(&self, graph_type: &str) -> Option<&CompleteGraphConfig> {
        self.defaults.get(graph_type)
    }

    /// Set default configuration for graph type
    pub fn set_default_config(&mut self, graph_type: String, config: CompleteGraphConfig) {
        self.defaults.insert(graph_type, config);
    }

    /// Save user configuration
    pub fn save_config(&mut self, name: String, config: CompleteGraphConfig) {
        self.saved_configs.insert(name, config);
    }

    /// Get saved configuration by name
    pub fn get_saved_config(&self, name: &str) -> Option<&CompleteGraphConfig> {
        self.saved_configs.get(name)
    }

    /// List all saved configuration names
    pub fn list_saved_configs(&self) -> Vec<&String> {
        self.saved_configs.keys().collect()
    }

    /// Add a preset configuration
    pub fn add_preset(&mut self, preset: ConfigPreset) {
        self.presets.push(preset);
    }

    /// Get all available presets
    pub fn get_presets(&self) -> &[ConfigPreset] {
        &self.presets
    }

    /// Create a configuration from preset
    pub fn create_from_preset(&self, preset_name: &str) -> Option<CompleteGraphConfig> {
        self.presets
            .iter()
            .find(|p| p.metadata.name == preset_name)
            .map(|p| p.config.clone())
    }

    /// Validate configuration
    pub fn validate_config(&self, config: &CompleteGraphConfig) -> Result<()> {
        // Validate base configuration
        if config.base.width == 0 || config.base.height == 0 {
            return Err(tgraph_common::TGraphError::config(
                "Graph dimensions must be greater than 0"
            ));
        }

        // Validate filters
        if let Some(limit) = config.filters.data_point_limit {
            if limit == 0 || limit > 10000 {
                return Err(tgraph_common::TGraphError::config(
                    "Data point limit must be between 1 and 10000"
                ));
            }
        }

        // Additional validation logic here
        Ok(())
    }

    /// Initialize with default presets
    pub fn initialize_defaults(&mut self) {
        // Add default presets for each graph type
        self.add_presentation_presets();
        self.add_report_presets();
        self.add_dashboard_presets();
    }

    /// Add presentation-style presets
    fn add_presentation_presets(&mut self) {
        // Dark theme for presentations
        let dark_preset = ConfigPreset {
            metadata: ConfigMetadata {
                name: "Dark Presentation".to_string(),
                description: Some("Dark theme optimized for presentations".to_string()),
                tags: vec!["dark".to_string(), "presentation".to_string()],
                ..Default::default()
            },
            config: CompleteGraphConfig {
                base: GraphConfig {
                    width: 1920,
                    height: 1080,
                    style: StyleConfig {
                        color_scheme: ColorScheme::Dark,
                        background_color: Some("#1a1a1a".to_string()),
                        title_font: crate::FontConfig {
                            family: "Arial".to_string(),
                            size: 24,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                filters: FilterConfig::default(),
                display: DisplayConfig {
                    show_legend: true,
                    show_grid: false,
                    show_data_labels: true,
                    ..Default::default()
                },
                graph_specific: None,
                metadata: ConfigMetadata::default(),
            },
        };
        self.add_preset(dark_preset);
    }

    /// Add report-style presets
    fn add_report_presets(&mut self) {
        // Clean report theme
        let report_preset = ConfigPreset {
            metadata: ConfigMetadata {
                name: "Clean Report".to_string(),
                description: Some("Professional theme for reports and documentation".to_string()),
                tags: vec!["light".to_string(), "report".to_string(), "professional".to_string()],
                ..Default::default()
            },
            config: CompleteGraphConfig {
                base: GraphConfig {
                    width: 800,
                    height: 600,
                    style: StyleConfig {
                        color_scheme: ColorScheme::Light,
                        background_color: Some("#ffffff".to_string()),
                        title_font: crate::FontConfig {
                            family: "Times New Roman".to_string(),
                            size: 18,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                filters: FilterConfig::default(),
                display: DisplayConfig {
                    show_legend: true,
                    show_grid: true,
                    show_data_labels: false,
                    ..Default::default()
                },
                graph_specific: None,
                metadata: ConfigMetadata::default(),
            },
        };
        self.add_preset(report_preset);
    }

    /// Add dashboard-style presets
    fn add_dashboard_presets(&mut self) {
        // Vibrant dashboard theme
        let dashboard_preset = ConfigPreset {
            metadata: ConfigMetadata {
                name: "Vibrant Dashboard".to_string(),
                description: Some("Colorful theme for interactive dashboards".to_string()),
                tags: vec!["vibrant".to_string(), "dashboard".to_string(), "interactive".to_string()],
                ..Default::default()
            },
            config: CompleteGraphConfig {
                base: GraphConfig {
                    width: 600,
                    height: 400,
                    style: StyleConfig {
                        color_scheme: ColorScheme::Vibrant,
                        background_color: Some("#f8f9fa".to_string()),
                        title_font: crate::FontConfig {
                            family: "Helvetica".to_string(),
                            size: 16,
                        },
                        ..Default::default()
                    },
                    ..Default::default()
                },
                filters: FilterConfig {
                    data_point_limit: Some(50),
                    ..Default::default()
                },
                display: DisplayConfig {
                    show_legend: false,
                    show_grid: true,
                    show_data_labels: true,
                    enable_animations: true,
                    ..Default::default()
                },
                graph_specific: None,
                metadata: ConfigMetadata::default(),
            },
        };
        self.add_preset(dashboard_preset);
    }
}

impl Default for ConfigurationManager {
    fn default() -> Self {
        let mut manager = Self::new();
        manager.initialize_defaults();
        manager
    }
}

impl Default for CompleteGraphConfig {
    fn default() -> Self {
        Self {
            base: GraphConfig::default(),
            filters: FilterConfig::default(),
            display: DisplayConfig::default(),
            graph_specific: None,
            metadata: ConfigMetadata::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_daily_play_count_config_default() {
        let config = DailyPlayCountConfig::default();
        assert!(config.highlight_weekends);
        assert!(!config.show_moving_average);
        assert_eq!(config.moving_average_days, 7);
        assert_eq!(config.line_thickness, 2);
    }

    #[test]
    fn test_daily_play_count_config_validation() {
        let mut config = DailyPlayCountConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid moving average days
        config.moving_average_days = 0;
        assert!(config.validate().is_err());
        
        config.moving_average_days = 500;
        assert!(config.validate().is_err());
        
        // Reset and test line thickness
        config.moving_average_days = 7;
        config.line_thickness = 0;
        assert!(config.validate().is_err());
        
        config.line_thickness = 15;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_day_of_week_config_validation() {
        let mut config = DayOfWeekConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid bar width ratio
        config.bar_width_ratio = 0.0;
        assert!(config.validate().is_err());
        
        config.bar_width_ratio = 1.5;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_hourly_distribution_config_validation() {
        let mut config = HourlyDistributionConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid peak threshold
        config.peak_threshold_percent = -5.0;
        assert!(config.validate().is_err());
        
        config.peak_threshold_percent = 150.0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_monthly_trends_config_validation() {
        let mut config = MonthlyTrendsConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid forecast months
        config.forecast_months = 15;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_top_items_config_validation() {
        let mut config = TopItemsConfig::default();
        
        // Valid configuration should pass
        assert!(config.validate().is_ok());
        
        // Invalid max items
        config.max_items = 0;
        assert!(config.validate().is_err());
        
        config.max_items = 150;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_date_range_creation() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
        let range = DateRange::new(start, end);
        
        assert_eq!(range.start, start);
        assert_eq!(range.end, end);
    }

    #[test]
    fn test_date_range_last_days() {
        let range = DateRange::last_days(30);
        let days_diff = range.end.signed_duration_since(range.start).num_days();
        assert_eq!(days_diff, 30);
    }

    #[test]
    fn test_configuration_manager_creation() {
        let manager = ConfigurationManager::new();
        assert_eq!(manager.list_saved_configs().len(), 0);
        assert_eq!(manager.get_presets().len(), 0);
    }

    #[test]
    fn test_configuration_manager_defaults() {
        let manager = ConfigurationManager::default();
        assert!(manager.get_presets().len() > 0);
    }

    #[test]
    fn test_config_export_import() {
        let mut manager = ConfigurationManager::new();
        let config = CompleteGraphConfig::default();
        
        // Export to JSON
        let json = manager.export_config(&config).unwrap();
        assert!(!json.is_empty());
        
        // Import from JSON
        let imported = manager.import_config(&json).unwrap();
        assert_eq!(imported.metadata.name, config.metadata.name);
    }

    #[tokio::test]
    async fn test_save_and_load_config() {
        let manager = ConfigurationManager::new();
        let config = CompleteGraphConfig::default();
        let temp_file = NamedTempFile::new().unwrap();
        
        // Save config
        manager.save_to_file(&config, temp_file.path()).await.unwrap();
        
        // Load config
        let mut manager2 = ConfigurationManager::new();
        manager2.load_from_file(temp_file.path()).await.unwrap();
        
        assert_eq!(manager2.list_saved_configs().len(), 1);
    }

    #[test]
    fn test_config_validation() {
        let manager = ConfigurationManager::new();
        let mut config = CompleteGraphConfig::default();
        
        // Valid config should pass
        assert!(manager.validate_config(&config).is_ok());
        
        // Invalid dimensions
        config.base.width = 0;
        assert!(manager.validate_config(&config).is_err());
        
        // Invalid data point limit
        config.base.width = 800;
        config.filters.data_point_limit = Some(0);
        assert!(manager.validate_config(&config).is_err());
        
        config.filters.data_point_limit = Some(15000);
        assert!(manager.validate_config(&config).is_err());
    }

    #[test]
    fn test_preset_creation() {
        let manager = ConfigurationManager::default();
        let presets = manager.get_presets();
        
        // Should have default presets
        assert!(presets.len() >= 3);
        
        // Check for specific presets
        let preset_names: Vec<&String> = presets.iter().map(|p| &p.metadata.name).collect();
        assert!(preset_names.contains(&&"Dark Presentation".to_string()));
        assert!(preset_names.contains(&&"Clean Report".to_string()));
        assert!(preset_names.contains(&&"Vibrant Dashboard".to_string()));
    }

    #[test]
    fn test_create_from_preset() {
        let manager = ConfigurationManager::default();
        
        // Create from existing preset
        let config = manager.create_from_preset("Dark Presentation");
        assert!(config.is_some());
        
        let config = config.unwrap();
        assert_eq!(config.base.width, 1920);
        assert_eq!(config.base.height, 1080);
        
        // Non-existent preset should return None
        let config = manager.create_from_preset("Non-existent");
        assert!(config.is_none());
    }
} 