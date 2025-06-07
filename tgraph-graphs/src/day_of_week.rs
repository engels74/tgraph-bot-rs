//! Day of week play count bar chart implementation

use crate::{DataSet, GraphConfig, GraphRenderer};
use async_trait::async_trait;
use chrono::Weekday;
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::{Result, TGraphError};

/// Data point for day of week play counts
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DayOfWeekDataPoint {
    pub weekday: Weekday,
    pub count: u32,
    pub label: Option<String>,
}

/// Day of week bar chart renderer for Tautulli data
#[derive(Debug)]
pub struct DayOfWeekGraph {
    /// Data points for each day of the week (0-6, Monday-Sunday)
    pub data: Vec<DayOfWeekDataPoint>,
    /// Whether to include weekend highlighting
    pub highlight_weekends: bool,
}

impl DayOfWeekGraph {
    /// Create a new day of week graph
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            highlight_weekends: true,
        }
    }

    /// Create a new graph with custom title and labels
    pub fn with_config(title: &str, x_label: Option<&str>, y_label: Option<&str>) -> (Self, GraphConfig) {
        let graph = Self::new();
        let mut config = GraphConfig {
            title: title.to_string(),
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
            graph_type: crate::GraphType::Bar,
            ..Default::default()
        };
        
        // Optimize dimensions for 7-day bar chart
        config.width = 800;
        config.height = 500;
        config.style.margins.bottom = 60; // More space for day labels
        
        (graph, config)
    }

    /// Create without weekend highlighting
    pub fn without_weekend_highlighting() -> Self {
        Self {
            data: Vec::new(),
            highlight_weekends: false,
        }
    }

    /// Add a data point for a specific weekday
    pub fn add_data_point(&mut self, weekday: Weekday, count: u32, label: Option<String>) {
        self.data.push(DayOfWeekDataPoint { weekday, count, label });
    }

    /// Set data from aggregated weekday counts
    pub fn set_data(&mut self, mut data: Vec<DayOfWeekDataPoint>) {
        // Sort by weekday for consistent display
        data.sort_by_key(|d| match d.weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        });
        self.data = data;
    }

    /// Convert weekday to numeric index (Monday = 0, Sunday = 6)
    fn weekday_to_index(&self, weekday: Weekday) -> usize {
        match weekday {
            Weekday::Mon => 0,
            Weekday::Tue => 1,
            Weekday::Wed => 2,
            Weekday::Thu => 3,
            Weekday::Fri => 4,
            Weekday::Sat => 5,
            Weekday::Sun => 6,
        }
    }

    /// Get weekday short name
    fn weekday_name(&self, weekday: Weekday) -> &'static str {
        match weekday {
            Weekday::Mon => "Mon",
            Weekday::Tue => "Tue", 
            Weekday::Wed => "Wed",
            Weekday::Thu => "Thu",
            Weekday::Fri => "Fri",
            Weekday::Sat => "Sat",
            Weekday::Sun => "Sun",
        }
    }

    /// Check if weekday is weekend
    fn is_weekend(&self, weekday: Weekday) -> bool {
        matches!(weekday, Weekday::Sat | Weekday::Sun)
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

    /// Convert data to plotters-compatible format with proper positioning
    fn prepare_plot_data(&self) -> Vec<(usize, u32)> {
        self.data
            .iter()
            .map(|point| (self.weekday_to_index(point.weekday), point.count))
            .collect()
    }

    /// Apply weekend highlighting theme
    pub fn apply_weekend_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#4a90e2".to_string(), // Regular days - blue
            "#ff6b6b".to_string(), // Weekend days - red/orange
            "#4ecdc4".to_string(), // Additional colors if needed
            "#45b7d1".to_string(),
            "#96ceb4".to_string(),
        ]);
    }

    /// Apply work-focused theme (highlight weekdays)
    pub fn apply_workday_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#2ecc71".to_string(), // Weekdays - green
            "#95a5a6".to_string(), // Weekends - gray
            "#3498db".to_string(), // Additional colors
            "#9b59b6".to_string(),
            "#f39c12".to_string(),
        ]);
    }
}

impl Default for DayOfWeekGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRenderer for DayOfWeekGraph {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        _datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        if self.data.is_empty() {
            return Err(TGraphError::graph("No data available for day of week chart"));
        }

        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;

