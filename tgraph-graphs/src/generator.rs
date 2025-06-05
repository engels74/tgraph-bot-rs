//! Graph generation implementation

use crate::{DataSet, GraphConfig};
use plotters::prelude::*;
use std::path::Path;
use tgraph_common::Result;

/// Graph generator using plotters
pub struct GraphGenerator;

impl GraphGenerator {
    /// Create a new graph generator
    pub fn new() -> Self {
        Self
    }

    /// Test PNG backend setup by creating a simple blank chart
    pub async fn test_png_backend(&self, path: &Path, width: u32, height: u32) -> Result<()> {
        let root = BitMapBackend::new(path, (width, height)).into_drawing_area();
        root.fill(&WHITE)?;
        
        let mut chart = ChartBuilder::on(&root)
            .caption("Test Chart", ("sans-serif", 30).into_font())
            .margin(10)
            .x_label_area_size(40)
            .y_label_area_size(40)
            .build_cartesian_2d(0f32..100f32, 0f32..100f32)?;

        chart.configure_mesh().draw()?;
        root.present()?;
        
        tracing::info!("Successfully created test PNG at {:?}", path);
        Ok(())
    }

    /// Generate a graph and return as bytes
    pub async fn generate(&self, _config: &GraphConfig, _datasets: &[DataSet]) -> Result<Vec<u8>> {
        // TODO: Implement actual graph generation using plotters
        Ok(vec![])
    }

    /// Generate a graph and save to file
    pub async fn generate_to_file(
        &self,
        _config: &GraphConfig,
        _datasets: &[DataSet],
        _path: &str,
    ) -> Result<()> {
        // TODO: Implement file-based graph generation
        Ok(())
    }
}

impl Default for GraphGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_png_backend_setup() {
        let generator = GraphGenerator::new();
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let test_path = temp_dir.path().join("test_chart.png");
        
        let result = generator.test_png_backend(&test_path, 800, 600).await;
        assert!(result.is_ok(), "PNG backend test failed: {:?}", result.err());
        
        // Verify file was created
        assert!(test_path.exists(), "PNG file was not created");
        
        // Verify file has reasonable size (not empty)
        let metadata = std::fs::metadata(&test_path).expect("Failed to read file metadata");
        assert!(metadata.len() > 100, "Generated PNG file is too small");
    }
} 