//! Daily play count time series graph implementation

use crate::{DataSet, GraphConfig, GraphRenderer};
use async_trait::async_trait;
use chrono::{Datelike, NaiveDate};
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::Result;

/// Time series data point for daily play counts
#[derive(Debug, Clone)]
pub struct PlayCountDataPoint {
    pub date: NaiveDate,
    pub count: u32,
    pub label: Option<String>,
}

/// Daily play count graph renderer for Tautulli data
#[derive(Debug)]
pub struct DailyPlayCountGraph {
    /// Data points for the time series
    pub data: Vec<PlayCountDataPoint>,
    /// Start date for the graph
    pub start_date: Option<NaiveDate>,
    /// End date for the graph
    pub end_date: Option<NaiveDate>,
}

impl DailyPlayCountGraph {
    /// Create a new daily play count graph
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            start_date: None,
            end_date: None,
        }
    }

    /// Create a new graph with custom title and labels
    pub fn with_config(title: &str, x_label: Option<&str>, y_label: Option<&str>) -> (Self, GraphConfig) {
        let graph = Self::new();
        let config = GraphConfig {
            title: title.to_string(),
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
            ..Default::default()
        };
        (graph, config)
    }

    /// Create with specific date range
    pub fn with_date_range(start: NaiveDate, end: NaiveDate) -> Self {
        Self {
            data: Vec::new(),
            start_date: Some(start),
            end_date: Some(end),
        }
    }

    /// Add a data point
    pub fn add_data_point(&mut self, date: NaiveDate, count: u32, label: Option<String>) {
        self.data.push(PlayCountDataPoint { date, count, label });
    }

    /// Set data from Tautulli history entries
    pub fn set_data(&mut self, data: Vec<PlayCountDataPoint>) {
        self.data = data;
        // Update date range based on data if not explicitly set
        if self.start_date.is_none() || self.end_date.is_none() {
            if let (Some(min_date), Some(max_date)) = (
                self.data.iter().map(|d| d.date).min(),
                self.data.iter().map(|d| d.date).max(),
            ) {
                if self.start_date.is_none() {
                    self.start_date = Some(min_date);
                }
                if self.end_date.is_none() {
                    self.end_date = Some(max_date);
                }
            }
        }
    }

    /// Convert data to plotters-compatible format
    fn prepare_plot_data(&self) -> Vec<(f64, f64)> {
        self.data
            .iter()
            .enumerate()
            .map(|(i, point)| (i as f64, point.count as f64))
            .collect()
    }

    /// Get max count for y-axis scaling
    fn get_max_count(&self) -> f64 {
        if self.data.is_empty() {
            return 10.0; // Default value for empty data
        }
        self.data
            .iter()
            .map(|d| d.count as f64)
            .fold(0.0, f64::max)
            * 1.1 // Add 10% padding
    }

    /// Highlight weekend points
    fn get_weekend_highlights(&self) -> Vec<(f64, f64)> {
        self.data
            .iter()
            .enumerate()
            .filter(|(_, point)| {
                let weekday = point.date.weekday();
                weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun
            })
            .map(|(i, point)| (i as f64, point.count as f64))
            .collect()
    }

    /// Apply custom color scheme to config
    pub fn apply_color_scheme(config: &mut GraphConfig, scheme: crate::ColorScheme) {
        config.style.color_scheme = scheme;
    }

    /// Apply dark theme to config
    pub fn apply_dark_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Dark;
        config.style.background_color = Some("#2b2b2b".to_string());
        config.style.grid.color = Some("#404040".to_string());
    }

    /// Apply light theme to config
    pub fn apply_light_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Light;
        config.style.background_color = Some("#ffffff".to_string());
        config.style.grid.color = Some("#e0e0e0".to_string());
    }

    /// Apply vibrant theme to config
    pub fn apply_vibrant_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Vibrant;
        config.style.background_color = Some("#f8f9fa".to_string());
        config.style.grid.color = Some("#dee2e6".to_string());
    }

    /// Customize margins
    pub fn set_margins(config: &mut GraphConfig, top: u32, right: u32, bottom: u32, left: u32) {
        config.style.margins.top = top;
        config.style.margins.right = right;
        config.style.margins.bottom = bottom;
        config.style.margins.left = left;
    }

    /// Customize font sizes
    pub fn set_font_sizes(config: &mut GraphConfig, title: u32, axis: u32, label: u32) {
        config.style.title_font.size = title;
        config.style.axis_font.size = axis;
        config.style.label_font.size = label;
    }

    /// Toggle grid lines
    pub fn set_grid_visibility(config: &mut GraphConfig, show_x: bool, show_y: bool) {
        config.style.grid.show_x = show_x;
        config.style.grid.show_y = show_y;
    }

    /// Set custom dimensions
    pub fn set_dimensions(config: &mut GraphConfig, width: u32, height: u32) {
        config.width = width;
        config.height = height;
    }

    /// Apply custom background color
    pub fn set_background_color(config: &mut GraphConfig, color: &str) {
        config.style.background_color = Some(color.to_string());
    }

    /// Add a custom primary color for the main data line
    pub fn set_primary_color(config: &mut GraphConfig, color: &str) {
        // Store the custom color in the color scheme
        if let crate::ColorScheme::Custom(ref mut colors) = config.style.color_scheme {
            colors[0] = color.to_string(); // Update first color
        } else {
            // Convert to custom scheme with the specified primary color
            config.style.color_scheme = crate::ColorScheme::Custom(vec![
                color.to_string(),
                "#ff6b6b".to_string(), // Secondary colors
                "#4ecdc4".to_string(),
                "#45b7d1".to_string(),
                "#96ceb4".to_string(),
            ]);
        }
    }

    /// Set a custom title with specific font size
    pub fn customize_title(config: &mut GraphConfig, title: &str, font_size: u32) {
        config.title = title.to_string();
        config.style.title_font.size = font_size;
    }

    /// Get a preconfigured theme for presentations
    pub fn presentation_theme() -> GraphConfig {
        let mut config = GraphConfig {
            width: 1920,
            height: 1080,
            title: "Tautulli Play Statistics".to_string(),
            ..Default::default()
        };
        
        // Use vibrant colors with large fonts
        Self::apply_vibrant_theme(&mut config);
        Self::set_font_sizes(&mut config, 36, 24, 20);
        Self::set_margins(&mut config, 80, 60, 100, 120);
        
        config
    }

    /// Get a preconfigured theme for reports
    pub fn report_theme() -> GraphConfig {
        let mut config = GraphConfig {
            width: 800,
            height: 600,
            title: "Daily Play Count Report".to_string(),
            ..Default::default()
        };
        
        // Use professional light theme
        Self::apply_light_theme(&mut config);
        Self::set_font_sizes(&mut config, 16, 12, 10);
        Self::set_margins(&mut config, 40, 30, 50, 60);
        
        config
    }
}

