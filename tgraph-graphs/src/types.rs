//! Graph types and data structures

use serde::{Deserialize, Serialize};

/// Supported graph types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GraphType {
    Line,
    Bar,
    Pie,
    Scatter,
    Histogram,
}

/// Graph configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphConfig {
    pub graph_type: GraphType,
    pub title: String,
    pub width: u32,
    pub height: u32,
    pub x_label: Option<String>,
    pub y_label: Option<String>,
    pub style: StyleConfig,
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            graph_type: GraphType::Line,
            title: "Graph".to_string(),
            width: 800,
            height: 600,
            x_label: None,
            y_label: None,
            style: StyleConfig::default(),
        }
    }
}

/// Data point for graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    pub x: f64,
    pub y: f64,
    pub label: Option<String>,
}

/// Graph data set
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataSet {
    pub name: String,
    pub data: Vec<DataPoint>,
    pub color: Option<String>,
}

/// Color scheme for graphs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ColorScheme {
    Default,
    Dark,
    Light,
    Vibrant,
    Monochrome,
    Custom(Vec<String>),
}

/// Font configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FontConfig {
    pub family: String,
    pub size: u32,
}

impl Default for FontConfig {
    fn default() -> Self {
        Self {
            family: "sans-serif".to_string(),
            size: 12,
        }
    }
}

/// Margin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarginConfig {
    pub top: u32,
    pub right: u32,
    pub bottom: u32,
    pub left: u32,
}

impl Default for MarginConfig {
    fn default() -> Self {
        Self {
            top: 20,
            right: 20,
            bottom: 40,
            left: 60,
        }
    }
}

/// Grid line configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GridConfig {
    pub show_x: bool,
    pub show_y: bool,
    pub color: Option<String>,
    pub style: GridStyle,
}

impl Default for GridConfig {
    fn default() -> Self {
        Self {
            show_x: true,
            show_y: true,
            color: None,
            style: GridStyle::Solid,
        }
    }
}

/// Grid line styles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GridStyle {
    Solid,
    Dashed,
    Dotted,
}

/// Comprehensive styling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StyleConfig {
    pub color_scheme: ColorScheme,
    pub background_color: Option<String>,
    pub title_font: FontConfig,
    pub axis_font: FontConfig,
    pub label_font: FontConfig,
    pub margins: MarginConfig,
    pub grid: GridConfig,
}

impl Default for StyleConfig {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::Default,
            background_color: Some("#FFFFFF".to_string()),
            title_font: FontConfig {
                family: "sans-serif".to_string(),
                size: 16,
            },
            axis_font: FontConfig::default(),
            label_font: FontConfig::default(),
            margins: MarginConfig::default(),
            grid: GridConfig::default(),
        }
    }
} 