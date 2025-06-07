//! Monthly trends line chart implementation

use crate::{DataSet, GraphConfig, GraphRenderer};
use async_trait::async_trait;
use chrono::{NaiveDate, Datelike};
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::{Result, TGraphError};

/// Data point for monthly trends
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonthlyDataPoint {
    pub year: i32,
    pub month: u32, // 1-12
    pub count: u32,
    pub label: Option<String>,
}

/// Monthly trends line chart renderer for Tautulli data
#[derive(Debug)]
pub struct MonthlyTrendsGraph {
    /// Data points for monthly counts
    pub data: Vec<MonthlyDataPoint>,
    /// Whether to show data points as circles
    pub show_data_points: bool,
    /// Whether to show trend line
    pub show_trend_line: bool,
    /// Whether to compare year-over-year
    pub compare_years: bool,
}

impl MonthlyTrendsGraph {
    /// Create a new monthly trends graph
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            show_data_points: true,
            show_trend_line: true,
            compare_years: false,
        }
    }

    /// Create a new graph with custom title and labels
    pub fn with_config(title: &str, x_label: Option<&str>, y_label: Option<&str>) -> (Self, GraphConfig) {
        let graph = Self::new();
        let mut config = GraphConfig {
            title: title.to_string(),
            x_label: x_label.map(|s| s.to_string()),
            y_label: y_label.map(|s| s.to_string()),
            graph_type: crate::GraphType::Line,
            ..Default::default()
        };
        
        // Optimize dimensions for monthly trends
        config.width = 1000;
        config.height = 600;
        config.style.margins.bottom = 80; // More space for month labels
        config.style.margins.left = 80; // More space for count labels
        
        (graph, config)
    }

    /// Create with year-over-year comparison enabled
    pub fn with_year_comparison() -> Self {
        Self {
            data: Vec::new(),
            show_data_points: true,
            show_trend_line: true,
            compare_years: true,
        }
    }

    /// Create with minimal styling (just line, no points)
    pub fn minimal() -> Self {
        Self {
            data: Vec::new(),
            show_data_points: false,
            show_trend_line: true,
            compare_years: false,
        }
    }

    /// Add a data point for a specific month
    pub fn add_data_point(&mut self, year: i32, month: u32, count: u32, label: Option<String>) {
        if (1..=12).contains(&month) {
            self.data.push(MonthlyDataPoint { year, month, count, label });
        }
    }

    /// Add data point from NaiveDate
    pub fn add_data_point_from_date(&mut self, date: NaiveDate, count: u32, label: Option<String>) {
        self.add_data_point(date.year(), date.month(), count, label);
    }

    /// Set data from monthly aggregated counts
    pub fn set_data(&mut self, mut data: Vec<MonthlyDataPoint>) {
        // Filter valid months and sort by year and month
        data.retain(|d| d.month >= 1 && d.month <= 12);
        data.sort_by_key(|d| (d.year, d.month));
        self.data = data;
    }

    /// Get month abbreviation
    fn month_abbr(&self, month: u32) -> &'static str {
        match month {
            1 => "Jan", 2 => "Feb", 3 => "Mar", 4 => "Apr",
            5 => "May", 6 => "Jun", 7 => "Jul", 8 => "Aug",
            9 => "Sep", 10 => "Oct", 11 => "Nov", 12 => "Dec",
            _ => "???"
        }
    }

    /// Get full month name
    #[allow(dead_code)]
    fn month_name(&self, month: u32) -> &'static str {
        match month {
            1 => "January", 2 => "February", 3 => "March", 4 => "April",
            5 => "May", 6 => "June", 7 => "July", 8 => "August",
            9 => "September", 10 => "October", 11 => "November", 12 => "December",
            _ => "Unknown"
        }
    }

    /// Convert year/month to a continuous x-axis value
    fn date_to_x_value(&self, year: i32, month: u32) -> f64 {
        year as f64 + (month as f64 - 1.0) / 12.0
    }

    /// Get data ranges for axis scaling
    fn get_data_ranges(&self) -> (f64, f64, f64, f64) {
        if self.data.is_empty() {
            return (0.0, 1.0, 0.0, 10.0);
        }

        let x_values: Vec<f64> = self.data.iter()
            .map(|d| self.date_to_x_value(d.year, d.month))
            .collect();
        let y_values: Vec<f64> = self.data.iter()
            .map(|d| d.count as f64)
            .collect();

        let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max = x_values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));
        let y_min = 0.0; // Always start y-axis at 0 for counts
        let y_max = y_values.iter().fold(0.0f64, |a, &b| a.max(b)) * 1.1; // Add 10% padding

        (x_min - 0.1, x_max + 0.1, y_min, y_max)
    }

    /// Convert data to plotters-compatible format
    fn prepare_plot_data(&self) -> Vec<(f64, f64)> {
        self.data.iter()
            .map(|point| (self.date_to_x_value(point.year, point.month), point.count as f64))
            .collect()
    }

    /// Group data by year for year-over-year comparison
    fn group_by_year(&self) -> std::collections::HashMap<i32, Vec<(u32, u32)>> {
        let mut grouped = std::collections::HashMap::new();
        for point in &self.data {
            grouped.entry(point.year)
                .or_insert_with(Vec::new)
                .push((point.month, point.count));
        }
        
        // Sort each year's data by month
        for year_data in grouped.values_mut() {
            year_data.sort_by_key(|&(month, _)| month);
        }
        
        grouped
    }

    /// Calculate growth rate between consecutive months
    pub fn calculate_growth_rates(&self) -> Vec<f64> {
        let mut growth_rates = Vec::new();
        let plot_data = self.prepare_plot_data();
        
        for i in 1..plot_data.len() {
            let prev_count = plot_data[i-1].1;
            let curr_count = plot_data[i].1;
            
            if prev_count > 0.0 {
                let growth = ((curr_count - prev_count) / prev_count) * 100.0;
                growth_rates.push(growth);
            } else {
                growth_rates.push(0.0);
            }
        }
        
        growth_rates
    }

    /// Get peak month (highest count)
    pub fn get_peak_month(&self) -> Option<&MonthlyDataPoint> {
        self.data.iter().max_by_key(|d| d.count)
    }

    /// Get average monthly count
    pub fn get_average_count(&self) -> f64 {
        if self.data.is_empty() {
            return 0.0;
        }
        let total: u32 = self.data.iter().map(|d| d.count).sum();
        total as f64 / self.data.len() as f64
    }

    /// Apply seasonal theme
    pub fn apply_seasonal_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#3498db".to_string(), // Spring - Blue
            "#2ecc71".to_string(), // Summer - Green  
            "#f39c12".to_string(), // Fall - Orange
            "#9b59b6".to_string(), // Winter - Purple
            "#e74c3c".to_string(), // Additional colors
            "#1abc9c".to_string(),
        ]);
    }

    /// Apply business theme
    pub fn apply_business_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#2c3e50".to_string(), // Primary - Dark blue
            "#34495e".to_string(), // Secondary - Dark gray
            "#7f8c8d".to_string(), // Tertiary - Gray
            "#95a5a6".to_string(), // Light gray
            "#bdc3c7".to_string(), // Very light gray
        ]);
    }

    /// Apply growth theme (green for positive, red for negative trends)
    pub fn apply_growth_theme(config: &mut GraphConfig) {
        config.style.color_scheme = crate::ColorScheme::Custom(vec![
            "#27ae60".to_string(), // Positive growth - Green
            "#e74c3c".to_string(), // Negative growth - Red
            "#f39c12".to_string(), // Neutral - Orange
            "#3498db".to_string(), // Info - Blue
            "#9b59b6".to_string(), // Additional - Purple
        ]);
    }
}

