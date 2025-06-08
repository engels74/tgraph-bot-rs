//! Metrics collection and monitoring for the scheduling system
//!
//! This module provides comprehensive metrics collection for task execution,
//! performance monitoring, and system health tracking.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use prometheus::{
    Counter, Gauge, Histogram, HistogramOpts, IntCounter, IntGauge, Opts, Registry,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Unique identifier for a metric entry
pub type MetricId = Uuid;

/// Task execution metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionMetric {
    /// Unique identifier for this metric entry
    pub id: MetricId,
    /// ID of the task that was executed
    pub task_id: String,
    /// Name of the task
    pub task_name: String,
    /// Type of task executed
    pub task_type: String,
    /// When the task started executing
    pub started_at: DateTime<Utc>,
    /// When the task finished executing (if completed)
    pub finished_at: Option<DateTime<Utc>>,
    /// Duration of task execution in milliseconds
    pub duration_ms: Option<u64>,
    /// Whether the task succeeded
    pub success: Option<bool>,
    /// Error message if the task failed
    pub error_message: Option<String>,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Memory usage during execution (if available)
    pub memory_usage_mb: Option<f64>,
    /// CPU usage during execution (if available)
    pub cpu_usage_percent: Option<f64>,
}

impl TaskExecutionMetric {
    /// Create a new task execution metric when a task starts
    pub fn new(task_id: String, task_name: String, task_type: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_id,
            task_name,
            task_type,
            started_at: Utc::now(),
            finished_at: None,
            duration_ms: None,
            success: None,
            error_message: None,
            retry_count: 0,
            memory_usage_mb: None,
            cpu_usage_percent: None,
        }
    }

    /// Mark the task as completed successfully
    pub fn mark_success(&mut self) {
        let now = Utc::now();
        self.finished_at = Some(now);
        self.success = Some(true);
        self.duration_ms = Some(
            (now - self.started_at)
                .num_milliseconds()
                .max(0) as u64
        );
    }

    /// Mark the task as failed with an error message
    pub fn mark_failure(&mut self, error_message: String) {
        let now = Utc::now();
        self.finished_at = Some(now);
        self.success = Some(false);
        self.error_message = Some(error_message);
        self.duration_ms = Some(
            (now - self.started_at)
                .num_milliseconds()
                .max(0) as u64
        );
    }

    /// Increment the retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Set resource usage metrics
    pub fn set_resource_usage(&mut self, memory_mb: Option<f64>, cpu_percent: Option<f64>) {
        self.memory_usage_mb = memory_mb;
        self.cpu_usage_percent = cpu_percent;
    }
}

/// Aggregated metrics for a specific task type or time period
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    /// Task type or identifier
    pub task_type: String,
    /// Time period for these metrics
    pub period_start: DateTime<Utc>,
    /// End of time period
    pub period_end: DateTime<Utc>,
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Success rate as a percentage
    pub success_rate: f64,
    /// Average execution duration in milliseconds
    pub avg_duration_ms: f64,
    /// Minimum execution duration in milliseconds
    pub min_duration_ms: u64,
    /// Maximum execution duration in milliseconds
    pub max_duration_ms: u64,
    /// Total retry attempts across all executions
    pub total_retries: u64,
    /// Average memory usage in MB
    pub avg_memory_usage_mb: Option<f64>,
    /// Average CPU usage percentage
    pub avg_cpu_usage_percent: Option<f64>,
}

/// Prometheus metrics collector for the scheduling system
pub struct PrometheusMetrics {
    /// Registry for all metrics
    registry: Registry,
    /// Counter for total task executions
    task_executions_total: IntCounter,
    /// Counter for successful task executions
    task_executions_success: IntCounter,
    /// Counter for failed task executions
    task_executions_failed: IntCounter,
    /// Histogram for task execution durations
    task_duration_histogram: Histogram,
    /// Gauge for currently running tasks
    tasks_running: IntGauge,
    /// Gauge for queued tasks
    tasks_queued: IntGauge,
    /// Counter for retry attempts
    task_retries_total: IntCounter,
}

