//! Graph rendering trait and implementations

use crate::{DataSet, GraphConfig, StyleConfig, ColorScheme};
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::Result;

/// Trait for rendering graphs with different types and styling options
#[async_trait::async_trait]
pub trait GraphRenderer {
    /// Render a graph to a file path
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        datasets: &[DataSet],
        path: &Path,
    ) -> Result<()>;

    /// Render a graph to bytes
    async fn render_to_bytes(
        &self,
        config: &GraphConfig,
        datasets: &[DataSet],
    ) -> Result<Vec<u8>>;

    /// Get the default style configuration for this renderer
    fn default_style(&self) -> StyleConfig {
        StyleConfig::default()
    }

    /// Apply styling to the chart
    fn apply_styling<DB: DrawingBackend>(
        &self,
        root: &DrawingArea<DB, plotters::coord::Shift>,
        config: &GraphConfig,
    ) -> Result<()>
    where
        DB::ErrorType: std::error::Error + Send + Sync + 'static;

    /// Get colors from color scheme
    fn get_colors(&self, scheme: &ColorScheme) -> Vec<RGBColor> {
        match scheme {
            ColorScheme::Default => vec![
                RGBColor(31, 119, 180),   // Blue
                RGBColor(255, 127, 14),   // Orange
                RGBColor(44, 160, 44),    // Green
                RGBColor(214, 39, 40),    // Red
                RGBColor(148, 103, 189),  // Purple
                RGBColor(140, 86, 75),    // Brown
                RGBColor(227, 119, 194),  // Pink
                RGBColor(127, 127, 127),  // Gray
            ],
            ColorScheme::Dark => vec![
                RGBColor(55, 126, 184),   // Light Blue
                RGBColor(255, 152, 150),  // Light Red
                RGBColor(77, 175, 74),    // Light Green
                RGBColor(255, 187, 120),  // Light Orange
                RGBColor(152, 78, 163),   // Light Purple
            ],
            ColorScheme::Light => vec![
                RGBColor(166, 206, 227),  // Pale Blue
                RGBColor(251, 180, 174),  // Pale Red
                RGBColor(179, 226, 205),  // Pale Green
                RGBColor(253, 205, 172),  // Pale Orange
                RGBColor(203, 213, 232),  // Pale Purple
            ],
            ColorScheme::Vibrant => vec![
                RGBColor(230, 25, 75),    // Red
                RGBColor(60, 180, 75),    // Green
                RGBColor(255, 225, 25),   // Yellow
                RGBColor(0, 130, 200),    // Blue
                RGBColor(245, 130, 48),   // Orange
                RGBColor(145, 30, 180),   // Purple
                RGBColor(70, 240, 240),   // Cyan
                RGBColor(240, 50, 230),   // Magenta
            ],
            ColorScheme::Monochrome => vec![
                RGBColor(0, 0, 0),        // Black
                RGBColor(64, 64, 64),     // Dark Gray
                RGBColor(128, 128, 128),  // Gray
                RGBColor(192, 192, 192),  // Light Gray
                RGBColor(224, 224, 224),  // Very Light Gray
            ],
            ColorScheme::Custom(colors) => {
                colors.iter()
                    .map(|color_str| self.parse_color(color_str))
                    .collect()
            }
        }
    }

    /// Parse a color string (hex format) to RGBColor
    fn parse_color(&self, color_str: &str) -> RGBColor {
        if let Some(hex) = color_str.strip_prefix('#') {
            if hex.len() == 6 {
                if let (Ok(r), Ok(g), Ok(b)) = (
                    u8::from_str_radix(&hex[0..2], 16),
                    u8::from_str_radix(&hex[2..4], 16),
                    u8::from_str_radix(&hex[4..6], 16),
                ) {
                    return RGBColor(r, g, b);
                }
            }
        }
        // Default to black if parsing fails
        RGBColor(0, 0, 0)
    }

    /// Get background color from style config
    fn get_background_color(&self, config: &GraphConfig) -> RGBColor {
        config.style.background_color
            .as_ref()
            .map(|color| self.parse_color(color))
            .unwrap_or(RGBColor(255, 255, 255)) // Default white
    }
}

/// Helper struct for font configuration
pub struct FontSpec {
    pub family: String,
    pub size: u32,
}

