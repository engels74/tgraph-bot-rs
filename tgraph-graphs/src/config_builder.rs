//! Configuration builder utilities for easy graph configuration creation

use crate::{
    ColorScheme, CompleteGraphConfig, ConfigMetadata, ConfigurationManager, 
    DailyPlayCountConfig, DayOfWeekConfig, DisplayConfig, FilterConfig, 
    FontConfig, GraphConfig, GraphSpecificConfig, GridConfig, GridStyle,
    HourlyDistributionConfig, MarginConfig, MonthlyTrendsConfig, 
    SortOrder, StyleConfig, TopItemsConfig, DateRange
};
use chrono::{NaiveDate, Utc};
use tgraph_common::Result;

/// Builder for creating complete graph configurations
#[derive(Debug, Clone)]
pub struct GraphConfigBuilder {
    config: CompleteGraphConfig,
}

impl GraphConfigBuilder {
    /// Create a new configuration builder with defaults
    pub fn new(name: &str) -> Self {
        let mut metadata = ConfigMetadata::default();
        metadata.name = name.to_string();
        
        Self {
            config: CompleteGraphConfig {
                metadata,
                ..Default::default()
            },
        }
    }

    /// Create a builder from an existing configuration
    pub fn from_config(config: CompleteGraphConfig) -> Self {
        Self { config }
    }

    /// Create a builder from a preset
    pub fn from_preset(manager: &ConfigurationManager, preset_name: &str) -> Result<Self> {
        let config = manager.create_from_preset(preset_name)
            .ok_or_else(|| tgraph_common::TGraphError::config(
                format!("Preset '{}' not found", preset_name)
            ))?;
        Ok(Self::from_config(config))
    }

    /// Set the graph title
    pub fn title(mut self, title: &str) -> Self {
        self.config.base.title = title.to_string();
        self
    }

    /// Set graph dimensions
    pub fn dimensions(mut self, width: u32, height: u32) -> Self {
        self.config.base.width = width;
        self.config.base.height = height;
        self
    }

    /// Set axis labels
    pub fn labels(mut self, x_label: Option<&str>, y_label: Option<&str>) -> Self {
        self.config.base.x_label = x_label.map(|s| s.to_string());
        self.config.base.y_label = y_label.map(|s| s.to_string());
        self
    }

    /// Set color scheme
    pub fn color_scheme(mut self, scheme: ColorScheme) -> Self {
        self.config.base.style.color_scheme = scheme;
        self
    }

    /// Set background color
    pub fn background_color(mut self, color: &str) -> Self {
        self.config.base.style.background_color = Some(color.to_string());
        self
    }

    /// Set title font
    pub fn title_font(mut self, family: &str, size: u32) -> Self {
        self.config.base.style.title_font = FontConfig {
            family: family.to_string(),
            size,
        };
        self
    }

    /// Set margins
    pub fn margins(mut self, top: u32, right: u32, bottom: u32, left: u32) -> Self {
        self.config.base.style.margins = MarginConfig {
            top, right, bottom, left
        };
        self
    }

    /// Set grid configuration
    pub fn grid(mut self, show_x: bool, show_y: bool, style: GridStyle) -> Self {
        self.config.base.style.grid = GridConfig {
            show_x,
            show_y,
            style,
            color: self.config.base.style.grid.color.clone(),
        };
        self
    }

    /// Set grid color
    pub fn grid_color(mut self, color: &str) -> Self {
        self.config.base.style.grid.color = Some(color.to_string());
        self
    }

    /// Set date range filter
    pub fn date_range(mut self, start: NaiveDate, end: NaiveDate) -> Self {
        self.config.filters.date_range = Some(DateRange::new(start, end));
        self
    }

    /// Set date range to last N days
    pub fn last_days(mut self, days: u32) -> Self {
        self.config.filters.date_range = Some(DateRange::last_days(days));
        self
    }

    /// Set platform filter
    pub fn platforms(mut self, platforms: Vec<String>) -> Self {
        self.config.filters.platforms = Some(platforms);
        self
    }

    /// Set user filter
    pub fn users(mut self, users: Vec<String>) -> Self {
        self.config.filters.users = Some(users);
        self
    }

