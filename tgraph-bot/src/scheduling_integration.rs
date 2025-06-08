//! Integration layer for the scheduling system components

use std::sync::Arc;
use anyhow::Result;
use tracing::{info, error};

use crate::scheduler::SchedulerService;
use crate::task_manager::TaskManager;
use crate::task_queue::TaskQueue;
use crate::monitoring_system::{MonitoringSystem, MonitoringConfig};

/// Integrated scheduling system that coordinates all scheduling components
pub struct SchedulingSystem {
    /// Core scheduler service
    pub scheduler_service: Arc<SchedulerService>,
    /// Background task manager
    pub task_manager: Arc<TaskManager>,
    /// Task queue with priority and retry logic
    pub task_queue: Arc<TaskQueue>,
    /// Monitoring system for observability
    pub monitoring_system: Arc<MonitoringSystem>,
}

impl SchedulingSystem {
    /// Create a new scheduling system with all components
    pub async fn new() -> Result<Self> {
        info!("Initializing scheduling system");

        // Create core components
        let scheduler_service = Arc::new(SchedulerService::new().await?);
        let task_manager = Arc::new(TaskManager::with_defaults());

        // Create task queue with default settings
        let task_queue = Arc::new(TaskQueue::with_defaults(task_manager.clone())?);

        // Create monitoring system
        let monitoring_config = MonitoringConfig::default();
        let monitoring_system = Arc::new(MonitoringSystem::new(
            monitoring_config,
            scheduler_service.clone(),
            task_manager.clone(),
            task_queue.clone(),
        ).await?);

        info!("Scheduling system initialized successfully");

        Ok(Self {
            scheduler_service,
            task_manager,
            task_queue,
            monitoring_system,
        })
    }

    /// Start all scheduling system components
    pub async fn start(&self) -> Result<()> {
        info!("Starting scheduling system components");

        // Start the scheduler
        self.scheduler_service.start().await?;
        info!("Scheduler service started");

        // Start the task queue
        self.task_queue.start().await?;
        info!("Task queue started");

        // Start the monitoring system
        self.monitoring_system.start().await?;
        info!("Monitoring system started");

        info!("All scheduling system components started successfully");
        Ok(())
    }

    /// Gracefully shutdown all scheduling system components
    pub async fn shutdown(&self) -> Result<()> {
        info!("Shutting down scheduling system");

        // Shutdown in reverse order of startup
        if let Err(e) = self.task_queue.stop().await {
            error!("Error shutting down task queue: {}", e);
        }

        if let Err(e) = self.task_manager.shutdown().await {
            error!("Error shutting down task manager: {}", e);
        }

        if let Err(e) = self.scheduler_service.stop().await {
            error!("Error shutting down scheduler service: {}", e);
        }

        info!("Scheduling system shutdown complete");
        Ok(())
    }

    /// Get a reference to the scheduler service
    pub fn scheduler(&self) -> Arc<SchedulerService> {
        self.scheduler_service.clone()
    }

    /// Get a reference to the task manager
    pub fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }

    /// Get a reference to the task queue
    pub fn task_queue(&self) -> Arc<TaskQueue> {
        self.task_queue.clone()
    }

    /// Get a reference to the monitoring system
    pub fn monitoring(&self) -> Arc<MonitoringSystem> {
        self.monitoring_system.clone()
    }

    /// Check if the scheduling system is healthy
    pub async fn is_healthy(&self) -> bool {
        self.scheduler_service.is_running().await
            && self.task_queue.is_running().await
    }

    /// Get system status information
    pub async fn get_status(&self) -> SchedulingSystemStatus {
        let scheduler_running = self.scheduler_service.is_running().await;
        let task_queue_stats = self.task_queue.get_stats().await;
        let task_manager_count = self.task_manager.task_count().await;

        SchedulingSystemStatus {
            scheduler_running,
            pending_tasks: task_queue_stats.pending_tasks,
            running_tasks: task_queue_stats.running_tasks,
            completed_tasks: task_queue_stats.completed_tasks,
            failed_tasks: task_queue_stats.failed_tasks,
            background_tasks: task_manager_count,
        }
    }
}

/// Status information for the scheduling system
#[derive(Debug, Clone)]
pub struct SchedulingSystemStatus {
    /// Whether the scheduler is running
    pub scheduler_running: bool,
    /// Number of pending tasks in queue
    pub pending_tasks: usize,
    /// Number of currently running tasks
    pub running_tasks: usize,
    /// Number of completed tasks
    pub completed_tasks: usize,
    /// Number of failed tasks
    pub failed_tasks: usize,
    /// Number of background tasks
    pub background_tasks: usize,
}

impl SchedulingSystemStatus {
    /// Get a human-readable status summary
    pub fn summary(&self) -> String {
        format!(
            "Scheduler: {}, Queue: {} pending, {} running, {} completed, {} failed, Background: {} tasks",
            if self.scheduler_running { "Running" } else { "Stopped" },
            self.pending_tasks,
            self.running_tasks,
            self.completed_tasks,
            self.failed_tasks,
            self.background_tasks
        )
    }

    /// Check if the system is healthy
    pub fn is_healthy(&self) -> bool {
        self.scheduler_running
    }
}