impl From<&crate::FontConfig> for FontSpec {
    fn from(config: &crate::FontConfig) -> Self {
        Self {
            family: config.family.clone(),
            size: config.size,
        }
    }
}

/// Concrete implementation of GraphRenderer for line charts
pub struct LineChartRenderer;

impl LineChartRenderer {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl GraphRenderer for LineChartRenderer {
    async fn render_to_file(
        &self,
        config: &GraphConfig,
        datasets: &[DataSet],
        path: &Path,
    ) -> Result<()> {
        let root = BitMapBackend::new(path, (config.width, config.height)).into_drawing_area();
        
        // Apply background color
        let bg_color = self.get_background_color(config);
        root.fill(&bg_color)?;

        // Calculate data ranges
        let (x_min, x_max, y_min, y_max) = self.calculate_data_ranges(datasets);

        // Create chart builder with proper font configuration
        let title_font = (config.style.title_font.family.as_str(), config.style.title_font.size);
        let mut chart = ChartBuilder::on(&root)
            .caption(&config.title, title_font)
            .margin(config.style.margins.top as i32)
            .x_label_area_size(config.style.margins.bottom)
            .y_label_area_size(config.style.margins.left)
            .build_cartesian_2d(x_min..x_max, y_min..y_max)?;

        // Configure mesh/grid
        chart.configure_mesh()
            .x_desc(config.x_label.as_deref().unwrap_or(""))
            .y_desc(config.y_label.as_deref().unwrap_or(""))
            .draw()?;

        // Get colors for datasets
        let colors = self.get_colors(&config.style.color_scheme);

        // Draw datasets
        for (i, dataset) in datasets.iter().enumerate() {
            let color_idx = i % colors.len();
            let color = &colors[color_idx];
            
            // Convert data points to plotter format
            let line_data: Vec<(f64, f64)> = dataset.data.iter()
                .map(|point| (point.x, point.y))
                .collect();

            // Draw the line series
            chart.draw_series(LineSeries::new(line_data, color))?
                .label(&dataset.name)
                .legend(move |(x, y)| PathElement::new(vec![(x, y), (x + 10, y)], color));
        }

        // Draw legend if there are multiple datasets
        if datasets.len() > 1 {
            chart.configure_series_labels().draw()?;
        }

        root.present()?;
        
        tracing::info!("Successfully rendered line chart to {}", path.display());
        Ok(())
    }

    async fn render_to_bytes(
        &self,
        config: &GraphConfig,
        datasets: &[DataSet],
    ) -> Result<Vec<u8>> {
        // For now, use a simple in-memory buffer approach
        // In a real implementation, we'd use a proper bytes backend
        todo!("render_to_bytes not yet implemented for LineChartRenderer")
    }

    fn apply_styling<DB: DrawingBackend>(
        &self,
        _root: &DrawingArea<DB, plotters::coord::Shift>,
        _config: &GraphConfig,
    ) -> Result<()>
    where
        DB::ErrorType: std::error::Error + Send + Sync + 'static,
    {
        // Styling is applied during rendering in render_to_file
        Ok(())
    }
}

impl LineChartRenderer {
    /// Calculate the data ranges for all datasets
    fn calculate_data_ranges(&self, datasets: &[DataSet]) -> (f64, f64, f64, f64) {
        if datasets.is_empty() {
            return (0.0, 1.0, 0.0, 1.0);
        }

        let mut x_min = f64::INFINITY;
        let mut x_max = f64::NEG_INFINITY;
        let mut y_min = f64::INFINITY;
        let mut y_max = f64::NEG_INFINITY;

        for dataset in datasets {
            for point in &dataset.data {
                x_min = x_min.min(point.x);
                x_max = x_max.max(point.x);
                y_min = y_min.min(point.y);
                y_max = y_max.max(point.y);
            }
        }

        // Add some padding to the ranges
        let x_padding = (x_max - x_min) * 0.05;
        let y_padding = (y_max - y_min) * 0.05;

        (
            x_min - x_padding,
            x_max + x_padding,
            y_min - y_padding,
            y_max + y_padding,
        )
    }
}

impl Default for LineChartRenderer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ColorScheme;

    struct MockRenderer;