    /// Set data point limit
    pub fn data_limit(mut self, limit: u32) -> Self {
        self.config.filters.data_point_limit = Some(limit);
        self
    }

    /// Set minimum threshold for data inclusion
    pub fn minimum_threshold(mut self, threshold: f64) -> Self {
        self.config.filters.minimum_threshold = Some(threshold);
        self
    }

    /// Set display options
    pub fn display(mut self, show_labels: bool, show_legend: bool, show_grid: bool) -> Self {
        self.config.display.show_data_labels = show_labels;
        self.config.display.show_legend = show_legend;
        self.config.display.show_grid = show_grid;
        self
    }

    /// Set sort order
    pub fn sort_order(mut self, order: SortOrder) -> Self {
        self.config.display.sort_order = order;
        self
    }

    /// Enable animations
    pub fn animations(mut self, enabled: bool) -> Self {
        self.config.display.enable_animations = enabled;
        self
    }

    /// Set metadata description
    pub fn description(mut self, description: &str) -> Self {
        self.config.metadata.description = Some(description.to_string());
        self
    }

    /// Add tags
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.config.metadata.tags = tags;
        self
    }

    /// Set author
    pub fn author(mut self, author: &str) -> Self {
        self.config.metadata.author = Some(author.to_string());
        self
    }

    /// Apply daily play count specific configuration
    pub fn daily_play_count_config<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(DailyPlayCountConfigBuilder) -> DailyPlayCountConfigBuilder
    {
        let builder = f(DailyPlayCountConfigBuilder::default());
        let config = builder.build();
        self.config.graph_specific = Some(serde_json::to_value(config).unwrap());
        self
    }

    /// Apply day of week specific configuration
    pub fn day_of_week_config<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(DayOfWeekConfigBuilder) -> DayOfWeekConfigBuilder
    {
        let builder = f(DayOfWeekConfigBuilder::default());
        let config = builder.build();
        self.config.graph_specific = Some(serde_json::to_value(config).unwrap());
        self
    }

    /// Apply hourly distribution specific configuration
    pub fn hourly_distribution_config<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(HourlyDistributionConfigBuilder) -> HourlyDistributionConfigBuilder
    {
        let builder = f(HourlyDistributionConfigBuilder::default());
        let config = builder.build();
        self.config.graph_specific = Some(serde_json::to_value(config).unwrap());
        self
    }

    /// Apply monthly trends specific configuration
    pub fn monthly_trends_config<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(MonthlyTrendsConfigBuilder) -> MonthlyTrendsConfigBuilder
    {
        let builder = f(MonthlyTrendsConfigBuilder::default());
        let config = builder.build();
        self.config.graph_specific = Some(serde_json::to_value(config).unwrap());
        self
    }

    /// Apply top items specific configuration
    pub fn top_items_config<F>(mut self, f: F) -> Self 
    where 
        F: FnOnce(TopItemsConfigBuilder) -> TopItemsConfigBuilder
    {
        let builder = f(TopItemsConfigBuilder::default());
        let config = builder.build();
        self.config.graph_specific = Some(serde_json::to_value(config).unwrap());
        self
    }

    /// Update metadata timestamp
    fn update_timestamp(mut self) -> Self {
        self.config.metadata.modified_at = Utc::now();
        self
    }

    /// Validate and build the configuration
    pub fn build(self) -> Result<CompleteGraphConfig> {
        let manager = ConfigurationManager::new();
        manager.validate_config(&self.config)?;
        Ok(self.update_timestamp().config)
    }

    /// Build without validation (unsafe)
    pub fn build_unchecked(self) -> CompleteGraphConfig {
        self.update_timestamp().config
    }
}

/// Builder for DailyPlayCountConfig
#[derive(Debug, Clone)]
pub struct DailyPlayCountConfigBuilder {
    config: DailyPlayCountConfig,
}

impl Default for DailyPlayCountConfigBuilder {
    fn default() -> Self {
        Self {
            config: DailyPlayCountConfig::default(),
        }
    }
}

