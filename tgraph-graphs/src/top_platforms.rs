//! Top platforms/users horizontal bar chart implementation

use crate::{DataSet, GraphConfig, GraphRenderer};
use async_trait::async_trait;
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::{Result, TGraphError};

/// Data point for top platforms or users
#[derive(Debug, Clone)]
pub struct TopItemDataPoint {
    pub name: String,
    pub count: u32,
    pub percentage: Option<f64>, // Percentage of total
    pub label: Option<String>,
}

/// Top platforms/users horizontal bar chart renderer
#[derive(Debug)]
pub struct TopPlatformsGraph {
    /// Data points sorted by count (descending)
    pub data: Vec<TopItemDataPoint>,
    /// Maximum number of items to display
    pub limit: usize,
    /// Whether to show percentages
    pub show_percentages: bool,
    /// Chart title (e.g., "Top Platforms", "Top Users")
    pub chart_type: String,
}

impl TopPlatformsGraph {
    /// Create a new top platforms graph
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            limit: 10, // Show top 10 by default
            show_percentages: true,
            chart_type: "Top Items".to_string(),
        }
    }

    /// Create a new graph with custom title and labels
    pub fn with_config(
        title: &str, 
        x_label: Option<&str>, 
        y_label: Option<&str>,
        limit: usize
    ) -> (Self, GraphConfig) {
        let mut graph = Self::new();
        graph.limit = limit;
        graph.chart_type = title.to_string();
        
        let mut config = GraphConfig {
            title: title.to_string(),
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
            graph_type: crate::GraphType::Bar,
            ..Default::default()
        };
        
        // Optimize dimensions for horizontal bar chart
        config.width = 800;
        config.height = std::cmp::max(400, 40 * limit as u32); // Height based on number of items
        config.style.margins.bottom = 60;
        config.style.margins.left = 150; // More space for item names
        config.style.margins.right = 50; // Space for values
        
        (graph, config)
    }

    /// Create for top platforms specifically
    pub fn for_platforms(limit: usize) -> Self {
        Self {
            data: Vec::new(),
            limit,
            show_percentages: true,
            chart_type: "Top Platforms".to_string(),
        }
    }

    /// Create for top users specifically
    pub fn for_users(limit: usize) -> Self {
        Self {
            data: Vec::new(),
            limit,
            show_percentages: true,
            chart_type: "Top Users".to_string(),
        }
    }

    /// Create without percentage display
    pub fn without_percentages() -> Self {
        Self {
            data: Vec::new(),
            limit: 10,
            show_percentages: false,
            chart_type: "Top Items".to_string(),
        }
    }

    /// Add a data point
    pub fn add_data_point(&mut self, name: String, count: u32, label: Option<String>) {
        self.data.push(TopItemDataPoint { 
            name, 
            count, 
            percentage: None, // Will be calculated when needed
            label 
        });
    }

    /// Set data and automatically sort and limit
    pub fn set_data(&mut self, mut data: Vec<TopItemDataPoint>) {
        // Sort by count descending
        data.sort_by(|a, b| b.count.cmp(&a.count));
        
        // Take only the top N items
        data.truncate(self.limit);
        
        // Calculate percentages if enabled
        if self.show_percentages {
            let total_count: u32 = data.iter().map(|d| d.count).sum();
            if total_count > 0 {
                for item in &mut data {
                    item.percentage = Some((item.count as f64 / total_count as f64) * 100.0);
                }
            }
        }
        
        self.data = data;
    }

    /// Get the top N items (already sorted and limited)
    pub fn get_top_items(&self, n: usize) -> Vec<&TopItemDataPoint> {
        self.data.iter().take(n).collect()
    }

    /// Get max count for x-axis scaling
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

    /// Truncate long names for display
    fn truncate_name(&self, name: &str, max_length: usize) -> String {
        if name.len() <= max_length {
            name.to_string()
        } else {
            format!("{}...", &name[..max_length.saturating_sub(3)])
        }
    }

    /// Format label with count and optional percentage
    fn format_label(&self, item: &TopItemDataPoint) -> String {
        if self.show_percentages {
            if let Some(percentage) = item.percentage {
                format!("{} ({:.1}%)", item.count, percentage)
            } else {
                item.count.to_string()
            }
        } else {
            item.count.to_string()
        }
    }

    /// Apply gradient theme for top items
    pub fn apply_gradient_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#e74c3c".to_string(), // #1 - Red
            "#f39c12".to_string(), // #2 - Orange  
            "#f1c40f".to_string(), // #3 - Yellow
            "#2ecc71".to_string(), // #4 - Green
            "#3498db".to_string(), // #5 - Blue
            "#9b59b6".to_string(), // #6 - Purple
            "#1abc9c".to_string(), // #7 - Teal
            "#34495e".to_string(), // #8 - Dark gray
            "#95a5a6".to_string(), // #9 - Gray
            "#7f8c8d".to_string(), // #10 - Light gray
        ]);
    }

    /// Apply platform-specific theme
    pub fn apply_platform_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#ff6b6b".to_string(), // Primary platform - red
            "#4ecdc4".to_string(), // Secondary - teal
            "#45b7d1".to_string(), // Third - blue
            "#96ceb4".to_string(), // Fourth - green
            "#feca57".to_string(), // Fifth - yellow
            "#ff9ff3".to_string(), // Sixth - pink
            "#54a0ff".to_string(), // Seventh - light blue
            "#5f27cd".to_string(), // Eighth - purple
            "#00d2d3".to_string(), // Ninth - cyan
            "#ff9f43".to_string(), // Tenth - orange
        ]);
    }

    /// Apply user ranking theme
    pub fn apply_user_ranking_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#ffd700".to_string(), // Gold - #1
            "#c0c0c0".to_string(), // Silver - #2
            "#cd7f32".to_string(), // Bronze - #3
            "#4a90e2".to_string(), // Regular users - blue shades
            "#357abd".to_string(),
            "#2e6da4".to_string(),
            "#1e4a72".to_string(),
            "#0f2537".to_string(),
            "#95a5a6".to_string(),
            "#7f8c8d".to_string(),
        ]);
    }
}

