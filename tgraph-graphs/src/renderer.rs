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
        assert_eq!(renderer.parse_color("#FFFFFF"), RGBColor(255, 255, 255));

        // Test invalid colors (should default to black)
        assert_eq!(renderer.parse_color("invalid"), RGBColor(0, 0, 0));
        assert_eq!(renderer.parse_color("#ZZ0000"), RGBColor(0, 0, 0));
        assert_eq!(renderer.parse_color("#FF00"), RGBColor(0, 0, 0));
    }

    #[test]
    fn test_default_style() {
        let renderer = MockRenderer;
        let style = renderer.default_style();
        assert!(matches!(style.color_scheme, ColorScheme::Default));
        assert_eq!(style.background_color, Some("#FFFFFF".to_string()));
    }

    #[test]
    fn test_background_color() {
        let renderer = MockRenderer;
        
        // Test with custom background
        let mut config = GraphConfig::default();
        config.style.background_color = Some("#FF0000".to_string());
        assert_eq!(renderer.get_background_color(&config), RGBColor(255, 0, 0));

        // Test with default background
        config.style.background_color = None;
        assert_eq!(renderer.get_background_color(&config), RGBColor(255, 255, 255));
    }
} 