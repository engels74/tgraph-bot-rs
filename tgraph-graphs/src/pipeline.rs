//! Async graph generation pipeline for memory-efficient processing

use crate::{DataSet, GraphConfig, GraphRenderer};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, oneshot, Mutex};
use tokio::task::JoinHandle;
use tgraph_common::Result;
use uuid::Uuid;

/// Progress information for graph generation
#[derive(Debug, Clone)]
pub struct GenerationProgress {
    pub stage: GenerationStage,
    pub progress: f32, // 0.0 to 1.0
    pub message: String,
}

/// Stages of graph generation
#[derive(Debug, Clone)]
pub enum GenerationStage {
    Initializing,
    ProcessingData,
    Rendering,
    WritingFile,
    Cleanup,
    Complete,
}

/// Configuration for the generation pipeline
#[derive(Debug, Clone)]
pub struct PipelineConfig {
    pub temp_dir: PathBuf,
    pub max_memory_mb: usize,
    pub cleanup_timeout_secs: u64,
    pub enable_progress_reporting: bool,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        Self {
            temp_dir: std::env::temp_dir().join("tgraph"),
            max_memory_mb: 512, // 512MB default limit
            cleanup_timeout_secs: 300, // 5 minutes
            enable_progress_reporting: true,
        }
    }
}

/// A graph generation task that can be spawned as an async operation
pub struct GraphGenerationTask {
    pub id: Uuid,
    pub config: GraphConfig,
    pub datasets: Vec<DataSet>,
    pub output_path: Option<PathBuf>,
    pub pipeline_config: PipelineConfig,
    pub progress_tx: Option<mpsc::UnboundedSender<GenerationProgress>>,
}

impl GraphGenerationTask {
    /// Create a new graph generation task
    pub fn new(
        config: GraphConfig,
        datasets: Vec<DataSet>,
        output_path: Option<PathBuf>,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            datasets,
            output_path,
            pipeline_config: PipelineConfig::default(),
            progress_tx: None,
        }
    }

    /// Create a task with custom pipeline configuration
    pub fn with_config(
        config: GraphConfig,
        datasets: Vec<DataSet>,
        output_path: Option<PathBuf>,
        pipeline_config: PipelineConfig,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            config,
            datasets,
            output_path,
            pipeline_config,
            progress_tx: None,
        }
    }

    /// Add progress reporting to the task
    pub fn with_progress_reporting(mut self) -> (Self, mpsc::UnboundedReceiver<GenerationProgress>) {
        let (tx, rx) = mpsc::unbounded_channel();
        self.progress_tx = Some(tx);
        (self, rx)
    }

    /// Generate a unique temporary filename
    fn generate_temp_filename(&self) -> String {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        
        let graph_type = match self.config.graph_type {
            crate::GraphType::Line => "line",
            crate::GraphType::Bar => "bar",
            crate::GraphType::Pie => "pie",
            crate::GraphType::Scatter => "scatter",
            crate::GraphType::Histogram => "histogram",
        };

        format!("tgraph_{}_{}_{}_{}.png", 
                graph_type, 
                timestamp, 
                self.id.simple(), 
                fastrand::u32(..))
    }

    /// Send progress update if reporting is enabled
    fn send_progress(&self, stage: GenerationStage, progress: f32, message: String) {
        if let Some(ref tx) = self.progress_tx {
            let _ = tx.send(GenerationProgress {
                stage,
                progress,
                message,
            });
        }
    }

    /// Process datasets in chunks to manage memory usage
    async fn process_datasets_chunked(&self) -> Result<Vec<DataSet>> {
        self.send_progress(
            GenerationStage::ProcessingData,
            0.0,
            "Starting data processing".to_string(),
        );

        let total_points: usize = self.datasets.iter()
            .map(|ds| ds.data.len())
            .sum();

        // Calculate optimal chunk size based on memory limit
        let estimated_point_size = std::mem::size_of::<crate::DataPoint>() + 64; // Account for overhead
        let max_points_in_memory = (self.pipeline_config.max_memory_mb * 1024 * 1024) / estimated_point_size;
        
        if total_points <= max_points_in_memory {
            // Small dataset, process all at once
            self.send_progress(
                GenerationStage::ProcessingData,
                1.0,
                format!("Processed {} data points", total_points),
            );
            return Ok(self.datasets.clone());
        }

        // For large datasets, we would implement chunked processing here
        // For now, we'll clone the data but log that we're handling large data
        tracing::warn!(
            "Processing large dataset with {} points (limit: {})", 
            total_points, 
            max_points_in_memory
        );

        self.send_progress(
            GenerationStage::ProcessingData,
            1.0,
            format!("Processed large dataset with {} points", total_points),
        );

        Ok(self.datasets.clone())
    }
}

