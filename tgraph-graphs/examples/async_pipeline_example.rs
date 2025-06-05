//! Example demonstrating the async graph generation pipeline

use tgraph_graphs::{
    DataPoint, DataSet, GraphConfig, GraphType, StyleConfig, ColorScheme,
    GraphPipeline, GraphGenerationTask, PipelineConfig, GenerationStage,
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    println!("🚀 Async Graph Generation Pipeline Example");

    // Create sample data
    let mut sample_data = Vec::new();
    for i in 0..100 {
        sample_data.push(DataPoint {
            x: i as f64,
            y: (i as f64 * 0.1).sin() * 50.0 + 50.0,
            label: Some(format!("Point {}", i)),
        });
    }

    let datasets = vec![
        DataSet {
            name: "Sine Wave".to_string(),
            data: sample_data.clone(),
            color: Some("#FF6B6B".to_string()),
        },
        DataSet {
            name: "Cosine Wave".to_string(),
            data: sample_data.iter().map(|p| DataPoint {
                x: p.x,
                y: (p.x * 0.1).cos() * 30.0 + 50.0,
                label: p.label.clone(),
            }).collect(),
            color: Some("#4ECDC4".to_string()),
        },
    ];

    // Configure the graph
    let graph_config = GraphConfig {
        graph_type: GraphType::Line,
        title: "Async Generated Graph".to_string(),
        width: 1200,
        height: 800,
        x_label: Some("Time".to_string()),
        y_label: Some("Value".to_string()),
        style: StyleConfig {
            color_scheme: ColorScheme::Vibrant,
            background_color: Some("#FFFFFF".to_string()),
            ..StyleConfig::default()
        },
    };

    // Configure the pipeline
    let pipeline_config = PipelineConfig {
        temp_dir: std::env::temp_dir().join("tgraph_example"),
        max_memory_mb: 256,
        cleanup_timeout_secs: 60,
        enable_progress_reporting: true,
    };

    // Create the pipeline
    let pipeline = GraphPipeline::new(pipeline_config);

    // Create output path
    let output_dir = std::env::current_dir()?.join("target").join("examples");
    tokio::fs::create_dir_all(&output_dir).await?;
    let output_path = output_dir.join("async_generated_graph.png");

    println!("📊 Creating graph generation task...");

    // Create a generation task with progress reporting
    let (task, mut progress_rx) = GraphGenerationTask::new(
        graph_config,
        datasets,
        Some(output_path.clone()),
    ).with_progress_reporting();

    // Create a simple mock renderer for this example
    // In a real application, you'd use LineChartRenderer or another concrete renderer
    use tgraph_graphs::GraphRenderer;
    
    struct ExampleRenderer;
    
    #[async_trait::async_trait]
    impl GraphRenderer for ExampleRenderer {
        async fn render_to_file(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
            path: &std::path::Path,
        ) -> Result<(), tgraph_common::TGraphError> {
            // Create a simple mock file for demonstration
            tokio::fs::write(path, b"Example graph data").await
                .map_err(|e| tgraph_common::TGraphError::new(format!("Failed to write file: {}", e)))?;
            Ok(())
        }

        async fn render_to_bytes(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
        ) -> Result<Vec<u8>, tgraph_common::TGraphError> {
            Ok(b"Example graph data".to_vec())
        }

        fn apply_styling<DB: plotters::prelude::DrawingBackend>(
            &self,
            _root: &plotters::prelude::DrawingArea<DB, plotters::coord::Shift>,
            _config: &GraphConfig,
        ) -> Result<(), tgraph_common::TGraphError>
        where
            DB::ErrorType: std::error::Error + Send + Sync + 'static,
        {
            Ok(())
        }
    }
    
    let renderer = ExampleRenderer;

    println!("🔄 Spawning async generation task...");

    // Spawn the generation task
    let handle = pipeline.spawn_generation(task, renderer).await?;

    // Monitor progress in a separate task
    let progress_task = tokio::spawn(async move {
        let mut last_stage = None;
        while let Some(progress) = progress_rx.recv().await {
            if Some(&progress.stage) != last_stage.as_ref() {
                println!("📈 Stage: {:?}", progress.stage);
                last_stage = Some(progress.stage.clone());
            }
            
            let progress_bar = "█".repeat((progress.progress * 20.0) as usize);
            let empty_bar = "░".repeat(20 - (progress.progress * 20.0) as usize);
            
            println!("   Progress: [{}{}] {:.1}% - {}", 
                progress_bar, empty_bar, progress.progress * 100.0, progress.message);
        }
    });

    // Wait for generation to complete
    println!("⏳ Waiting for generation to complete...");
    
    match handle.await_result().await {
        Ok(result_path) => {
            println!("✅ Graph generated successfully!");
            println!("📁 Output file: {}", result_path.display());
            
            if result_path.exists() {
                let file_size = tokio::fs::metadata(&result_path).await?.len();
                println!("📊 File size: {} bytes", file_size);
            }
        }
        Err(e) => {
            eprintln!("❌ Generation failed: {}", e);
        }
    }

    // Wait for progress monitoring to complete
    let _ = progress_task.await;

    println!("🎉 Example completed!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_example_pipeline() {
        // This test ensures the example code compiles and runs
        let pipeline_config = PipelineConfig {
            temp_dir: std::env::temp_dir().join("tgraph_test"),
            max_memory_mb: 128,
            cleanup_timeout_secs: 10,
            enable_progress_reporting: true,
        };

        let pipeline = GraphPipeline::new(pipeline_config);
        
        // Verify pipeline can be created
        assert!(pipeline.temp_manager.lock().await.cleanup_tasks.is_empty());
    }
} 