//! Hourly distribution histogram implementation

use crate::{DataSet, GraphConfig, GraphRenderer};
use async_trait::async_trait;
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::{Result, TGraphError};

/// Data point for hourly play counts
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HourlyDataPoint {
    pub hour: u8, // 0-23 hour of day
    pub count: u32,
    pub label: Option<String>,
}

/// Hourly distribution histogram renderer for Tautulli data
#[derive(Debug)]
pub struct HourlyDistributionGraph {
    /// Data points for each hour of the day (0-23)
    pub data: Vec<HourlyDataPoint>,
    /// Whether to highlight peak hours
    pub highlight_peaks: bool,
    /// Peak threshold percentage (e.g., 0.8 = top 20% of hours)
    pub peak_threshold: f64,
}

impl HourlyDistributionGraph {
    /// Create a new hourly distribution graph
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            highlight_peaks: true,
            peak_threshold: 0.8, // Top 20% by default
        }
    }

    /// Create a new graph with custom title and labels
    pub fn with_config(title: &str, x_label: Option<&str>, y_label: Option<&str>) -> (Self, GraphConfig) {
        let graph = Self::new();
        let mut config = GraphConfig {
            title: title.to_string(),
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
            graph_type: crate::GraphType::Histogram,
            ..Default::default()
        };
        
        // Optimize dimensions for 24-hour histogram
        config.width = 1000;
        config.height = 500;
        config.style.margins.bottom = 60; // More space for hour labels
        config.style.margins.left = 80; // More space for count labels
        
        (graph, config)
    }

    /// Create without peak highlighting
    pub fn without_peak_highlighting() -> Self {
        Self {
            data: Vec::new(),
            highlight_peaks: false,
            peak_threshold: 0.8,
        }
    }

    /// Create with custom peak threshold
    pub fn with_peak_threshold(threshold: f64) -> Self {
        Self {
            data: Vec::new(),
            highlight_peaks: true,
            peak_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Add a data point for a specific hour
    pub fn add_data_point(&mut self, hour: u8, count: u32, label: Option<String>) {
        if hour < 24 {
            self.data.push(HourlyDataPoint { hour, count, label });
        }
    }

    /// Set data from aggregated hourly counts
    pub fn set_data(&mut self, data: Vec<HourlyDataPoint>) {
        // Filter valid hours and sort
        self.data = data.into_iter()
            .filter(|d| d.hour < 24)
            .collect();
        self.data.sort_by_key(|d| d.hour);
    }

    /// Get hour in 12-hour format with AM/PM
    #[allow(dead_code)]
    fn format_hour_12(&self, hour: u8) -> String {
        match hour {
            0 => "12 AM".to_string(),
            1..=11 => format!("{} AM", hour),
            12 => "12 PM".to_string(),
            13..=23 => format!("{} PM", hour - 12),
            _ => "??".to_string(),
        }
    }

    /// Get hour in 24-hour format
    fn format_hour_24(&self, hour: u8) -> String {
        format!("{:02}:00", hour)
    }

    /// Check if hour is in peak time (based on threshold)
    fn is_peak_hour(&self, hour: u8) -> bool {
        if !self.highlight_peaks || self.data.is_empty() {
            return false;
        }

        let hour_count = self.data.iter()
            .find(|d| d.hour == hour)
            .map(|d| d.count)
            .unwrap_or(0);

        let max_count = self.data.iter().map(|d| d.count).max().unwrap_or(0);
        let threshold_count = (max_count as f64 * self.peak_threshold) as u32;
        
        hour_count >= threshold_count
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

    /// Convert data to plotters-compatible format
    fn prepare_plot_data(&self) -> Vec<(i32, u32)> {
        // Ensure we have data for all 24 hours
        let mut plot_data = Vec::new();
        for hour in 0..24 {
            let count = self.data.iter()
                .find(|d| d.hour == hour)
                .map(|d| d.count)
                .unwrap_or(0);
            plot_data.push((hour as i32, count));
        }
        plot_data
    }

    /// Get peak hours based on threshold
    pub fn get_peak_hours(&self) -> Vec<u8> {
        if !self.highlight_peaks || self.data.is_empty() {
            return Vec::new();
        }

        let max_count = self.data.iter().map(|d| d.count).max().unwrap_or(0);
        let threshold_count = (max_count as f64 * self.peak_threshold) as u32;
        
        self.data.iter()
            .filter(|d| d.count >= threshold_count)
            .map(|d| d.hour)
            .collect()
    }

    /// Apply business hours theme (9 AM - 5 PM highlighting)
    pub fn apply_business_hours_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#34495e".to_string(), // Regular hours - dark gray
            "#3498db".to_string(), // Business hours - blue
            "#e74c3c".to_string(), // Additional colors
            "#2ecc71".to_string(),
            "#f39c12".to_string(),
        ]);
    }

    /// Apply day/night theme
    pub fn apply_day_night_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#2c3e50".to_string(), // Night hours - dark blue
            "#f1c40f".to_string(), // Day hours - yellow
            "#e67e22".to_string(), // Evening hours - orange
            "#8e44ad".to_string(), // Dawn hours - purple
            "#27ae60".to_string(),
        ]);
    }

    /// Apply peak hours theme
    pub fn apply_peak_hours_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#95a5a6".to_string(), // Regular hours - gray
            "#e74c3c".to_string(), // Peak hours - red
            "#f39c12".to_string(), // High activity - orange
            "#3498db".to_string(),
            "#2ecc71".to_string(),
        ]);
    }
}