impl PrometheusMetrics {
    /// Create a new Prometheus metrics collector
    pub fn new() -> Result<Self> {
        let registry = Registry::new();

        let task_executions_total = IntCounter::with_opts(
            Opts::new("task_executions_total", "Total number of task executions")
        )?;

        let task_executions_success = IntCounter::with_opts(
            Opts::new("task_executions_success_total", "Total number of successful task executions")
        )?;

        let task_executions_failed = IntCounter::with_opts(
            Opts::new("task_executions_failed_total", "Total number of failed task executions")
        )?;

        let task_duration_histogram = Histogram::with_opts(
            HistogramOpts::new("task_duration_seconds", "Task execution duration in seconds")
                .buckets(vec![0.1, 0.5, 1.0, 5.0, 10.0, 30.0, 60.0, 300.0, 600.0])
        )?;

        let tasks_running = IntGauge::with_opts(
            Opts::new("tasks_running", "Number of currently running tasks")
        )?;

        let tasks_queued = IntGauge::with_opts(
            Opts::new("tasks_queued", "Number of tasks in queue")
        )?;

        let task_retries_total = IntCounter::with_opts(
            Opts::new("task_retries_total", "Total number of task retry attempts")
        )?;

        // Register all metrics
        registry.register(Box::new(task_executions_total.clone()))?;
        registry.register(Box::new(task_executions_success.clone()))?;
        registry.register(Box::new(task_executions_failed.clone()))?;
        registry.register(Box::new(task_duration_histogram.clone()))?;
        registry.register(Box::new(tasks_running.clone()))?;
        registry.register(Box::new(tasks_queued.clone()))?;
        registry.register(Box::new(task_retries_total.clone()))?;

        Ok(Self {
            registry,
            task_executions_total,
            task_executions_success,
            task_executions_failed,
            task_duration_histogram,
            tasks_running,
            tasks_queued,
            task_retries_total,
        })
    }

    /// Record a task execution start
    pub fn record_task_start(&self) {
        self.task_executions_total.inc();
        self.tasks_running.inc();
    }

    /// Record a successful task completion
    pub fn record_task_success(&self, duration: Duration) {
        self.task_executions_success.inc();
        self.tasks_running.dec();
        self.task_duration_histogram.observe(duration.as_secs_f64());
    }

    /// Record a failed task execution
    pub fn record_task_failure(&self, duration: Duration) {
        self.task_executions_failed.inc();
        self.tasks_running.dec();
        self.task_duration_histogram.observe(duration.as_secs_f64());
    }

    /// Record a task retry attempt
    pub fn record_retry(&self) {
        self.task_retries_total.inc();
    }

    /// Update the number of queued tasks
    pub fn set_queued_tasks(&self, count: i64) {
        self.tasks_queued.set(count);
    }

    /// Get the Prometheus registry for exporting metrics
    pub fn registry(&self) -> &Registry {
        &self.registry
    }
}

/// Main metrics collector that aggregates and stores task execution metrics
pub struct MetricsCollector {
    /// In-memory storage for recent metrics
    metrics: Arc<RwLock<HashMap<MetricId, TaskExecutionMetric>>>,
    /// Prometheus metrics for real-time monitoring
    prometheus: PrometheusMetrics,
    /// Maximum number of metrics to keep in memory
    max_metrics_in_memory: usize,
}