impl Default for TopPlatformsGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRenderer for TopPlatformsGraph {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        _datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        if self.data.is_empty() {
            return Err(TGraphError::graph("No data available for top platforms chart"));
        }

        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;

        let max_count = self.get_max_count();
        let num_items = self.data.len();

        // Create chart with horizontal orientation
        let title_font = (config.style.title_font.family.as_str(), config.style.title_font.size);
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, title_font)
            .margin(config.style.margins.top as i32)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(0.0..max_count, 0..num_items)?;

        // Configure mesh
        chart.configure_mesh()
            .x_desc(config.x_label.as_deref().unwrap_or("Count"))
            .y_desc(config.y_label.as_deref().unwrap_or(&self.chart_type))
            .y_label_formatter(&|y| {
                if *y < self.data.len() {
                    let name = &self.data[*y].name;
                    self.truncate_name(name, 20) // Truncate long names
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // Get colors for bars
        let colors = self.get_colors(&config.style.color_scheme);

        // Draw horizontal bars
        for (i, item) in self.data.iter().enumerate() {
            let color_idx = i % colors.len();
            let bar_color = &colors[color_idx];

            // Bar dimensions
            let bar_height = 0.7; // Slightly less than 1 to create gaps
            let y_start = i as f64 - bar_height / 2.0;
            let y_end = i as f64 + bar_height / 2.0;

            // Draw horizontal bar
            chart.draw_series(std::iter::once(Rectangle::new([
                (0.0, y_start as usize),
                (item.count as f64, y_end as usize)
            ], bar_color.filled())))?;

            // Add value label at the end of the bar
            let label_text = self.format_label(item);
            chart.draw_series(std::iter::once(Text::new(
                label_text,
                (item.count as f64 + max_count * 0.01, i), // Slight offset from bar end
                ("sans-serif", 12).into_font().color(&BLACK)
            )))?;
        }

        root.present()?;
        tracing::info!("Successfully rendered top platforms chart to {}", path.display());
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        Err(TGraphError::graph("render_to_bytes not implemented for TopPlatformsGraph"))
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
    fn test_top_platforms_creation() {
        let graph = TopPlatformsGraph::new();
        assert!(graph.data.is_empty());
        assert_eq!(graph.limit, 10);
        assert!(graph.show_percentages);
        assert_eq!(graph.chart_type, "Top Items");
    }

    #[test]
    fn test_for_platforms() {
        let graph = TopPlatformsGraph::for_platforms(5);
        assert_eq!(graph.limit, 5);
        assert_eq!(graph.chart_type, "Top Platforms");
    }

    #[test]
    fn test_for_users() {
        let graph = TopPlatformsGraph::for_users(15);
        assert_eq!(graph.limit, 15);
        assert_eq!(graph.chart_type, "Top Users");
    }

    #[test]
    fn test_without_percentages() {
        let graph = TopPlatformsGraph::without_percentages();
        assert!(!graph.show_percentages);
    }

    #[test]
    fn test_add_data_point() {
        let mut graph = TopPlatformsGraph::new();
        graph.add_data_point("Plex".to_string(), 150, Some("Main platform".to_string()));
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].name, "Plex");
        assert_eq!(graph.data[0].count, 150);
        assert_eq!(graph.data[0].label, Some("Main platform".to_string()));
    }

    #[test]
    fn test_set_data_sorts_and_limits() {
        let mut graph = TopPlatformsGraph::new();
        graph.limit = 3; // Only top 3
        
        let data = vec![
            TopItemDataPoint { name: "Platform B".to_string(), count: 50, percentage: None, label: None },
            TopItemDataPoint { name: "Platform A".to_string(), count: 100, percentage: None, label: None },
            TopItemDataPoint { name: "Platform D".to_string(), count: 10, percentage: None, label: None },
            TopItemDataPoint { name: "Platform C".to_string(), count: 75, percentage: None, label: None },
        ];
        
        graph.set_data(data);
        
        // Should be sorted by count (desc) and limited to top 3
        assert_eq!(graph.data.len(), 3);
        assert_eq!(graph.data[0].name, "Platform A"); // 100
        assert_eq!(graph.data[1].name, "Platform C"); // 75
        assert_eq!(graph.data[2].name, "Platform B"); // 50
        // Platform D (10) should be excluded
    }

    #[test]
    fn test_set_data_calculates_percentages() {
        let mut graph = TopPlatformsGraph::new();
        graph.show_percentages = true;
        
        let data = vec![
            TopItemDataPoint { name: "A".to_string(), count: 60, percentage: None, label: None },
            TopItemDataPoint { name: "B".to_string(), count: 40, percentage: None, label: None },
        ];
        
        graph.set_data(data);
        
        // Total = 100, so A = 60%, B = 40%
        assert!((graph.data[0].percentage.unwrap() - 60.0).abs() < 0.1);
        assert!((graph.data[1].percentage.unwrap() - 40.0).abs() < 0.1);
    }

    #[test]
    fn test_get_top_items() {
        let mut graph = TopPlatformsGraph::new();
        let data = vec![
            TopItemDataPoint { name: "A".to_string(), count: 100, percentage: None, label: None },
            TopItemDataPoint { name: "B".to_string(), count: 50, percentage: None, label: None },
            TopItemDataPoint { name: "C".to_string(), count: 25, percentage: None, label: None },
        ];
        graph.set_data(data);
        
        let top_2 = graph.get_top_items(2);
        assert_eq!(top_2.len(), 2);
        assert_eq!(top_2[0].name, "A");
        assert_eq!(top_2[1].name, "B");
    }

    #[test]
    fn test_get_max_count() {
        let mut graph = TopPlatformsGraph::new();
        
        // Empty data should return default
        assert_eq!(graph.get_max_count(), 10.0);
        
        // With data should return max + 10% padding
        let data = vec![
            TopItemDataPoint { name: "A".to_string(), count: 100, percentage: None, label: None },
            TopItemDataPoint { name: "B".to_string(), count: 50, percentage: None, label: None },
        ];
        graph.set_data(data);
        assert_eq!(graph.get_max_count(), 110.0); // 100 * 1.1
    }

    #[test]
    fn test_truncate_name() {
        let graph = TopPlatformsGraph::new();
        assert_eq!(graph.truncate_name("Short", 10), "Short");
        assert_eq!(graph.truncate_name("This is a very long platform name", 15), "This is a ve...");
        assert_eq!(graph.truncate_name("Exactly15Chars!", 15), "Exactly15Chars!");
    }

    #[test]
    fn test_format_label() {
        let mut graph = TopPlatformsGraph::new();
        
        // With percentages
        graph.show_percentages = true;
        let item_with_pct = TopItemDataPoint { 
            name: "Test".to_string(), 
            count: 50, 
            percentage: Some(25.5), 
            label: None 
        };
        assert_eq!(graph.format_label(&item_with_pct), "50 (25.5%)");
        
        // Without percentages
        graph.show_percentages = false;
        assert_eq!(graph.format_label(&item_with_pct), "50");
        
        // With percentages but no percentage value
        graph.show_percentages = true;
        let item_no_pct = TopItemDataPoint { 
            name: "Test".to_string(), 
            count: 30, 
            percentage: None, 
            label: None 
        };
        assert_eq!(graph.format_label(&item_no_pct), "30");
    }

    #[tokio::test]
    async fn test_render_to_file() {
        let mut graph = TopPlatformsGraph::for_platforms(5);
        let data = vec![
            TopItemDataPoint { name: "Plex".to_string(), count: 150, percentage: None, label: None },
            TopItemDataPoint { name: "Jellyfin".to_string(), count: 100, percentage: None, label: None },
            TopItemDataPoint { name: "Emby".to_string(), count: 75, percentage: None, label: None },
            TopItemDataPoint { name: "Kodi".to_string(), count: 50, percentage: None, label: None },
            TopItemDataPoint { name: "VLC".to_string(), count: 25, percentage: None, label: None },
        ];
        graph.set_data(data);

        let (_, config) = TopPlatformsGraph::with_config(
            "Top Platforms by Play Count",
            Some("Play Count"),
            Some("Platform"),
            5
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("top_platforms_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_render_empty_data_error() {
        let graph = TopPlatformsGraph::new();
        let config = GraphConfig::default();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_with_config() {
        let (graph, config) = TopPlatformsGraph::with_config(
            "Test Chart",
            Some("X Label"),
            Some("Y Label"),
            8
        );
        
        assert_eq!(config.title, "Test Chart");
        assert_eq!(config.x_label, Some("X Label".to_string()));
        assert_eq!(config.y_label, Some("Y Label".to_string()));
        assert!(matches!(config.graph_type, crate::GraphType::Bar));
        assert_eq!(graph.limit, 8);
        assert_eq!(config.height, 320); // 40 * 8
    }

    #[test]
    fn test_apply_gradient_theme() {
        let mut config = GraphConfig::default();
        TopPlatformsGraph::apply_gradient_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert_eq!(colors.len(), 10);
                assert_eq!(colors[0], "#e74c3c"); // #1 - Red
                assert_eq!(colors[1], "#f39c12"); // #2 - Orange
                assert_eq!(colors[2], "#f1c40f"); // #3 - Yellow
            }
            _ => panic!("Expected custom color scheme"),
        }
    }

    #[test]
    fn test_apply_user_ranking_theme() {
        let mut config = GraphConfig::default();
        TopPlatformsGraph::apply_user_ranking_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert_eq!(colors[0], "#ffd700"); // Gold
                assert_eq!(colors[1], "#c0c0c0"); // Silver
                assert_eq!(colors[2], "#cd7f32"); // Bronze
            }
            _ => panic!("Expected custom color scheme"),
        }
    }
} 