        let max_count = self.get_max_count();
        let plot_data = self.prepare_plot_data();

        // Create chart with proper margins for day labels
        let title_font = (config.style.title_font.family.as_str(), config.style.title_font.size);
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, title_font)
            .margin(config.style.margins.top as i32)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(0usize..6usize, 0.0..max_count)?;

        // Configure mesh with custom x-axis labels
        chart.configure_mesh()
            .x_desc(config.x_label.as_deref().unwrap_or("Day of Week"))
            .y_desc(config.y_label.as_deref().unwrap_or("Play Count"))
            .x_label_formatter(&|x| {
                match *x {
                    0 => "Mon".to_string(),
                    1 => "Tue".to_string(),
                    2 => "Wed".to_string(),
                    3 => "Thu".to_string(),
                    4 => "Fri".to_string(),
                    5 => "Sat".to_string(),
                    6 => "Sun".to_string(),
                    _ => "".to_string(),
                }
            })
            .draw()?;

        // Get colors for bars
        let colors = self.get_colors(&config.style.color_scheme);
        let primary_color = &colors[0];
        let weekend_color = if colors.len() > 1 { &colors[1] } else { primary_color };

        // Draw bars for each day
        for (day_index, count) in plot_data {
            let weekday = match day_index {
                0 => Weekday::Mon,
                1 => Weekday::Tue,
                2 => Weekday::Wed,
                3 => Weekday::Thu,
                4 => Weekday::Fri,
                5 => Weekday::Sat,
                6 => Weekday::Sun,
                _ => continue,
            };

            // Choose color based on weekend highlighting
            let bar_color = if self.highlight_weekends && self.is_weekend(weekday) {
                weekend_color
            } else {
                primary_color
            };

            // Draw individual bar
            chart.draw_series(std::iter::once(Rectangle::new([
                (day_index, 0.0),
                (day_index, count as f64)
            ], bar_color.filled())))?;
        }

        root.present()?;
        tracing::info!("Successfully rendered day of week chart to {}", path.display());
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        Err(TGraphError::graph("render_to_bytes not implemented for DayOfWeekGraph"))
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
    use chrono::Weekday;
    use tempfile::tempdir;

    #[test]
    fn test_day_of_week_creation() {
        let graph = DayOfWeekGraph::new();
        assert!(graph.data.is_empty());
        assert!(graph.highlight_weekends);
    }

    #[test]
    fn test_without_weekend_highlighting() {
        let graph = DayOfWeekGraph::without_weekend_highlighting();
        assert!(!graph.highlight_weekends);
    }

    #[test]
    fn test_add_data_point() {
        let mut graph = DayOfWeekGraph::new();
        graph.add_data_point(Weekday::Mon, 42, Some("Monday plays".to_string()));
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].weekday, Weekday::Mon);
        assert_eq!(graph.data[0].count, 42);
        assert_eq!(graph.data[0].label, Some("Monday plays".to_string()));
    }

    #[test]
    fn test_set_data_sorts_by_weekday() {
        let mut graph = DayOfWeekGraph::new();
        let data = vec![
            DayOfWeekDataPoint { weekday: Weekday::Fri, count: 15, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Mon, count: 25, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Wed, count: 20, label: None },
        ];
        
        graph.set_data(data);
        
        // Should be sorted Monday, Wednesday, Friday
        assert_eq!(graph.data[0].weekday, Weekday::Mon);
        assert_eq!(graph.data[1].weekday, Weekday::Wed);
        assert_eq!(graph.data[2].weekday, Weekday::Fri);
    }

    #[test]
    fn test_weekday_to_index() {
        let graph = DayOfWeekGraph::new();
        assert_eq!(graph.weekday_to_index(Weekday::Mon), 0);
        assert_eq!(graph.weekday_to_index(Weekday::Tue), 1);
        assert_eq!(graph.weekday_to_index(Weekday::Wed), 2);
        assert_eq!(graph.weekday_to_index(Weekday::Thu), 3);
        assert_eq!(graph.weekday_to_index(Weekday::Fri), 4);
        assert_eq!(graph.weekday_to_index(Weekday::Sat), 5);
        assert_eq!(graph.weekday_to_index(Weekday::Sun), 6);
    }

    #[test]
    fn test_weekday_names() {
        let graph = DayOfWeekGraph::new();
        assert_eq!(graph.weekday_name(Weekday::Mon), "Mon");
        assert_eq!(graph.weekday_name(Weekday::Tue), "Tue");
        assert_eq!(graph.weekday_name(Weekday::Wed), "Wed");
        assert_eq!(graph.weekday_name(Weekday::Thu), "Thu");
        assert_eq!(graph.weekday_name(Weekday::Fri), "Fri");
        assert_eq!(graph.weekday_name(Weekday::Sat), "Sat");
        assert_eq!(graph.weekday_name(Weekday::Sun), "Sun");
    }

    #[test]
    fn test_is_weekend() {
        let graph = DayOfWeekGraph::new();
        assert!(!graph.is_weekend(Weekday::Mon));
        assert!(!graph.is_weekend(Weekday::Tue));
        assert!(!graph.is_weekend(Weekday::Wed));
        assert!(!graph.is_weekend(Weekday::Thu));
        assert!(!graph.is_weekend(Weekday::Fri));
        assert!(graph.is_weekend(Weekday::Sat));
        assert!(graph.is_weekend(Weekday::Sun));
    }

    #[test]
    fn test_get_max_count() {
        let mut graph = DayOfWeekGraph::new();
        
        // Empty data should return default
        assert_eq!(graph.get_max_count(), 10.0);
        
        // With data should return max + 10% padding
        let data = vec![
            DayOfWeekDataPoint { weekday: Weekday::Mon, count: 10, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Tue, count: 20, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Wed, count: 15, label: None },
        ];
        graph.set_data(data);
        assert_eq!(graph.get_max_count(), 22.0); // 20 * 1.1
    }

    #[test]
    fn test_prepare_plot_data() {
        let mut graph = DayOfWeekGraph::new();
        let data = vec![
            DayOfWeekDataPoint { weekday: Weekday::Mon, count: 10, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Wed, count: 15, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Fri, count: 20, label: None },
        ];
        graph.set_data(data);
        
        let plot_data = graph.prepare_plot_data();
        assert_eq!(plot_data.len(), 3);
        assert_eq!(plot_data[0], (0, 10)); // Monday
        assert_eq!(plot_data[1], (2, 15)); // Wednesday
        assert_eq!(plot_data[2], (4, 20)); // Friday
    }

    #[tokio::test]
    async fn test_render_to_file() {
        let mut graph = DayOfWeekGraph::new();
        let data = vec![
            DayOfWeekDataPoint { weekday: Weekday::Mon, count: 10, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Tue, count: 15, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Wed, count: 12, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Thu, count: 18, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Fri, count: 22, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Sat, count: 25, label: None },
            DayOfWeekDataPoint { weekday: Weekday::Sun, count: 20, label: None },
        ];
        graph.set_data(data);

        let (_, config) = DayOfWeekGraph::with_config(
            "Play Count by Day of Week",
            Some("Day of Week"),
            Some("Number of Plays")
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("day_of_week_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_render_empty_data_error() {
        let graph = DayOfWeekGraph::new();
        let config = GraphConfig::default();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_with_config() {
        let (graph, config) = DayOfWeekGraph::with_config(
            "Test Chart",
            Some("X Label"),
            Some("Y Label")
        );
        
        assert_eq!(config.title, "Test Chart");
        assert_eq!(config.x_label, Some("X Label".to_string()));
        assert_eq!(config.y_label, Some("Y Label".to_string()));
        assert!(matches!(config.graph_type, crate::GraphType::Bar));
        assert_eq!(config.width, 800);
        assert_eq!(config.height, 500);
    }

    #[test]
    fn test_apply_weekend_theme() {
        let mut config = GraphConfig::default();
        DayOfWeekGraph::apply_weekend_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#4a90e2"); // Regular days
                assert_eq!(colors[1], "#ff6b6b"); // Weekend days
            }
            _ => panic!("Expected custom color scheme"),
        }
    }

    #[test]
    fn test_apply_workday_theme() {
        let mut config = GraphConfig::default();
        DayOfWeekGraph::apply_workday_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#2ecc71"); // Weekdays
                assert_eq!(colors[1], "#95a5a6"); // Weekends
            }
            _ => panic!("Expected custom color scheme"),
        }
    }
} 