impl Default for MonthlyTrendsGraph {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl GraphRenderer for MonthlyTrendsGraph {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        _datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        if self.data.is_empty() {
            return Err(TGraphError::graph("No data available for monthly trends chart"));
        }

        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;

        let (x_min, x_max, y_min, y_max) = self.get_data_ranges();
        let plot_data = self.prepare_plot_data();

        // Create chart
        let title_font = (config.style.title_font.family.as_str(), config.style.title_font.size);
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, title_font)
            .margin(config.style.margins.top as i32)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        // Configure mesh with custom x-axis labels for months/years
        chart.configure_mesh()
            .x_desc(config.x_label.as_deref().unwrap_or("Month"))
            .y_desc(config.y_label.as_deref().unwrap_or("Count"))
            .x_label_formatter(&|x| {
                let year = *x as i32;
                let month_frac = x - year as f64;
                let month = ((month_frac * 12.0) + 1.0) as u32;
                if (1..=12).contains(&month) {
                    format!("{} {}", self.month_abbr(month), year)
                } else {
                    "".to_string()
                }
            })
            .draw()?;

        // Get colors for the line
        let colors = self.get_colors(&config.style.color_scheme);
        let primary_color = colors[0];

        if self.compare_years {
            // Year-over-year comparison mode
            let grouped_data = self.group_by_year();
            for (i, (year, year_data)) in grouped_data.iter().enumerate() {
                let color_idx = i % colors.len();
                let line_color = colors[color_idx];
                
                // Convert to plot data for this year
                let year_plot_data: Vec<(f64, f64)> = year_data.iter()
                    .map(|&(month, count)| (month as f64, count as f64))
                    .collect();

                // Draw line for this year
                chart.draw_series(LineSeries::new(year_plot_data.clone(), &line_color))?
                    .label(format!("{}", year))
                    .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], line_color));

                // Draw data points if enabled
                if self.show_data_points {
                    chart.draw_series(year_plot_data.iter().map(|&(x, y)| {
                        Circle::new((x, y), 3, line_color.filled())
                    }))?;
                }
            }
            
            // Draw legend for multiple years
            chart.configure_series_labels().draw()?;
        } else {
            // Single timeline mode
            if self.show_trend_line {
                chart.draw_series(LineSeries::new(plot_data.clone(), &primary_color))?;
            }

            // Draw data points if enabled
            if self.show_data_points {
                chart.draw_series(plot_data.iter().map(|&(x, y)| {
                    Circle::new((x, y), 4, primary_color.filled())
                }))?;
            }
        }

        root.present()?;
        tracing::info!("Successfully rendered monthly trends chart to {}", path.display());
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        Err(TGraphError::graph("render_to_bytes not implemented for MonthlyTrendsGraph"))
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
    use tempfile::tempdir;

    #[test]
    fn test_monthly_trends_creation() {
        let graph = MonthlyTrendsGraph::new();
        assert!(graph.data.is_empty());
        assert!(graph.show_data_points);
        assert!(graph.show_trend_line);
        assert!(!graph.compare_years);
    }

    #[test]
    fn test_with_year_comparison() {
        let graph = MonthlyTrendsGraph::with_year_comparison();
        assert!(graph.compare_years);
    }

    #[test]
    fn test_minimal() {
        let graph = MonthlyTrendsGraph::minimal();
        assert!(!graph.show_data_points);
        assert!(graph.show_trend_line);
    }

    #[test]
    fn test_add_data_point() {
        let mut graph = MonthlyTrendsGraph::new();
        graph.add_data_point(2023, 6, 150, Some("June plays".to_string()));
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].year, 2023);
        assert_eq!(graph.data[0].month, 6);
        assert_eq!(graph.data[0].count, 150);
        assert_eq!(graph.data[0].label, Some("June plays".to_string()));

        // Test invalid month (should be ignored)
        graph.add_data_point(2023, 13, 100, None);
        assert_eq!(graph.data.len(), 1); // Should not add invalid month
    }

    #[test]
    fn test_add_data_point_from_date() {
        let mut graph = MonthlyTrendsGraph::new();
        let date = NaiveDate::from_ymd_opt(2023, 7, 15).unwrap();
        graph.add_data_point_from_date(date, 200, None);
        
        assert_eq!(graph.data.len(), 1);
        assert_eq!(graph.data[0].year, 2023);
        assert_eq!(graph.data[0].month, 7);
        assert_eq!(graph.data[0].count, 200);
    }

    #[test]
    fn test_set_data_sorts_and_filters() {
        let mut graph = MonthlyTrendsGraph::new();
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 6, count: 150, label: None },
            MonthlyDataPoint { year: 2023, month: 3, count: 100, label: None },
            MonthlyDataPoint { year: 2022, month: 12, count: 200, label: None },
            MonthlyDataPoint { year: 2023, month: 15, count: 50, label: None }, // Invalid month
        ];
        
        graph.set_data(data);
        
        // Should be sorted by year/month and filtered (invalid month removed)
        assert_eq!(graph.data.len(), 3);
        assert_eq!(graph.data[0].year, 2022);
        assert_eq!(graph.data[0].month, 12);
        assert_eq!(graph.data[1].year, 2023);
        assert_eq!(graph.data[1].month, 3);
        assert_eq!(graph.data[2].year, 2023);
        assert_eq!(graph.data[2].month, 6);
    }

    #[test]
    fn test_month_abbreviations() {
        let graph = MonthlyTrendsGraph::new();
        assert_eq!(graph.month_abbr(1), "Jan");
        assert_eq!(graph.month_abbr(6), "Jun");
        assert_eq!(graph.month_abbr(12), "Dec");
        assert_eq!(graph.month_abbr(13), "???");
    }

    #[test]
    fn test_month_names() {
        let graph = MonthlyTrendsGraph::new();
        assert_eq!(graph.month_name(1), "January");
        assert_eq!(graph.month_name(6), "June");
        assert_eq!(graph.month_name(12), "December");
        assert_eq!(graph.month_name(13), "Unknown");
    }

    #[test]
    fn test_date_to_x_value() {
        let graph = MonthlyTrendsGraph::new();
        assert_eq!(graph.date_to_x_value(2023, 1), 2023.0); // January
        assert_eq!(graph.date_to_x_value(2023, 7), 2023.5); // July (6/12 = 0.5)
        assert_eq!(graph.date_to_x_value(2023, 12), 2_023.916_666_666_666_7); // December (11/12)
    }

    #[test]
    fn test_get_data_ranges() {
        let mut graph = MonthlyTrendsGraph::new();
        
        // Empty data should return defaults
        let (x_min, x_max, y_min, y_max) = graph.get_data_ranges();
        assert_eq!(x_min, 0.0);
        assert_eq!(x_max, 1.0);
        assert_eq!(y_min, 0.0);
        assert_eq!(y_max, 10.0);
        
        // With data
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 1, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 6, count: 200, label: None },
        ];
        graph.set_data(data);
        
        let (x_min, x_max, y_min, y_max) = graph.get_data_ranges();
        assert!(x_min < 2023.0); // Should have padding
        assert!(x_max > 2023.5); // Should have padding
        assert_eq!(y_min, 0.0); // Always starts at 0
        assert_eq!(y_max, 220.0); // 200 * 1.1 padding
    }

    #[test]
    fn test_calculate_growth_rates() {
        let mut graph = MonthlyTrendsGraph::new();
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 1, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 2, count: 110, label: None }, // 10% growth
            MonthlyDataPoint { year: 2023, month: 3, count: 99, label: None },  // ~-10% growth
        ];
        graph.set_data(data);
        
        let growth_rates = graph.calculate_growth_rates();
        assert_eq!(growth_rates.len(), 2);
        assert!((growth_rates[0] - 10.0).abs() < 0.1); // ~10% growth
        assert!(growth_rates[1] < 0.0); // Negative growth
    }

    #[test]
    fn test_get_peak_month() {
        let mut graph = MonthlyTrendsGraph::new();
        
        // Empty data should return None
        assert!(graph.get_peak_month().is_none());
        
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 1, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 6, count: 250, label: None }, // Peak
            MonthlyDataPoint { year: 2023, month: 12, count: 150, label: None },
        ];
        graph.set_data(data);
        
        let peak = graph.get_peak_month().unwrap();
        assert_eq!(peak.month, 6);
        assert_eq!(peak.count, 250);
    }

    #[test]
    fn test_get_average_count() {
        let mut graph = MonthlyTrendsGraph::new();
        
        // Empty data should return 0
        assert_eq!(graph.get_average_count(), 0.0);
        
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 1, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 2, count: 200, label: None },
            MonthlyDataPoint { year: 2023, month: 3, count: 300, label: None },
        ];
        graph.set_data(data);
        
        assert_eq!(graph.get_average_count(), 200.0); // (100+200+300)/3
    }

    #[test]
    fn test_group_by_year() {
        let mut graph = MonthlyTrendsGraph::new();
        let data = vec![
            MonthlyDataPoint { year: 2022, month: 12, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 1, count: 150, label: None },
            MonthlyDataPoint { year: 2023, month: 6, count: 200, label: None },
            MonthlyDataPoint { year: 2022, month: 6, count: 120, label: None },
        ];
        graph.set_data(data);
        
        let grouped = graph.group_by_year();
        assert_eq!(grouped.len(), 2); // 2022 and 2023
        
        let year_2022 = &grouped[&2022];
        let year_2023 = &grouped[&2023];
        
        assert_eq!(year_2022.len(), 2);
        assert_eq!(year_2023.len(), 2);
        
        // Should be sorted by month within each year
        assert_eq!(year_2022[0].0, 6);  // June comes before December
        assert_eq!(year_2022[1].0, 12);
        assert_eq!(year_2023[0].0, 1);  // January comes before June
        assert_eq!(year_2023[1].0, 6);
    }

    #[tokio::test]
    async fn test_render_to_file() {
        let mut graph = MonthlyTrendsGraph::new();
        let data = vec![
            MonthlyDataPoint { year: 2023, month: 1, count: 100, label: None },
            MonthlyDataPoint { year: 2023, month: 2, count: 120, label: None },
            MonthlyDataPoint { year: 2023, month: 3, count: 150, label: None },
            MonthlyDataPoint { year: 2023, month: 4, count: 180, label: None },
            MonthlyDataPoint { year: 2023, month: 5, count: 200, label: None },
            MonthlyDataPoint { year: 2023, month: 6, count: 220, label: None },
        ];
        graph.set_data(data);

        let (_, config) = MonthlyTrendsGraph::with_config(
            "Monthly Play Count Trends",
            Some("Month"),
            Some("Play Count")
        );

        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("monthly_trends_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_ok());
        assert!(file_path.exists());
    }

    #[tokio::test]
    async fn test_render_empty_data_error() {
        let graph = MonthlyTrendsGraph::new();
        let config = GraphConfig::default();
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join("empty_test.png");
        
        let result = graph.render_to_file(&config, &[], &file_path).await;
        assert!(result.is_err());
    }

    #[test]
    fn test_with_config() {
        let (graph, config) = MonthlyTrendsGraph::with_config(
            "Test Chart",
            Some("X Label"),
            Some("Y Label")
        );
        
        assert_eq!(config.title, "Test Chart");
        assert_eq!(config.x_label, Some("X Label".to_string()));
        assert_eq!(config.y_label, Some("Y Label".to_string()));
        assert!(matches!(config.graph_type, crate::GraphType::Line));
        assert_eq!(config.width, 1000);
        assert_eq!(config.height, 600);
    }

    #[test]
    fn test_apply_seasonal_theme() {
        let mut config = GraphConfig::default();
        MonthlyTrendsGraph::apply_seasonal_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert!(!colors.is_empty());
                assert_eq!(colors[0], "#3498db"); // Spring - Blue
                assert_eq!(colors[1], "#2ecc71"); // Summer - Green
            }
            _ => panic!("Expected custom color scheme"),
        }
    }

    #[test]
    fn test_apply_growth_theme() {
        let mut config = GraphConfig::default();
        MonthlyTrendsGraph::apply_growth_theme(&mut config);
        
        match config.style.color_scheme {
            crate::ColorScheme::Custom(colors) => {
                assert_eq!(colors[0], "#27ae60"); // Positive - Green
                assert_eq!(colors[1], "#e74c3c"); // Negative - Red
            }
            _ => panic!("Expected custom color scheme"),
        }
    }
} 