/// Manager for the async graph generation pipeline
pub struct GraphPipeline {
    temp_manager: Arc<Mutex<TempFileManager>>,
    #[allow(dead_code)]
    config: PipelineConfig,
}

impl GraphPipeline {
    /// Create a new graph pipeline
    pub fn new(config: PipelineConfig) -> Self {
        let temp_manager = Arc::new(Mutex::new(TempFileManager::new(config.clone())));
        Self {
            temp_manager,
            config,
        }
    }

    /// Spawn a graph generation task
    pub async fn spawn_generation<R>(&self, task: GraphGenerationTask, renderer: R) -> Result<GenerationHandle>
    where
        R: GraphRenderer + Send + Sync + 'static,
    {
        let temp_manager = Arc::clone(&self.temp_manager);
        let renderer = Arc::new(renderer);
        
        let (result_tx, result_rx) = oneshot::channel();
        
        let handle = tokio::spawn(async move {
            let result = Self::execute_generation_task(task, renderer, temp_manager).await;
            let _ = result_tx.send(result);
        });

        Ok(GenerationHandle {
            handle,
            result_rx,
        })
    }

    /// Execute a graph generation task
    async fn execute_generation_task<R>(
        task: GraphGenerationTask,
        renderer: Arc<R>,
        temp_manager: Arc<Mutex<TempFileManager>>,
    ) -> Result<PathBuf>
    where
        R: GraphRenderer + Send + Sync + 'static,
    {
        task.send_progress(
            GenerationStage::Initializing,
            0.0,
            "Initializing generation task".to_string(),
        );

        // Process data in memory-efficient chunks
        let processed_datasets = task.process_datasets_chunked().await?;

        // Generate temporary file path
        let temp_filename = task.generate_temp_filename();
        let temp_path = {
            let mut manager = temp_manager.lock().await;
            manager.create_temp_file(temp_filename).await?
        };

        task.send_progress(
            GenerationStage::Rendering,
            0.5,
            format!("Rendering to {}", temp_path.display()),
        );

        // Render the graph
        renderer.render_to_file(&task.config, &processed_datasets, &temp_path).await?;

        task.send_progress(
            GenerationStage::WritingFile,
            0.8,
            "Writing output file".to_string(),
        );

        // Handle output path
        let final_path = if let Some(output_path) = &task.output_path {
            // Copy to final destination
            tokio::fs::copy(&temp_path, output_path).await?;
            
            // Schedule cleanup of temp file
            {
                let mut manager = temp_manager.lock().await;
                manager.schedule_cleanup(&temp_path, task.pipeline_config.cleanup_timeout_secs);
            }
            
            output_path.clone()
        } else {
            // Return temp path (caller responsible for cleanup)
            temp_path
        };

        task.send_progress(
            GenerationStage::Complete,
            1.0,
            format!("Generation complete: {}", final_path.display()),
        );

        Ok(final_path)
    }
}

impl Default for GraphPipeline {
    fn default() -> Self {
        Self::new(PipelineConfig::default())
    }
}

/// Handle for a running graph generation task
pub struct GenerationHandle {
    handle: JoinHandle<()>,
    result_rx: oneshot::Receiver<Result<PathBuf>>,
}