impl DailyPlayCountConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn highlight_weekends(mut self, highlight: bool) -> Self {
        self.config.highlight_weekends = highlight;
        self
    }

    pub fn moving_average(mut self, enabled: bool, days: u32) -> Self {
        self.config.show_moving_average = enabled;
        self.config.moving_average_days = days;
        self
    }

    pub fn growth_trends(mut self, enabled: bool) -> Self {
        self.config.show_growth_trends = enabled;
        self
    }

    pub fn weekend_color(mut self, color: &str) -> Self {
        self.config.weekend_color = Some(color.to_string());
        self
    }

    pub fn line_style(mut self, thickness: u32, show_markers: bool) -> Self {
        self.config.line_thickness = thickness;
        self.config.show_markers = show_markers;
        self
    }

    pub fn build(self) -> DailyPlayCountConfig {
        self.config
    }
}

/// Builder for DayOfWeekConfig
#[derive(Debug, Clone)]
pub struct DayOfWeekConfigBuilder {
    config: DayOfWeekConfig,
}

impl Default for DayOfWeekConfigBuilder {
    fn default() -> Self {
        Self {
            config: DayOfWeekConfig::default(),
        }
    }
}

impl DayOfWeekConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn start_week_monday(mut self, monday: bool) -> Self {
        self.config.start_week_monday = monday;
        self
    }

    pub fn show_percentages(mut self, percentages: bool) -> Self {
        self.config.show_percentages = percentages;
        self
    }

    pub fn highlight_weekends(mut self, highlight: bool) -> Self {
        self.config.highlight_weekends = highlight;
        self
    }

    pub fn bar_width_ratio(mut self, ratio: f64) -> Self {
        self.config.bar_width_ratio = ratio;
        self
    }

    pub fn show_average_line(mut self, show: bool) -> Self {
        self.config.show_average_line = show;
        self
    }

    pub fn build(self) -> DayOfWeekConfig {
        self.config
    }
}

/// Builder for HourlyDistributionConfig
#[derive(Debug, Clone)]
pub struct HourlyDistributionConfigBuilder {
    config: HourlyDistributionConfig,
}

impl Default for HourlyDistributionConfigBuilder {
    fn default() -> Self {
        Self {
            config: HourlyDistributionConfig::default(),
        }
    }
}

impl HourlyDistributionConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn time_format_24h(mut self, use_24h: bool) -> Self {
        self.config.use_24_hour_format = use_24h;
        self
    }

    pub fn group_time_periods(mut self, group: bool) -> Self {
        self.config.group_time_periods = group;
        self
    }

    pub fn highlight_peak_hours(mut self, highlight: bool, threshold: f64) -> Self {
        self.config.highlight_peak_hours = highlight;
        self.config.peak_threshold_percent = threshold;
        self
    }

    pub fn smooth_curve(mut self, smooth: bool) -> Self {
        self.config.smooth_curve = smooth;
        self
    }

    pub fn build(self) -> HourlyDistributionConfig {
        self.config
    }
}

/// Builder for MonthlyTrendsConfig
#[derive(Debug, Clone)]
pub struct MonthlyTrendsConfigBuilder {
    config: MonthlyTrendsConfig,
}

impl Default for MonthlyTrendsConfigBuilder {
    fn default() -> Self {
        Self {
            config: MonthlyTrendsConfig::default(),
        }
    }
}

impl MonthlyTrendsConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn year_over_year_comparison(mut self, enabled: bool) -> Self {
        self.config.show_yoy_comparison = enabled;
        self
    }

    pub fn seasonal_trends(mut self, enabled: bool) -> Self {
        self.config.show_seasonal_trends = enabled;
        self
    }

    pub fn quarterly_aggregation(mut self, enabled: bool) -> Self {
        self.config.quarterly_aggregation = enabled;
        self
    }

    pub fn forecast(mut self, enabled: bool, months: u32) -> Self {
        self.config.show_forecast = enabled;
        self.config.forecast_months = months;
        self
    }

    pub fn growth_labels(mut self, enabled: bool) -> Self {
        self.config.show_growth_labels = enabled;
        self
    }

    pub fn build(self) -> MonthlyTrendsConfig {
        self.config
    }
}

/// Builder for TopItemsConfig
#[derive(Debug, Clone)]
pub struct TopItemsConfigBuilder {
    config: TopItemsConfig,
}