impl Default for HourlyDistributionGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRenderer for HourlyDistributionGraph {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        _datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;

        let max_count = self.get_max_count();
        let plot_data = self.prepare_plot_data();

        // Create chart with proper margins
        let title_font = (config.style.title_font.family.as_str(), config.style.title_font.size);
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, title_font)
            .margin(config.style.margins.top as i32)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(0i32..23i32, 0.0..max_count)?;

        // Configure mesh with custom x-axis labels for hours
        chart.configure_mesh()
            .x_desc(config.x_label.as_deref().unwrap_or("Hour of Day"))
            .y_desc(config.y_label.as_deref().unwrap_or("Play Count"))
            .x_label_formatter(&|x| {
                // Show every 4th hour to avoid crowding
                if *x % 4 == 0 {
                    self.format_hour_24(*x as u8)
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // Get colors for bars
        let colors = self.get_colors(&config.style.color_scheme);
        let primary_color = &colors[0];
        let peak_color = if colors.len() > 1 { &colors[1] } else { primary_color };

        // Draw histogram bars
        for (hour, count) in plot_data {
            // Choose color based on peak highlighting
            let bar_color = if self.is_peak_hour(hour as u8) {
                peak_color
            } else {
                primary_color
            };

            // Calculate bar width (slightly less than 1 to create gaps)
            let bar_width = 0.8;
            let x_start = hour as f64 - bar_width / 2.0;
            let x_end = hour as f64 + bar_width / 2.0;

            // Draw individual bar
            chart.draw_series(std::iter::once(Rectangle::new([
                (x_start as i32, 0.0),
                (x_end as i32, count as f64)
            ], bar_color.filled())))?;
        }

        root.present()?;
        tracing::info!("Successfully rendered hourly distribution chart to {}", path.display());
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        Err(TGraphError::graph("render_to_bytes not implemented for HourlyDistributionGraph"))
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
    use tempfile::tempdir;

    #[test]
    fn test_hourly_distribution_creation() {
        let graph = HourlyDistributionGraph::new();
        assert!(graph.data.is_empty());
        assert!(graph.highlight_peaks);
        assert_eq!(graph.peak_threshold, 0.8);
    }

    #[test]
    fn test_without_peak_highlighting() {
        let graph = HourlyDistributionGraph::without_peak_highlighting();
        assert!(!graph.highlight_peaks);
    }

    #[test]
    fn test_with_peak_threshold() {
        let graph = HourlyDistributionGraph::with_peak_threshold(0.5);
        assert_eq!(graph.peak_threshold, 0.5);
        assert!(graph.highlight_peaks);

        // Test clamping
        let graph_low = HourlyDistributionGraph::with_peak_threshold(-0.1);
        assert_eq!(graph_low.peak_threshold, 0.0);

        let graph_high = HourlyDistributionGraph::with_peak_threshold(1.5);
        assert_eq!(graph_high.peak_threshold, 1.0);
    }

    #[test]
    fn test_add_data_point() {
        let mut graph = HourlyDistributionGraph::new();
        graph.add_data_point(14, 42, Some("2 PM plays".to_string()));
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].hour, 14);
        assert_eq!(graph.data[0].count, 42);
        assert_eq!(graph.data[0].label, Some("2 PM plays".to_string()));

        // Test invalid hour (should be ignored)
        graph.add_data_point(25, 10, None);
        assert_eq!(graph.data.len(), 1); // Should not add invalid hour
    }

    #[test]
    fn test_set_data_sorts_and_filters() {
        let mut graph = HourlyDistributionGraph::new();
        let data = vec![
            HourlyDataPoint { hour: 15, count: 20, label: None },
            HourlyDataPoint { hour: 9, count: 30, label: None },
            HourlyDataPoint { hour: 25, count: 40, label: None }, // Invalid hour
            HourlyDataPoint { hour: 12, count: 25, label: None },
        ];
        
        graph.set_data(data);
        
        // Should be sorted and filtered (invalid hour removed)
        assert_eq!(graph.data.len(), 3);
        assert_eq!(graph.data[0].hour, 9);
        assert_eq!(graph.data[1].hour, 12);
        assert_eq!(graph.data[2].hour, 15);
    }

    #[test]
    fn test_format_hour_12() {
        let graph = HourlyDistributionGraph::new();
        assert_eq!(graph.format_hour_12(0), "12 AM");
        assert_eq!(graph.format_hour_12(1), "1 AM");
        assert_eq!(graph.format_hour_12(11), "11 AM");
        assert_eq!(graph.format_hour_12(12), "12 PM");
        assert_eq!(graph.format_hour_12(13), "1 PM");
        assert_eq!(graph.format_hour_12(23), "11 PM");
    }

    #[test]
    fn test_format_hour_24() {
        let graph = HourlyDistributionGraph::new();
        assert_eq!(graph.format_hour_24(0), "00:00");
        assert_eq!(graph.format_hour_24(9), "09:00");
        assert_eq!(graph.format_hour_24(15), "15:00");
        assert_eq!(graph.format_hour_24(23), "23:00");
    }

    #[test]
    fn test_is_peak_hour() {
        let mut graph = HourlyDistributionGraph::new();
        
        // Empty data should return false
        assert!(!graph.is_peak_hour(12));

        // Set data with peak at hour 15
        let data = vec![
            HourlyDataPoint { hour: 9, count: 10, label: None },
            HourlyDataPoint { hour: 12, count: 20, label: None },
            HourlyDataPoint { hour: 15, count: 100, label: None }, // Peak
            HourlyDataPoint { hour: 18, count: 85, label: None },  // Above threshold (80% of 100 = 80)
        ];
        graph.set_data(data);
        graph.peak_threshold = 0.8; // 80%

        assert!(!graph.is_peak_hour(9));  // 10 < 80
        assert!(!graph.is_peak_hour(12)); // 20 < 80
        assert!(graph.is_peak_hour(15));  // 100 >= 80
        assert!(graph.is_peak_hour(18));  // 85 >= 80
    }

    #[test]
    fn test_get_max_count() {
        let mut graph = HourlyDistributionGraph::new();
        
        // Empty data should return default
        assert_eq!(graph.get_max_count(), 10.0);
        
        // With data should return max + 10% padding
        let data = vec![
            HourlyDataPoint { hour: 9, count: 10, label: None },
            HourlyDataPoint { hour: 12, count: 50, label: None },
            HourlyDataPoint { hour: 15, count: 30, label: None },
        ];
        graph.set_data(data);
        assert_eq!(graph.get_max_count(), 55.0); // 50 * 1.1
    }

    #[test]
    fn test_prepare_plot_data() {
        let mut graph = HourlyDistributionGraph::new();
        let data = vec![
            HourlyDataPoint { hour: 9, count: 10, label: None },
            HourlyDataPoint { hour: 15, count: 20, label: None },
            HourlyDataPoint { hour: 21, count: 15, label: None },
        ];
        graph.set_data(data);
        
        let plot_data = graph.prepare_plot_data();
        assert_eq!(plot_data.len(), 24); // Should have all 24 hours
        assert_eq!(plot_data[9], (9, 10));   // Hour 9 has data
        assert_eq!(plot_data[15], (15, 20)); // Hour 15 has data
        assert_eq!(plot_data[21], (21, 15)); // Hour 21 has data
        assert_eq!(plot_data[0], (0, 0));    // Hour 0 has no data (0)
        assert_eq!(plot_data[10], (10, 0));  // Hour 10 has no data (0)
    }

    #[test]
    fn test_get_peak_hours() {
        let mut graph = HourlyDistributionGraph::new();
        
        // Empty data should return empty vector
        assert!(graph.get_peak_hours().is_empty());

        let data = vec![
            HourlyDataPoint { hour: 9, count: 10, label: None },
            HourlyDataPoint { hour: 12, count: 90, label: None }, // Peak
            HourlyDataPoint { hour: 15, count: 100, label: None }, // Peak (max)
            HourlyDataPoint { hour: 18, count: 85, label: None },  // Peak (above threshold)
            HourlyDataPoint { hour: 21, count: 70, label: None },  // Below threshold
        ];
        graph.set_data(data);
        graph.peak_threshold = 0.8; // 80% of 100 = 80

        let peak_hours = graph.get_peak_hours();
        assert_eq!(peak_hours.len(), 3);
        assert!(peak_hours.contains(&12)); // 90 >= 80
        assert!(peak_hours.contains(&15)); // 100 >= 80
        assert!(peak_hours.contains(&18)); // 85 >= 80
        assert!(!peak_hours.contains(&9));  // 10 < 80
        assert!(!peak_hours.contains(&21)); // 70 < 80
    }

    #[tokio::test]
    async fn test_render_to_file() {
        let mut graph = HourlyDistributionGraph::new();
        let data = vec![
            HourlyDataPoint { hour: 0, count: 5, label: None },
            HourlyDataPoint { hour: 6, count: 15, label: None },
            HourlyDataPoint { hour: 9, count: 25, label: None },
            HourlyDataPoint { hour: 12, count: 40, label: None },
            HourlyDataPoint { hour: 15, count: 35, label: None },
            HourlyDataPoint { hour: 18, count: 45, label: None },
            HourlyDataPoint { hour: 21, count: 30, label: None },
            HourlyDataPoint { hour: 23, count: 10, label: None },
        ];
        graph.set_data(data);

        let (_, config) = HourlyDistributionGraph::with_config(
            "Play Count by Hour",
            Some("Hour of Day"),
            Some("Number of Plays")
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("hourly_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[test]
    fn test_with_config() {
        let (graph, config) = HourlyDistributionGraph::with_config(
            "Test Chart",
            Some("X Label"),
            Some("Y Label")
        );
        
        assert_eq!(config.title, "Test Chart");
        assert_eq!(config.x_label, Some("X Label".to_string()));
        assert_eq!(config.y_label, Some("Y Label".to_string()));
        assert!(matches!(config.graph_type, crate::GraphType::Histogram));
        assert_eq!(config.width, 1000);
        assert_eq!(config.height, 500);
    }

    #[test]
    fn test_apply_business_hours_theme() {
        let mut config = GraphConfig::default();
        HourlyDistributionGraph::apply_business_hours_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#34495e"); // Regular hours
                assert_eq!(colors[1], "#3498db"); // Business hours
            }
            _ => panic!("Expected custom color scheme"),
        }
    }

    #[test]
    fn test_apply_day_night_theme() {
        let mut config = GraphConfig::default();
        HourlyDistributionGraph::apply_day_night_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#2c3e50"); // Night hours
                assert_eq!(colors[1], "#f1c40f"); // Day hours
            }
            _ => panic!("Expected custom color scheme"),
        }
    }

    #[test]
    fn test_apply_peak_hours_theme() {
        let mut config = GraphConfig::default();
        HourlyDistributionGraph::apply_peak_hours_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#95a5a6"); // Regular hours
                assert_eq!(colors[1], "#e74c3c"); // Peak hours
            }
            _ => panic!("Expected custom color scheme"),
        }
    }
} 