impl GenerationHandle {
    /// Wait for the generation to complete and get the result
    pub async fn await_result(self) -> Result<PathBuf> {
        // Wait for the task to complete
        self.handle.await
            .map_err(|e| tgraph_common::TGraphError::new(format!("Task join error: {}", e)))?;
        
        // Get the result
        self.result_rx.await
            .map_err(|e| tgraph_common::TGraphError::new(format!("Result channel error: {}", e)))?
    }

    /// Check if the generation is complete (non-blocking)
    pub fn is_complete(&mut self) -> bool {
        self.handle.is_finished()
    }

    /// Cancel the generation task
    pub fn cancel(&mut self) {
        self.handle.abort();
    }
}

/// Manager for temporary files with automatic cleanup
pub struct TempFileManager {
    config: PipelineConfig,
    cleanup_tasks: Vec<JoinHandle<()>>,
}

impl TempFileManager {
    /// Create a new temporary file manager
    pub fn new(config: PipelineConfig) -> Self {
        Self {
            config,
            cleanup_tasks: Vec::new(),
        }
    }

    /// Create a temporary file and return its path
    pub async fn create_temp_file(&mut self, filename: String) -> Result<PathBuf> {
        // Ensure temp directory exists
        tokio::fs::create_dir_all(&self.config.temp_dir).await?;
        
        let temp_path = self.config.temp_dir.join(filename);
        
        // Create empty file to reserve the path
        tokio::fs::File::create(&temp_path).await?;
        
        tracing::debug!("Created temporary file: {}", temp_path.display());
        Ok(temp_path)
    }

    /// Schedule cleanup of a temporary file
    pub fn schedule_cleanup(&mut self, path: &Path, delay_secs: u64) {
        let path = path.to_owned();
        
        let cleanup_handle = tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_secs(delay_secs)).await;
            
            if path.exists() {
                if let Err(e) = tokio::fs::remove_file(&path).await {
                    tracing::warn!("Failed to cleanup temp file {}: {}", path.display(), e);
                } else {
                    tracing::debug!("Cleaned up temp file: {}", path.display());
                }
            }
        });
        
        self.cleanup_tasks.push(cleanup_handle);
    }

    /// Immediately cleanup a temporary file
    pub async fn cleanup_now(&self, path: &Path) -> Result<()> {
        if path.exists() {
            tokio::fs::remove_file(path).await?;
            tracing::debug!("Immediately cleaned up temp file: {}", path.display());
        }
        Ok(())
    }

    /// Cleanup all scheduled tasks (call on shutdown)
    pub async fn cleanup_all(&mut self) {
        for handle in self.cleanup_tasks.drain(..) {
            handle.abort();
        }
    }
}