impl Default for TopItemsConfigBuilder {
    fn default() -> Self {
        Self {
            config: TopItemsConfig::default(),
        }
    }
}

impl TopItemsConfigBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max_items(mut self, max: u32) -> Self {
        self.config.max_items = max;
        self
    }

    pub fn show_percentages(mut self, percentages: bool) -> Self {
        self.config.show_percentages = percentages;
        self
    }

    pub fn show_others_category(mut self, show_others: bool) -> Self {
        self.config.show_others_category = show_others;
        self
    }

    pub fn horizontal_bars(mut self, horizontal: bool) -> Self {
        self.config.horizontal_bars = horizontal;
        self
    }

    pub fn sort_by_count(mut self, sort_by_count: bool) -> Self {
        self.config.sort_by_count = sort_by_count;
        self
    }

    pub fn show_data_labels(mut self, show_labels: bool) -> Self {
        self.config.show_data_labels = show_labels;
        self
    }

    pub fn minimum_count(mut self, min: u32) -> Self {
        self.config.minimum_count = min;
        self
    }

    pub fn build(self) -> TopItemsConfig {
        self.config
    }
}

/// Preset configuration templates for common use cases
pub struct ConfigPresets;

impl ConfigPresets {
    /// Create a presentation-ready configuration
    pub fn presentation(title: &str) -> GraphConfigBuilder {
        GraphConfigBuilder::new("Presentation")
            .title(title)
            .dimensions(1920, 1080)
            .color_scheme(ColorScheme::Dark)
            .background_color("#1a1a1a")
            .title_font("Arial", 24)
            .margins(40, 40, 60, 80)
            .grid(false, true, GridStyle::Solid)
            .display(true, true, false)
            .tags(vec!["presentation".to_string(), "dark".to_string()])
    }

    /// Create a report-ready configuration
    pub fn report(title: &str) -> GraphConfigBuilder {
        GraphConfigBuilder::new("Report")
            .title(title)
            .dimensions(800, 600)
            .color_scheme(ColorScheme::Light)
            .background_color("#ffffff")
            .title_font("Times New Roman", 18)
            .margins(20, 20, 40, 60)
            .grid(true, true, GridStyle::Solid)
            .display(false, true, true)
            .tags(vec!["report".to_string(), "professional".to_string()])
    }

    /// Create a dashboard-ready configuration
    pub fn dashboard(title: &str) -> GraphConfigBuilder {
        GraphConfigBuilder::new("Dashboard")
            .title(title)
            .dimensions(600, 400)
            .color_scheme(ColorScheme::Vibrant)
            .background_color("#f8f9fa")
            .title_font("Helvetica", 16)
            .margins(15, 15, 30, 45)
            .grid(true, true, GridStyle::Dotted)
            .display(true, false, true)
            .data_limit(50)
            .animations(true)
            .tags(vec!["dashboard".to_string(), "compact".to_string()])
    }

    /// Create a social media-ready configuration
    pub fn social_media(title: &str) -> GraphConfigBuilder {
        GraphConfigBuilder::new("Social Media")
            .title(title)
            .dimensions(1080, 1080) // Square format
            .color_scheme(ColorScheme::Vibrant)
            .background_color("#ffffff")
            .title_font("Arial", 20)
            .margins(30, 30, 50, 60)
            .grid(false, false, GridStyle::Solid)
            .display(true, false, false)
            .tags(vec!["social".to_string(), "square".to_string()])
    }