impl Default for DailyPlayCountGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRenderer for DailyPlayCountGraph {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        _datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        if self.data.is_empty() {
            return Err(tgraph_common::TGraphError::graph("No data to render"));
        }

        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        self.apply_styling(&root, config)?;

        let plot_data = self.prepare_plot_data();
        let max_count = self.get_max_count();
        let max_x = (self.data.len() - 1) as f64;

        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, ("sans-serif", config.style.title_font.size))
            .margin(config.style.margins.top)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(0f64..max_x, 0f64..max_count)?;

        // Configure and draw mesh
        let mut mesh = chart.configure_mesh();
        
        if let Some(x_label) = &config.x_label {
            mesh.x_desc(x_label);
        }
        if let Some(y_label) = &config.y_label {
            mesh.y_desc(y_label);
        }

        // Apply grid color if specified
        if let Some(grid_color) = &config.style.grid.color {
            let color = self.parse_color(grid_color);
            mesh.light_line_style(color);
        }

        // Apply grid configuration
        if config.style.grid.show_x && config.style.grid.show_y {
            mesh.draw()?;
        } else if config.style.grid.show_x {
            mesh.disable_y_mesh().draw()?;
        } else if config.style.grid.show_y {
            mesh.disable_x_mesh().draw()?;
        } else {
            mesh.disable_mesh().draw()?;
        }

        // Get colors for the graph
        let colors = self.get_colors(&config.style.color_scheme);
        let primary_color = colors.first().copied().unwrap_or(RGBColor(31, 119, 180));

        // Draw the main line series
        chart
            .draw_series(LineSeries::new(plot_data.iter().copied(), &primary_color))?
            .label("Daily Play Count")
            .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], primary_color));

        // Draw weekend highlights if we have them
        let weekend_points = self.get_weekend_highlights();
        if !weekend_points.is_empty() {
            chart.draw_series(
                weekend_points
                    .iter()
                    .map(|point| Circle::new(*point, 3, RGBColor(255, 165, 0).filled())),
            )?
            .label("Weekends")
            .legend(|(x, y)| Circle::new((x + 5, y), 3, RGBColor(255, 165, 0).filled()));
        }

        // Draw legend
        chart.configure_series_labels().draw()?;

        root.present()?;
        tracing::info!("Successfully rendered daily play count graph to {:?}", path);
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        // For now, return a placeholder implementation
        // This would need a proper implementation using BitMapBackend::with_buffer
        // or another approach for in-memory rendering
        Err(tgraph_common::TGraphError::graph(
            "Byte rendering not yet implemented - use render_to_file instead"
        ))
    }

    fn apply_styling<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        config: &GraphConfig,
    ) -> Result<()>
    where
        DB::ErrorType: std::error::Error + Send + Sync + 'static,
    {
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    #[test]
    fn test_daily_play_count_creation() {
        let graph = DailyPlayCountGraph::new();
        assert!(graph.data.is_empty());
        assert!(graph.start_date.is_none());
        assert!(graph.end_date.is_none());
    }

    #[test]
    fn test_with_date_range() {
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        let graph = DailyPlayCountGraph::with_date_range(start, end);
        
        assert_eq!(graph.start_date, Some(start));
        assert_eq!(graph.end_date, Some(end));
    }

    #[test]
    fn test_add_data_point() {
        let mut graph = DailyPlayCountGraph::new();
        let date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        
        graph.add_data_point(date, 42, Some("Test label".to_string()));
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].date, date);
        assert_eq!(graph.data[0].count, 42);
        assert_eq!(graph.data[0].label, Some("Test label".to_string()));
    }

    #[test]
    fn test_set_data_updates_date_range() {
        let mut graph = DailyPlayCountGraph::new();
        let data = vec![
            PlayCountDataPoint {
                date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                count: 10,
                label: None,
            },
            PlayCountDataPoint {
                date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
                count: 20,
                label: None,
            },
            PlayCountDataPoint {
                date: NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
                count: 30,
                label: None,
            },
        ];
        
        graph.set_data(data);
        
        assert_eq!(graph.data.len(), 3);
        assert_eq!(graph.start_date, Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()));
        assert_eq!(graph.end_date, Some(NaiveDate::from_ymd_opt(2024, 1, 31).unwrap()));
    }

    #[test]
    fn test_prepare_plot_data() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 20, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 15, None);
        
        let plot_data = graph.prepare_plot_data();
        
        assert_eq!(plot_data.len(), 3);
        assert_eq!(plot_data[0], (0.0, 10.0));
        assert_eq!(plot_data[1], (1.0, 20.0));
        assert_eq!(plot_data[2], (2.0, 15.0));
    }

    #[test]
    fn test_get_max_count() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 25, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 15, None);
        
        let max_count = graph.get_max_count();
        
        // Should be 25 * 1.1 = 27.5, allowing for floating point precision
        assert!((max_count - 27.5).abs() < 1e-10);
    }

    #[test]
    fn test_weekend_highlights() {
        let mut graph = DailyPlayCountGraph::new();
        // January 6, 2024 is a Saturday
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 6).unwrap(), 15, None);
        // January 7, 2024 is a Sunday
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 7).unwrap(), 20, None);
        // January 8, 2024 is a Monday
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 8).unwrap(), 10, None);
        
        let weekend_points = graph.get_weekend_highlights();
        
        assert_eq!(weekend_points.len(), 2); // Saturday and Sunday
        assert_eq!(weekend_points[0], (0.0, 15.0)); // Saturday
        assert_eq!(weekend_points[1], (1.0, 20.0)); // Sunday
    }

    #[tokio::test]
    async fn test_render_to_file() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 20, None);
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_path = temp_dir.path().join("test_daily_graph.png");
        
        let mut config = GraphConfig::default();
        config.title = "Test Daily Play Count".to_string();
        config.x_label = Some("Date".to_string());
        config.y_label = Some("Play Count".to_string());
        
        let result = graph.render_to_file(&config, &[], &test_path).await;
        assert!(result.is_ok(), "Failed to render graph: {:?}", result.err());
        
        // Verify file was created
        assert!(test_path.exists(), "Graph file was not created");
        
        // Verify file has reasonable size
        let metadata = std::fs::metadata(&test_path).expect("Failed to read file metadata");
        assert!(metadata.len() > 1000, "Generated graph file is too small");
    }

    #[tokio::test]
    async fn test_render_to_bytes_not_implemented() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        
        let config = GraphConfig::default();
        let result = graph.render_to_bytes(&config, &[]).await;
        
        assert!(result.is_err(), "Should return error for unimplemented feature");
    }

    #[tokio::test]
    async fn test_render_empty_data_error() {
        let graph = DailyPlayCountGraph::new();
        let config = GraphConfig::default();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_path = temp_dir.path().join("empty_graph.png");
        
        let result = graph.render_to_file(&config, &[], &test_path).await;
        assert!(result.is_err(), "Should fail with empty data");
    }

    #[test]
    fn test_with_config() {
        let (graph, config) = DailyPlayCountGraph::with_config(
            "Custom Title",
            Some("X Axis"),
            Some("Y Axis")
        );
        
        assert!(graph.data.is_empty());
        assert_eq!(config.title, "Custom Title");
        assert_eq!(config.x_label, Some("X Axis".to_string()));
        assert_eq!(config.y_label, Some("Y Axis".to_string()));
    }

    #[test]
    fn test_apply_color_scheme() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::apply_color_scheme(&mut config, crate::ColorScheme::Vibrant);
        
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Vibrant));
    }

    #[test]
    fn test_apply_dark_theme() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::apply_dark_theme(&mut config);
        
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Dark));
        assert_eq!(config.style.background_color, Some("#2b2b2b".to_string()));
        assert_eq!(config.style.grid.color, Some("#404040".to_string()));
    }

    #[test]
    fn test_apply_light_theme() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::apply_light_theme(&mut config);
        
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Light));
        assert_eq!(config.style.background_color, Some("#ffffff".to_string()));
        assert_eq!(config.style.grid.color, Some("#e0e0e0".to_string()));
    }

    #[test]
    fn test_apply_vibrant_theme() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::apply_vibrant_theme(&mut config);
        
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Vibrant));
        assert_eq!(config.style.background_color, Some("#f8f9fa".to_string()));
        assert_eq!(config.style.grid.color, Some("#dee2e6".to_string()));
    }

    #[test]
    fn test_set_margins() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_margins(&mut config, 30, 25, 50, 70);
        
        assert_eq!(config.style.margins.top, 30);
        assert_eq!(config.style.margins.right, 25);
        assert_eq!(config.style.margins.bottom, 50);
        assert_eq!(config.style.margins.left, 70);
    }

    #[test]
    fn test_set_font_sizes() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_font_sizes(&mut config, 20, 14, 12);
        
        assert_eq!(config.style.title_font.size, 20);
        assert_eq!(config.style.axis_font.size, 14);
        assert_eq!(config.style.label_font.size, 12);
    }

    #[test]
    fn test_set_grid_visibility() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_grid_visibility(&mut config, false, true);
        
        assert!(!config.style.grid.show_x);
        assert!(config.style.grid.show_y);
    }

    #[test]
    fn test_set_dimensions() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_dimensions(&mut config, 1200, 800);
        
        assert_eq!(config.width, 1200);
        assert_eq!(config.height, 800);
    }

    #[tokio::test]
    async fn test_customized_graph_rendering() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 20, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 3).unwrap(), 15, None);
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_path = temp_dir.path().join("customized_graph.png");
        
        // Create a heavily customized config
        let mut config = GraphConfig::default();
        config.title = "Customized Play Count Graph".to_string();
        config.x_label = Some("Days".to_string());
        config.y_label = Some("Plays".to_string());
        
        // Apply customizations
        DailyPlayCountGraph::apply_vibrant_theme(&mut config);
        DailyPlayCountGraph::set_dimensions(&mut config, 1000, 600);
        DailyPlayCountGraph::set_font_sizes(&mut config, 18, 12, 10);
        DailyPlayCountGraph::set_margins(&mut config, 25, 20, 45, 65);
        DailyPlayCountGraph::set_grid_visibility(&mut config, true, false);
        
        let result = graph.render_to_file(&config, &[], &test_path).await;
        assert!(result.is_ok(), "Failed to render customized graph: {:?}", result.err());
        
        // Verify file was created with correct dimensions
        assert!(test_path.exists(), "Customized graph file was not created");
        let metadata = std::fs::metadata(&test_path).expect("Failed to read file metadata");
        assert!(metadata.len() > 1000, "Generated customized graph file is too small");
    }

    #[test]
    fn test_set_background_color() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_background_color(&mut config, "#f0f0f0");
        
        assert_eq!(config.style.background_color, Some("#f0f0f0".to_string()));
    }

    #[test]
    fn test_set_primary_color() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::set_primary_color(&mut config, "#ff0000");
        
        // Should convert to custom color scheme
        if let crate::ColorScheme::Custom(colors) = &config.style.color_scheme {
            assert_eq!(colors[0], "#ff0000");
        } else {
            panic!("Expected custom color scheme");
        }
    }

    #[test]
    fn test_customize_title() {
        let mut config = GraphConfig::default();
        DailyPlayCountGraph::customize_title(&mut config, "My Custom Title", 24);
        
        assert_eq!(config.title, "My Custom Title");
        assert_eq!(config.style.title_font.size, 24);
    }

    #[test]
    fn test_presentation_theme() {
        let config = DailyPlayCountGraph::presentation_theme();
        
        assert_eq!(config.width, 1920);
        assert_eq!(config.height, 1080);
        assert_eq!(config.title, "Tautulli Play Statistics");
        assert_eq!(config.style.title_font.size, 36);
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Vibrant));
    }

    #[test]
    fn test_report_theme() {
        let config = DailyPlayCountGraph::report_theme();
        
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 600);
        assert_eq!(config.title, "Daily Play Count Report");
        assert_eq!(config.style.title_font.size, 16);
        assert!(matches!(config.style.color_scheme, crate::ColorScheme::Light));
    }

    #[tokio::test]
    async fn test_theme_based_rendering() {
        let mut graph = DailyPlayCountGraph::new();
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(), 10, None);
        graph.add_data_point(NaiveDate::from_ymd_opt(2024, 1, 2).unwrap(), 15, None);
        
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        
        // Test presentation theme
        let presentation_path = temp_dir.path().join("presentation_theme.png");
        let presentation_config = DailyPlayCountGraph::presentation_theme();
        
        let result = graph.render_to_file(&presentation_config, &[], &presentation_path).await;
        assert!(result.is_ok(), "Failed to render presentation theme: {:?}", result.err());
        assert!(presentation_path.exists());
        
        // Test report theme
        let report_path = temp_dir.path().join("report_theme.png");
        let report_config = DailyPlayCountGraph::report_theme();
        
        let result = graph.render_to_file(&report_config, &[], &report_path).await;
        assert!(result.is_ok(), "Failed to render report theme: {:?}", result.err());
        assert!(report_path.exists());
    }
} 