impl MetricsCollector {
    /// Create a new metrics collector
    pub fn new(max_metrics_in_memory: usize) -> Result<Self> {
        info!("Creating new metrics collector with max {} metrics in memory", max_metrics_in_memory);
        
        let prometheus = PrometheusMetrics::new()
            .context("Failed to create Prometheus metrics")?;

        Ok(Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
            prometheus,
            max_metrics_in_memory,
        })
    }

    /// Create a metrics collector with default settings
    pub fn with_defaults() -> Result<Self> {
        Self::new(10000) // Keep 10k metrics in memory by default
    }

    /// Start tracking a new task execution
    pub async fn start_task_execution(&self, task_id: String, task_name: String, task_type: String) -> MetricId {
        let metric = TaskExecutionMetric::new(task_id, task_name, task_type);
        let metric_id = metric.id;

        // Store in memory
        let mut metrics = self.metrics.write().await;
        metrics.insert(metric_id, metric);

        // Clean up old metrics if we exceed the limit
        if metrics.len() > self.max_metrics_in_memory {
            self.cleanup_old_metrics(&mut metrics).await;
        }

        // Record in Prometheus
        self.prometheus.record_task_start();

        debug!("Started tracking task execution: {}", metric_id);
        metric_id
    }

    /// Mark a task execution as successful
    pub async fn mark_task_success(&self, metric_id: MetricId) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(&metric_id) {
            let start_time = SystemTime::from(metric.started_at);
            let duration = start_time.elapsed().unwrap_or(Duration::ZERO);

            metric.mark_success();
            self.prometheus.record_task_success(duration);

            debug!("Marked task {} as successful (duration: {:?})", metric_id, duration);
            Ok(())
        } else {
            warn!("Attempted to mark unknown task as successful: {}", metric_id);
            Err(anyhow::anyhow!("Task metric not found: {}", metric_id))
        }
    }

    /// Mark a task execution as failed
    pub async fn mark_task_failure(&self, metric_id: MetricId, error_message: String) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(&metric_id) {
            let start_time = SystemTime::from(metric.started_at);
            let duration = start_time.elapsed().unwrap_or(Duration::ZERO);

            metric.mark_failure(error_message.clone());
            self.prometheus.record_task_failure(duration);

            debug!("Marked task {} as failed (duration: {:?}): {}", metric_id, duration, error_message);
            Ok(())
        } else {
            warn!("Attempted to mark unknown task as failed: {}", metric_id);
            Err(anyhow::anyhow!("Task metric not found: {}", metric_id))
        }
    }

    /// Record a retry attempt for a task
    pub async fn record_retry(&self, metric_id: MetricId) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(&metric_id) {
            metric.increment_retry();
            self.prometheus.record_retry();

            debug!("Recorded retry for task {} (attempt #{})", metric_id, metric.retry_count);
            Ok(())
        } else {
            warn!("Attempted to record retry for unknown task: {}", metric_id);
            Err(anyhow::anyhow!("Task metric not found: {}", metric_id))
        }
    }

    /// Update resource usage for a task
    pub async fn update_resource_usage(&self, metric_id: MetricId, memory_mb: Option<f64>, cpu_percent: Option<f64>) -> Result<()> {
        let mut metrics = self.metrics.write().await;
        if let Some(metric) = metrics.get_mut(&metric_id) {
            metric.set_resource_usage(memory_mb, cpu_percent);
            debug!("Updated resource usage for task {}: memory={:?}MB, cpu={:?}%", metric_id, memory_mb, cpu_percent);
            Ok(())
        } else {
            warn!("Attempted to update resource usage for unknown task: {}", metric_id);
            Err(anyhow::anyhow!("Task metric not found: {}", metric_id))
        }
    }

    /// Update the number of queued tasks
    pub fn update_queued_tasks(&self, count: usize) {
        self.prometheus.set_queued_tasks(count as i64);
    }

    /// Get a specific task metric
    pub async fn get_metric(&self, metric_id: MetricId) -> Option<TaskExecutionMetric> {
        let metrics = self.metrics.read().await;
        metrics.get(&metric_id).cloned()
    }

    /// Get all metrics for a specific task type
    pub async fn get_metrics_by_task_type(&self, task_type: &str) -> Vec<TaskExecutionMetric> {
        let metrics = self.metrics.read().await;
        metrics.values()
            .filter(|m| m.task_type == task_type)
            .cloned()
            .collect()
    }

    /// Get recent metrics (last N entries)
    pub async fn get_recent_metrics(&self, limit: usize) -> Vec<TaskExecutionMetric> {
        let metrics = self.metrics.read().await;
        let mut sorted_metrics: Vec<_> = metrics.values().cloned().collect();
        sorted_metrics.sort_by(|a, b| b.started_at.cmp(&a.started_at));
        sorted_metrics.into_iter().take(limit).collect()
    }

    /// Calculate aggregated metrics for a task type within a time range
    pub async fn calculate_aggregated_metrics(
        &self,
        task_type: &str,
        start_time: DateTime<Utc>,
        end_time: DateTime<Utc>,
    ) -> AggregatedMetrics {
        let metrics = self.metrics.read().await;
        let filtered_metrics: Vec<_> = metrics.values()
            .filter(|m| {
                m.task_type == task_type &&
                m.started_at >= start_time &&
                m.started_at <= end_time &&
                m.finished_at.is_some()
            })
            .collect();

        let total_executions = filtered_metrics.len() as u64;
        let successful_executions = filtered_metrics.iter()
            .filter(|m| m.success == Some(true))
            .count() as u64;
        let failed_executions = total_executions - successful_executions;

        let success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let durations: Vec<u64> = filtered_metrics.iter()
            .filter_map(|m| m.duration_ms)
            .collect();

        let (avg_duration_ms, min_duration_ms, max_duration_ms) = if !durations.is_empty() {
            let avg = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
            let min = *durations.iter().min().unwrap_or(&0);
            let max = *durations.iter().max().unwrap_or(&0);
            (avg, min, max)
        } else {
            (0.0, 0, 0)
        };

        let total_retries = filtered_metrics.iter()
            .map(|m| m.retry_count as u64)
            .sum();

        let memory_usages: Vec<f64> = filtered_metrics.iter()
            .filter_map(|m| m.memory_usage_mb)
            .collect();
        let avg_memory_usage_mb = if !memory_usages.is_empty() {
            Some(memory_usages.iter().sum::<f64>() / memory_usages.len() as f64)
        } else {
            None
        };

        let cpu_usages: Vec<f64> = filtered_metrics.iter()
            .filter_map(|m| m.cpu_usage_percent)
            .collect();
        let avg_cpu_usage_percent = if !cpu_usages.is_empty() {
            Some(cpu_usages.iter().sum::<f64>() / cpu_usages.len() as f64)
        } else {
            None
        };

        AggregatedMetrics {
            task_type: task_type.to_string(),
            period_start: start_time,
            period_end: end_time,
            total_executions,
            successful_executions,
            failed_executions,
            success_rate,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            total_retries,
            avg_memory_usage_mb,
            avg_cpu_usage_percent,
        }
    }

    /// Get the Prometheus registry for metrics export
    pub fn prometheus_registry(&self) -> &prometheus::Registry {
        self.prometheus.registry()
    }

    /// Clean up old metrics to prevent memory bloat
    async fn cleanup_old_metrics(&self, metrics: &mut HashMap<MetricId, TaskExecutionMetric>) {
        let target_size = self.max_metrics_in_memory * 3 / 4; // Remove 25% when cleanup is triggered

        if metrics.len() <= target_size {
            return;
        }

        // Sort by start time and keep the most recent ones
        let mut sorted_metrics: Vec<_> = metrics.iter().collect();
        sorted_metrics.sort_by(|a, b| b.1.started_at.cmp(&a.1.started_at));

        // Keep only the most recent metrics
        let to_keep: std::collections::HashSet<_> = sorted_metrics
            .into_iter()
            .take(target_size)
            .map(|(id, _)| *id)
            .collect();

        metrics.retain(|id, _| to_keep.contains(id));

        info!("Cleaned up old metrics, kept {} out of {} metrics", to_keep.len(), metrics.len() + to_keep.len());
    }
}
