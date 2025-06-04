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