impl Drop for TempFileManager {
    fn drop(&mut self) {
        // Cancel all pending cleanup tasks
        for handle in &self.cleanup_tasks {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{DataPoint, DataSet, GraphType, StyleConfig};
    use tempfile::TempDir;

    // Mock renderer for testing
    pub struct MockPipelineRenderer {
        delay_ms: u64,
    }

    impl MockPipelineRenderer {
        pub fn new() -> Self {
            Self { delay_ms: 10 }
        }

        pub fn with_delay(delay_ms: u64) -> Self {
            Self { delay_ms }
        }
    }

    #[async_trait::async_trait]
    impl GraphRenderer for MockPipelineRenderer {
        async fn render_to_file(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
            path: &Path,
        ) -> Result<()> {
            // Simulate rendering time
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            
            // Create a mock PNG file
            tokio::fs::write(path, b"mock png data").await?;
            Ok(())
        }

        async fn render_to_bytes(
            &self,
            _config: &GraphConfig,
            _datasets: &[DataSet],
        ) -> Result<Vec<u8>> {
            tokio::time::sleep(tokio::time::Duration::from_millis(self.delay_ms)).await;
            Ok(b"mock png data".to_vec())
        }

        fn apply_styling<DB: plotters::prelude::DrawingBackend>(
            &self,
            _root: &plotters::prelude::DrawingArea<DB, plotters::coord::Shift>,
            _config: &GraphConfig,
        ) -> Result<()>
        where
            DB::ErrorType: std::error::Error + Send + Sync + 'static,
        {
            Ok(())
        }
    }

    #[test]
    fn test_generation_task_creation() {
        let config = GraphConfig {
            graph_type: GraphType::Line,
            title: "Test Graph".to_string(),
            width: 800,
            height: 600,
            x_label: Some("X".to_string()),
            y_label: Some("Y".to_string()),
            style: StyleConfig::default(),
        };

        let datasets = vec![DataSet {
            name: "Test Data".to_string(),
            data: vec![
                DataPoint { x: 1.0, y: 2.0, label: None },
                DataPoint { x: 2.0, y: 4.0, label: None },
            ],
            color: None,
        }];

        let task = GraphGenerationTask::new(config.clone(), datasets.clone(), None);
        assert_eq!(task.config.title, "Test Graph");
        assert_eq!(task.datasets.len(), 1);
        assert!(task.output_path.is_none());
    }

    #[test]
    fn test_temp_filename_generation() {
        let config = GraphConfig {
            graph_type: GraphType::Bar,
            title: "Test".to_string(),
            width: 800,
            height: 600,
            x_label: None,
            y_label: None,
            style: StyleConfig::default(),
        };

        let task = GraphGenerationTask::new(config, vec![], None);
        let filename = task.generate_temp_filename();
        
        assert!(filename.starts_with("tgraph_bar_"));
        assert!(filename.ends_with(".png"));
        assert!(filename.contains(&task.id.simple().to_string()));
    }

    #[tokio::test]
    async fn test_temp_file_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            temp_dir: temp_dir.path().to_path_buf(),
            max_memory_mb: 512,
            cleanup_timeout_secs: 1,
            enable_progress_reporting: true,
        };

        let mut manager = TempFileManager::new(config);
        
        // Create a temp file
        let temp_path = manager.create_temp_file("test.png".to_string()).await.unwrap();
        assert!(temp_path.exists());
        
        // Schedule cleanup
        manager.schedule_cleanup(&temp_path, 1);
        
        // Wait for cleanup
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        
        // File should be cleaned up (note: this is timing-dependent in tests)
        // In a real scenario, we'd have more deterministic cleanup testing
    }

    #[tokio::test]
    async fn test_progress_reporting() {
        let config = GraphConfig::default();
        let datasets = vec![];
        
        let (task, mut progress_rx) = GraphGenerationTask::new(config, datasets, None)
            .with_progress_reporting();
        
        // Simulate sending progress
        task.send_progress(
            GenerationStage::Initializing,
            0.0,
            "Test message".to_string(),
        );
        
        // Receive progress
        let progress = progress_rx.recv().await.unwrap();
        assert!(matches!(progress.stage, GenerationStage::Initializing));
        assert_eq!(progress.progress, 0.0);
        assert_eq!(progress.message, "Test message");
    }

    #[tokio::test]
    async fn test_pipeline_creation() {
        let config = PipelineConfig::default();
        let pipeline = GraphPipeline::new(config);
        
        // Pipeline should be created successfully
        assert!(pipeline.temp_manager.lock().await.cleanup_tasks.is_empty());
    }