    /// Create a print-ready configuration
    pub fn print(title: &str) -> GraphConfigBuilder {
        GraphConfigBuilder::new("Print")
            .title(title)
            .dimensions(2400, 1800) // High resolution
            .color_scheme(ColorScheme::Monochrome)
            .background_color("#ffffff")
            .title_font("Times New Roman", 24)
            .margins(60, 60, 80, 100)
            .grid(true, true, GridStyle::Solid)
            .display(false, true, true)
            .tags(vec!["print".to_string(), "high-res".to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graph_config_builder_basic() {
        let config = GraphConfigBuilder::new("Test Config")
            .title("Test Graph")
            .dimensions(800, 600)
            .build()
            .unwrap();

        assert_eq!(config.metadata.name, "Test Config");
        assert_eq!(config.base.title, "Test Graph");
        assert_eq!(config.base.width, 800);
        assert_eq!(config.base.height, 600);
    }

    #[test]
    fn test_daily_play_count_config_builder() {
        let config = GraphConfigBuilder::new("Daily Play Count")
            .daily_play_count_config(|builder| {
                builder
                    .highlight_weekends(true)
                    .moving_average(true, 7)
                    .line_style(3, true)
            })
            .build()
            .unwrap();

        assert!(config.graph_specific.is_some());
        
        let daily_config: DailyPlayCountConfig = 
            serde_json::from_value(config.graph_specific.unwrap()).unwrap();
        assert!(daily_config.highlight_weekends);
        assert!(daily_config.show_moving_average);
        assert_eq!(daily_config.moving_average_days, 7);
        assert_eq!(daily_config.line_thickness, 3);
    }

    #[test]
    fn test_preset_configurations() {
        let presentation = ConfigPresets::presentation("Test")
            .build()
            .unwrap();
        assert_eq!(presentation.base.width, 1920);
        assert_eq!(presentation.base.height, 1080);

        let report = ConfigPresets::report("Test")
            .build()
            .unwrap();
        assert_eq!(report.base.width, 800);
        assert_eq!(report.base.height, 600);

        let dashboard = ConfigPresets::dashboard("Test")
            .build()
            .unwrap();
        assert_eq!(dashboard.base.width, 600);
        assert_eq!(dashboard.base.height, 400);
    }

    #[test]
    fn test_filter_configuration() {
        let config = GraphConfigBuilder::new("Filtered")
            .last_days(30)
            .data_limit(100)
            .minimum_threshold(5.0)
            .platforms(vec!["Platform 1".to_string(), "Platform 2".to_string()])
            .build()
            .unwrap();

        assert!(config.filters.date_range.is_some());
        assert_eq!(config.filters.data_point_limit, Some(100));
        assert_eq!(config.filters.minimum_threshold, Some(5.0));
        assert_eq!(config.filters.platforms.as_ref().unwrap().len(), 2);
    }

    #[test]
    fn test_styling_configuration() {
        let config = GraphConfigBuilder::new("Styled")
            .color_scheme(ColorScheme::Dark)
            .background_color("#1a1a1a")
            .title_font("Arial", 20)
            .margins(10, 15, 20, 25)
            .grid_color("#333333")
            .build()
            .unwrap();

        assert!(matches!(config.base.style.color_scheme, ColorScheme::Dark));
        assert_eq!(config.base.style.background_color, Some("#1a1a1a".to_string()));
        assert_eq!(config.base.style.title_font.family, "Arial");
        assert_eq!(config.base.style.title_font.size, 20);
        assert_eq!(config.base.style.margins.top, 10);
        assert_eq!(config.base.style.grid.color, Some("#333333".to_string()));
    }

    #[test]
    fn test_validation_errors() {
        let invalid_config = GraphConfigBuilder::new("Invalid")
            .dimensions(0, 0) // Invalid dimensions
            .build();

        assert!(invalid_config.is_err());
    }

    #[test]
    fn test_builder_chaining() {
        let config = GraphConfigBuilder::new("Chained")
            .title("Chained Title")
            .dimensions(1000, 800)
            .color_scheme(ColorScheme::Vibrant)
            .last_days(7)
            .data_limit(50)
            .display(true, false, true)
            .description("Test description")
            .tags(vec!["test".to_string(), "chained".to_string()])
            .build()
            .unwrap();

        assert_eq!(config.base.title, "Chained Title");
        assert_eq!(config.base.width, 1000);
        assert!(matches!(config.base.style.color_scheme, ColorScheme::Vibrant));
        assert!(config.filters.date_range.is_some());
        assert_eq!(config.filters.data_point_limit, Some(50));
        assert!(config.display.show_data_labels);
        assert!(!config.display.show_legend);
        assert!(config.display.show_grid);
        assert_eq!(config.metadata.description, Some("Test description".to_string()));
        assert_eq!(config.metadata.tags.len(), 2);
    }
} 