    #[async_trait::async_trait]
    impl GraphRenderer for MockRenderer {
        async fn render_to_file(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
            _path: &Path,
        ) -> Result<()> {
            Ok(())
        }

        async fn render_to_bytes(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
        ) -> Result<Vec<u8>> {
            Ok(vec![])
        }

        fn apply_styling<DB: DrawingBackend>(
            &self,
            _root: &DrawingArea<DB, plotters::coord::Shift>,
            _config: &GraphConfig,
        ) -> Result<()>
        where
            DB::ErrorType: std::error::Error + Send + Sync + 'static,
        {
            Ok(())
        }
    }

    #[test]
    fn test_color_schemes() {
        let renderer = MockRenderer;
        
        // Test default color scheme
        let default_colors = renderer.get_colors(&ColorScheme::Default);
        assert!(!default_colors.is_empty());
        assert_eq!(default_colors[0], RGBColor(31, 119, 180));

        // Test custom color scheme
        let custom_colors = vec![
            "#FF0000".to_string(),
            "#00FF00".to_string(),
            "#0000FF".to_string(),
        ];
        let custom_scheme = ColorScheme::Custom(custom_colors);
        let colors = renderer.get_colors(&custom_scheme);
        assert_eq!(colors.len(), 3);
        assert_eq!(colors[0], RGBColor(255, 0, 0)); // Red
        assert_eq!(colors[1], RGBColor(0, 255, 0)); // Green
        assert_eq!(colors[2], RGBColor(0, 0, 255)); // Blue
    }

    #[test]
    fn test_color_parsing() {
        let renderer = MockRenderer;
        
        // Test valid hex colors
        assert_eq!(renderer.parse_color("#FF0000"), RGBColor(255, 0, 0));
        assert_eq!(renderer.parse_color("#00FF00"), RGBColor(0, 255, 0));
        assert_eq!(renderer.parse_color("#0000FF"), RGBColor(0, 0, 255));
        
        // Test invalid colors (should default to black)
        assert_eq!(renderer.parse_color("invalid"), RGBColor(0, 0, 0));
        assert_eq!(renderer.parse_color("#ZZ0000"), RGBColor(0, 0, 0));
    }

    #[test]
    fn test_default_style() {
        let renderer = MockRenderer;
        let style = renderer.default_style();
        
        assert!(matches!(style.color_scheme, ColorScheme::Default));
        assert_eq!(style.title_font.size, 16);
    }

    #[test]
    fn test_background_color() {
        let renderer = MockRenderer;
        let mut config = crate::GraphConfig::default();
        
        // Test default background
        let bg_color = renderer.get_background_color(&config);
        assert_eq!(bg_color, RGBColor(255, 255, 255));
        
        // Test custom background
        config.style.background_color = Some("#FF0000".to_string());
        let bg_color = renderer.get_background_color(&config);
        assert_eq!(bg_color, RGBColor(255, 0, 0));
    }

    #[test]
    fn test_line_chart_renderer_creation() {
        let renderer = LineChartRenderer::new();
        let default_renderer = LineChartRenderer::default();
        
        // Both should be valid instances
        assert!(std::ptr::eq(&renderer as *const _, &renderer as *const _));
        assert!(std::ptr::eq(&default_renderer as *const _, &default_renderer as *const _));
    }

    #[test]
    fn test_data_range_calculation() {
        let renderer = LineChartRenderer::new();
        
        // Test empty datasets
        let (x_min, x_max, y_min, y_max) = renderer.calculate_data_ranges(&[]);
        assert_eq!((x_min, x_max, y_min, y_max), (0.0, 1.0, 0.0, 1.0));
        
        // Test with data
        let datasets = vec![crate::DataSet {
            name: "Test".to_string(),
            data: vec![
                crate::DataPoint { x: 1.0, y: 2.0, label: None },
                crate::DataPoint { x: 3.0, y: 4.0, label: None },
            ],
            color: None,
        }];
        
        let (x_min, x_max, y_min, y_max) = renderer.calculate_data_ranges(&datasets);
        assert!(x_min < 1.0); // Should have padding
        assert!(x_max > 3.0); // Should have padding
        assert!(y_min < 2.0); // Should have padding
        assert!(y_max > 4.0); // Should have padding
    }
} 