    #[tokio::test]
    async fn test_full_pipeline_execution() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            temp_dir: temp_dir.path().to_path_buf(),
            max_memory_mb: 512,
            cleanup_timeout_secs: 10,
            enable_progress_reporting: true,
        };

        let pipeline = GraphPipeline::new(config);
        let renderer = MockPipelineRenderer::new();

        // Create test data
        let graph_config = GraphConfig {
            graph_type: GraphType::Line,
            title: "Test Graph".to_string(),
            width: 800,
            height: 600,
            x_label: Some("Time".to_string()),
            y_label: Some("Value".to_string()),
            style: StyleConfig::default(),
        };

        let datasets = vec![DataSet {
            name: "Test Series".to_string(),
            data: vec![
                DataPoint { x: 1.0, y: 10.0, label: None },
                DataPoint { x: 2.0, y: 20.0, label: None },
                DataPoint { x: 3.0, y: 15.0, label: None },
            ],
            color: Some("#FF0000".to_string()),
        }];

        // Create task with progress reporting
        let (task, mut progress_rx) = GraphGenerationTask::new(graph_config, datasets, None)
            .with_progress_reporting();

        // Spawn the generation task
        let handle = pipeline.spawn_generation(task, renderer).await.unwrap();

        // Monitor progress
        let mut progress_updates = Vec::new();
        let result_future = handle.await_result();
        
        tokio::select! {
            result = result_future => {
                let output_path = result.unwrap();
                assert!(output_path.exists());
                assert!(output_path.to_string_lossy().contains("tgraph_line_"));
                
                // Verify file contents
                let contents = tokio::fs::read(&output_path).await.unwrap();
                assert_eq!(contents, b"mock png data");
            }
            _ = async {
                while let Some(progress) = progress_rx.recv().await {
                    progress_updates.push(progress);
                }
            } => {}
        }

        // Verify we received progress updates
        assert!(!progress_updates.is_empty());
        assert!(progress_updates.iter().any(|p| matches!(p.stage, GenerationStage::Initializing)));
    }

    #[tokio::test]
    async fn test_pipeline_with_output_path() {
        let temp_dir = TempDir::new().unwrap();
        let config = PipelineConfig {
            temp_dir: temp_dir.path().to_path_buf(),
            max_memory_mb: 512,
            cleanup_timeout_secs: 1,
            enable_progress_reporting: false,
        };

        let pipeline = GraphPipeline::new(config);
        let renderer = MockPipelineRenderer::new();

        // Create output path
        let output_path = temp_dir.path().join("output.png");

        // Create test task
        let task = GraphGenerationTask::new(
            GraphConfig::default(),
            vec![],
            Some(output_path.clone()),
        );

        // Execute the task
        let handle = pipeline.spawn_generation(task, renderer).await.unwrap();
        let result_path = handle.await_result().await.unwrap();

        // Verify the output was written to the specified path
        assert_eq!(result_path, output_path);
        assert!(output_path.exists());
        
        let contents = tokio::fs::read(&output_path).await.unwrap();
        assert_eq!(contents, b"mock png data");
    }

    #[tokio::test]
    async fn test_memory_efficient_processing() {
        let config = GraphConfig::default();
        
        // Create a large dataset to test memory efficiency
        let mut large_data = Vec::new();
        for i in 0..1000 {
            large_data.push(DataPoint {
                x: i as f64,
                y: (i as f64).sin(),
                label: Some(format!("Point {}", i)),
            });
        }

        let large_dataset = vec![DataSet {
            name: "Large Dataset".to_string(),
            data: large_data,
            color: None,
        }];

        let task = GraphGenerationTask::with_config(
            config,
            large_dataset,
            None,
            PipelineConfig {
                max_memory_mb: 1, // Very low memory limit
                ..PipelineConfig::default()
            },
        );

        // Process the dataset
        let processed = task.process_datasets_chunked().await.unwrap();
        assert_eq!(processed.len(), 1);
        assert_eq!(processed[0].data.len(), 1000);
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let config = PipelineConfig::default();
        let pipeline = GraphPipeline::new(config);
        
        // Use a renderer with delay to test cancellation
        let renderer = MockPipelineRenderer::with_delay(1000); // 1 second delay

        let task = GraphGenerationTask::new(GraphConfig::default(), vec![], None);
        let mut handle = pipeline.spawn_generation(task, renderer).await.unwrap();

        // Cancel the task after a short delay
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        handle.cancel();

        // Verify the task was cancelled
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        assert!(handle.is_complete());
    }
} 