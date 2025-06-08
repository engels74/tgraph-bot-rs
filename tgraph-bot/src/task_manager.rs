//! Task Manager - Background task management with graceful shutdown
//! 
//! This module provides a task manager that can spawn and monitor background tasks
//! with proper lifecycle management, priority support, and graceful shutdown capabilities.

use std::sync::Arc;
use std::collections::HashMap;
use std::time::Duration;
use anyhow::Result;
use tokio::sync::{RwLock, broadcast, oneshot};
use tokio::task::JoinHandle;
use tokio::time::timeout;
use tracing::{info, warn, debug};
use uuid::Uuid;

/// Type alias for task identifiers
pub type TaskId = Uuid;

/// Priority levels for tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum TaskPriority {
    /// Low priority tasks - can be cancelled immediately on shutdown
    Low = 0,
    /// Normal priority tasks - given short grace period on shutdown
    Normal = 1,
    /// High priority tasks - given longer grace period on shutdown
    High = 2,
    /// Critical tasks - must complete before shutdown
    Critical = 3,
}

impl Default for TaskPriority {
    fn default() -> Self {
        TaskPriority::Normal
    }
}

/// Metadata for tracking background tasks
#[derive(Debug, Clone)]
pub struct TaskMetadata {
    /// Unique identifier for the task
    pub id: TaskId,
    /// Human-readable name for the task
    pub name: String,
    /// Priority level of the task
    pub priority: TaskPriority,
    /// Optional description of what the task does
    pub description: Option<String>,
    /// Whether the task is currently running
    pub is_running: bool,
    /// Timestamp when the task was created
    pub created_at: std::time::SystemTime,
}

/// Handle for a background task with metadata
struct TaskHandle {
    /// The actual tokio task handle
    handle: JoinHandle<()>,
    /// Metadata about the task
    metadata: TaskMetadata,
    /// Shutdown signal sender for this specific task
    shutdown_tx: Option<oneshot::Sender<()>>,
}

/// Background task manager with graceful shutdown capabilities
pub struct TaskManager {
    /// Map of task IDs to their handles and metadata
    tasks: Arc<RwLock<HashMap<TaskId, TaskHandle>>>,
    /// Global shutdown signal broadcaster
    shutdown_broadcast: broadcast::Sender<()>,
    /// Whether the task manager is shutting down
    is_shutting_down: Arc<RwLock<bool>>,
    /// Default timeout for task shutdown
    shutdown_timeout: Duration,
}

impl TaskManager {
    /// Create a new task manager instance
    /// 
    /// # Arguments
    /// 
    /// * `shutdown_timeout` - Default timeout for waiting for tasks to shutdown gracefully
    /// 
    /// # Returns
    /// 
    /// A new `TaskManager` instance
    pub fn new(shutdown_timeout: Duration) -> Self {
        let (shutdown_tx, _) = broadcast::channel(100);
        
        info!("Creating new task manager with shutdown timeout: {:?}", shutdown_timeout);
        
        TaskManager {
            tasks: Arc::new(RwLock::new(HashMap::new())),
            shutdown_broadcast: shutdown_tx,
            is_shutting_down: Arc::new(RwLock::new(false)),
            shutdown_timeout,
        }
    }
    
    /// Create a new task manager with default settings
    pub fn with_defaults() -> Self {
        Self::new(Duration::from_secs(30)) // 30 second default timeout
    }
    
    /// Spawn a new background task with the given priority
    /// 
    /// # Arguments
    /// 
    /// * `name` - Human-readable name for the task
    /// * `priority` - Priority level for shutdown handling
    /// * `description` - Optional description of the task
    /// * `task_fn` - Async function to execute as the background task
    /// 
    /// # Returns
    /// 
    /// The TaskId of the newly spawned task
    /// 
    /// # Errors
    /// 
    /// Returns an error if the task manager is shutting down
    pub async fn spawn_task<F, Fut>(
        &self,
        name: String,
        priority: TaskPriority,
        description: Option<String>,
        task_fn: F,
    ) -> Result<TaskId>
    where
        F: FnOnce(broadcast::Receiver<()>) -> Fut + Send + 'static,
        Fut: std::future::Future<Output = ()> + Send + 'static,
    {
        // Check if we're shutting down
        {
            let shutting_down = self.is_shutting_down.read().await;
            if *shutting_down {
                return Err(anyhow::anyhow!("Task manager is shutting down, cannot spawn new tasks"));
            }
        }
        
        let task_id = TaskId::new_v4();
        info!("Spawning new task: {} (ID: {:?}, Priority: {:?})", name, task_id, priority);

        // Create shutdown signal for this task
        let shutdown_rx = self.shutdown_broadcast.subscribe();
        let (task_shutdown_tx, task_shutdown_rx) = oneshot::channel();

        // Create task metadata
        let metadata = TaskMetadata {
            id: task_id,
            name: name.clone(),
            priority,
            description,
            is_running: true,
            created_at: std::time::SystemTime::now(),
        };

        // Clone name for use in the spawned task
        let task_name = name.clone();

        // Spawn the actual task
        let handle = tokio::spawn(async move {
            debug!("Task {} started execution", task_name);

            // Create a combined shutdown receiver that listens to both global and task-specific signals
            let global_shutdown = shutdown_rx;
            let mut task_shutdown = task_shutdown_rx;

            tokio::select! {
                _ = task_fn(global_shutdown) => {
                    debug!("Task {} completed normally", task_name);
                }
                _ = &mut task_shutdown => {
                    debug!("Task {} received task-specific shutdown signal", task_name);
                }
            }

            debug!("Task {} finished execution", task_name);
        });
        
        // Store the task handle and metadata
        let task_handle = TaskHandle {
            handle,
            metadata: metadata.clone(),
            shutdown_tx: Some(task_shutdown_tx),
        };
        
        {
            let mut tasks = self.tasks.write().await;
            tasks.insert(task_id, task_handle);
        }
        
        debug!("Task {} registered successfully", name);
        Ok(task_id)
    }
    
    /// Get metadata for a specific task
    /// 
    /// # Arguments
    /// 
    /// * `task_id` - The TaskId to look up
    /// 
    /// # Returns
    /// 
    /// The TaskMetadata if found, None otherwise
    pub async fn get_task(&self, task_id: TaskId) -> Option<TaskMetadata> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).map(|handle| handle.metadata.clone())
    }
    
    /// List all currently running tasks
    /// 
    /// # Returns
    /// 
    /// A vector of TaskMetadata for all running tasks
    pub async fn list_tasks(&self) -> Vec<TaskMetadata> {
        let tasks = self.tasks.read().await;
        tasks.values().map(|handle| handle.metadata.clone()).collect()
    }
    
    /// Get the number of currently running tasks
    pub async fn task_count(&self) -> usize {
        let tasks = self.tasks.read().await;
        tasks.len()
    }
    
    /// Cancel a specific task by ID
    ///
    /// # Arguments
    ///
    /// * `task_id` - The TaskId of the task to cancel
    ///
    /// # Errors
    ///
    /// Returns an error if the task doesn't exist
    pub async fn cancel_task(&self, task_id: TaskId) -> Result<()> {
        info!("Cancelling task with ID: {:?}", task_id);

        let task_handle = {
            let mut tasks = self.tasks.write().await;
            tasks.remove(&task_id)
                .ok_or_else(|| anyhow::anyhow!("Task with ID {:?} not found", task_id))?
        };

        let task_name = task_handle.metadata.name.clone();

        // Send task-specific shutdown signal if available
        if let Some(shutdown_tx) = task_handle.shutdown_tx {
            let _ = shutdown_tx.send(()); // Ignore if receiver is dropped
        }

        // Abort the task
        task_handle.handle.abort();

        info!("Successfully cancelled task: {}", task_name);
        Ok(())
    }

    /// Initiate graceful shutdown of all tasks
    ///
    /// This method will:
    /// 1. Mark the task manager as shutting down
    /// 2. Send shutdown signals to all tasks
    /// 3. Wait for tasks to complete based on their priority
    /// 4. Force-cancel any remaining tasks after timeout
    ///
    /// # Errors
    ///
    /// Returns an error if shutdown fails or times out
    pub async fn shutdown(&self) -> Result<()> {
        info!("Initiating graceful shutdown of task manager");

        // Mark as shutting down
        {
            let mut shutting_down = self.is_shutting_down.write().await;
            if *shutting_down {
                warn!("Task manager is already shutting down");
                return Ok(());
            }
            *shutting_down = true;
        }

        // Get all current tasks grouped by priority
        let tasks_by_priority = {
            let tasks = self.tasks.read().await;
            if tasks.is_empty() {
                info!("No tasks to shutdown");
                return Ok(());
            }

            let mut by_priority: HashMap<TaskPriority, Vec<TaskId>> = HashMap::new();
            for (task_id, handle) in tasks.iter() {
                by_priority
                    .entry(handle.metadata.priority)
                    .or_insert_with(Vec::new)
                    .push(*task_id);
            }
            by_priority
        };

        info!("Found {} priority levels with tasks to shutdown", tasks_by_priority.len());

        // Send global shutdown signal
        if let Err(e) = self.shutdown_broadcast.send(()) {
            warn!("Failed to send global shutdown signal: {}", e);
        } else {
            debug!("Global shutdown signal sent to all tasks");
        }

        // Shutdown tasks by priority (highest first)
        let mut priorities: Vec<_> = tasks_by_priority.keys().copied().collect();
        priorities.sort_by(|a, b| b.cmp(a)); // Reverse order (highest priority first)

        for priority in priorities {
            let task_ids = &tasks_by_priority[&priority];
            info!("Shutting down {} tasks with priority {:?}", task_ids.len(), priority);

            // Calculate timeout based on priority
            let priority_timeout = match priority {
                TaskPriority::Critical => self.shutdown_timeout * 2, // Double timeout for critical
                TaskPriority::High => self.shutdown_timeout,
                TaskPriority::Normal => self.shutdown_timeout / 2,
                TaskPriority::Low => Duration::from_secs(1), // Very short timeout for low priority
            };

            // Wait for tasks of this priority to complete
            let shutdown_result = timeout(
                priority_timeout,
                self.wait_for_tasks_completion(task_ids.clone())
            ).await;

            match shutdown_result {
                Ok(Ok(())) => {
                    info!("All {:?} priority tasks completed gracefully", priority);
                }
                Ok(Err(e)) => {
                    warn!("Error waiting for {:?} priority tasks: {}", priority, e);
                }
                Err(_) => {
                    warn!("Timeout waiting for {:?} priority tasks, force-cancelling remaining", priority);
                    self.force_cancel_tasks(task_ids.clone()).await;
                }
            }
        }

        // Final cleanup - ensure all tasks are removed
        {
            let mut tasks = self.tasks.write().await;
            let remaining_count = tasks.len();
            if remaining_count > 0 {
                warn!("Force-cancelling {} remaining tasks", remaining_count);
                for (_, handle) in tasks.drain() {
                    handle.handle.abort();
                }
            }
        }

        info!("Task manager shutdown completed");
        Ok(())
    }

    /// Wait for specific tasks to complete
    async fn wait_for_tasks_completion(&self, task_ids: Vec<TaskId>) -> Result<()> {
        let mut join_handles = Vec::new();

        // Collect join handles for the specified tasks
        {
            let mut tasks = self.tasks.write().await;
            for task_id in &task_ids {
                if let Some(mut task_handle) = tasks.remove(task_id) {
                    // Send individual shutdown signal
                    if let Some(shutdown_tx) = task_handle.shutdown_tx.take() {
                        let _ = shutdown_tx.send(());
                    }
                    join_handles.push((task_id, task_handle.handle, task_handle.metadata.name.clone()));
                }
            }
        }

        // Wait for all tasks to complete
        for (task_id, handle, name) in join_handles {
            match handle.await {
                Ok(()) => {
                    debug!("Task {} (ID: {:?}) completed successfully", name, task_id);
                }
                Err(e) if e.is_cancelled() => {
                    debug!("Task {} (ID: {:?}) was cancelled", name, task_id);
                }
                Err(e) => {
                    warn!("Task {} (ID: {:?}) failed: {}", name, task_id, e);
                }
            }
        }

        Ok(())
    }

    /// Force cancel specific tasks
    async fn force_cancel_tasks(&self, task_ids: Vec<TaskId>) {
        let mut tasks = self.tasks.write().await;
        for task_id in task_ids {
            if let Some(task_handle) = tasks.remove(&task_id) {
                warn!("Force-cancelling task: {}", task_handle.metadata.name);
                task_handle.handle.abort();
            }
        }
    }

    /// Check if the task manager is shutting down
    pub async fn is_shutting_down(&self) -> bool {
        *self.is_shutting_down.read().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::time::sleep;

    #[tokio::test]
    async fn test_task_manager_creation() {
        let manager = TaskManager::with_defaults();
        assert_eq!(manager.task_count().await, 0);
        assert!(!manager.is_shutting_down().await);
    }

    #[tokio::test]
    async fn test_spawn_and_track_task() {
        let manager = TaskManager::with_defaults();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let task_id = manager.spawn_task(
            "test_task".to_string(),
            TaskPriority::Normal,
            Some("Test task description".to_string()),
            move |mut _shutdown_rx| {
                let counter = counter_clone.clone();
                async move {
                    counter.fetch_add(1, Ordering::SeqCst);
                    sleep(Duration::from_millis(100)).await;
                    counter.fetch_add(1, Ordering::SeqCst);
                }
            }
        ).await.unwrap();

        // Verify task was added
        assert_eq!(manager.task_count().await, 1);

        let task_metadata = manager.get_task(task_id).await;
        assert!(task_metadata.is_some());

        let metadata = task_metadata.unwrap();
        assert_eq!(metadata.name, "test_task");
        assert_eq!(metadata.priority, TaskPriority::Normal);
        assert_eq!(metadata.description, Some("Test task description".to_string()));
        assert!(metadata.is_running);

        // Wait for task to complete
        sleep(Duration::from_millis(200)).await;

        // Task should have executed
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_task_priorities() {
        let manager = TaskManager::with_defaults();

        // Spawn tasks with different priorities
        let _low_task = manager.spawn_task(
            "low_priority".to_string(),
            TaskPriority::Low,
            None,
            |_| async { sleep(Duration::from_millis(50)).await }
        ).await.unwrap();

        let _high_task = manager.spawn_task(
            "high_priority".to_string(),
            TaskPriority::High,
            None,
            |_| async { sleep(Duration::from_millis(50)).await }
        ).await.unwrap();

        let _critical_task = manager.spawn_task(
            "critical_priority".to_string(),
            TaskPriority::Critical,
            None,
            |_| async { sleep(Duration::from_millis(50)).await }
        ).await.unwrap();

        assert_eq!(manager.task_count().await, 3);

        let tasks = manager.list_tasks().await;
        let priorities: Vec<_> = tasks.iter().map(|t| t.priority).collect();

        assert!(priorities.contains(&TaskPriority::Low));
        assert!(priorities.contains(&TaskPriority::High));
        assert!(priorities.contains(&TaskPriority::Critical));
    }

    #[tokio::test]
    async fn test_cancel_task() {
        let manager = TaskManager::with_defaults();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let task_id = manager.spawn_task(
            "cancellable_task".to_string(),
            TaskPriority::Normal,
            None,
            move |_| {
                let counter = counter_clone.clone();
                async move {
                    for _i in 0..10 {
                        counter.fetch_add(1, Ordering::SeqCst);
                        sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        ).await.unwrap();

        assert_eq!(manager.task_count().await, 1);

        // Let task run briefly
        sleep(Duration::from_millis(150)).await;

        // Cancel the task
        manager.cancel_task(task_id).await.unwrap();

        // Task should be removed from tracking
        assert_eq!(manager.task_count().await, 0);
        assert!(manager.get_task(task_id).await.is_none());

        // Counter should be less than 10 (task was cancelled)
        let final_count = counter.load(Ordering::SeqCst);
        assert!(final_count < 10, "Task should have been cancelled before completion");
    }

    #[tokio::test]
    async fn test_shutdown_signal_handling() {
        let manager = TaskManager::with_defaults();
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        let _task_id = manager.spawn_task(
            "shutdown_aware_task".to_string(),
            TaskPriority::Normal,
            None,
            move |mut shutdown_rx| {
                let counter = counter_clone.clone();
                async move {
                    loop {
                        tokio::select! {
                            _ = sleep(Duration::from_millis(10)) => {
                                counter.fetch_add(1, Ordering::SeqCst);
                            }
                            result = shutdown_rx.recv() => {
                                match result {
                                    Ok(_) => {
                                        counter.fetch_add(100, Ordering::SeqCst); // Mark shutdown received
                                        break;
                                    }
                                    Err(_) => {
                                        // Channel closed, also means shutdown
                                        counter.fetch_add(100, Ordering::SeqCst);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        ).await.unwrap();

        // Let task run briefly to ensure it starts
        sleep(Duration::from_millis(50)).await;

        // Initiate shutdown
        manager.shutdown().await.unwrap();

        // Give a moment for the shutdown to complete
        sleep(Duration::from_millis(50)).await;

        // Task should have received shutdown signal
        let final_count = counter.load(Ordering::SeqCst);
        assert!(final_count >= 100, "Task should have received shutdown signal, got: {}", final_count);

        // All tasks should be cleaned up
        assert_eq!(manager.task_count().await, 0);
        assert!(manager.is_shutting_down().await);
    }

    #[tokio::test]
    async fn test_graceful_shutdown_with_priorities() {
        let manager = TaskManager::new(Duration::from_millis(500));
        let execution_order = Arc::new(RwLock::new(Vec::new()));

        // Spawn tasks with different priorities
        for (name, priority) in [
            ("low1", TaskPriority::Low),
            ("normal1", TaskPriority::Normal),
            ("high1", TaskPriority::High),
            ("critical1", TaskPriority::Critical),
        ] {
            let order_clone = execution_order.clone();
            let name_clone = name.to_string();

            manager.spawn_task(
                name.to_string(),
                priority,
                None,
                move |mut shutdown_rx| {
                    let order = order_clone.clone();
                    let name = name_clone.clone();
                    async move {
                        tokio::select! {
                            _ = sleep(Duration::from_millis(200)) => {
                                let mut order = order.write().await;
                                order.push(format!("{}_completed", name));
                            }
                            _ = shutdown_rx.recv() => {
                                let mut order = order.write().await;
                                order.push(format!("{}_shutdown", name));
                            }
                        }
                    }
                }
            ).await.unwrap();
        }

        assert_eq!(manager.task_count().await, 4);

        // Let tasks start
        sleep(Duration::from_millis(50)).await;

        // Initiate shutdown
        manager.shutdown().await.unwrap();

        // Verify all tasks were handled
        assert_eq!(manager.task_count().await, 0);

        let order = execution_order.read().await;
        assert_eq!(order.len(), 4);

        // All tasks should have received shutdown signal
        assert!(order.iter().all(|entry| entry.contains("shutdown")));
    }

    #[tokio::test]
    async fn test_prevent_spawn_during_shutdown() {
        let manager = TaskManager::with_defaults();

        // Start shutdown
        manager.shutdown().await.unwrap();

        // Try to spawn a task after shutdown
        let result = manager.spawn_task(
            "post_shutdown_task".to_string(),
            TaskPriority::Normal,
            None,
            |_| async {}
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("shutting down"));
    }

    #[tokio::test]
    async fn test_timeout_and_force_cancel() {
        let manager = TaskManager::new(Duration::from_millis(100)); // Very short timeout
        let counter = Arc::new(AtomicUsize::new(0));
        let counter_clone = counter.clone();

        // Spawn a task that ignores shutdown signals
        let _task_id = manager.spawn_task(
            "stubborn_task".to_string(),
            TaskPriority::Normal,
            None,
            move |_shutdown_rx| {
                let counter = counter_clone.clone();
                async move {
                    // Ignore shutdown signal and keep running
                    for _ in 0..20 {
                        counter.fetch_add(1, Ordering::SeqCst);
                        sleep(Duration::from_millis(50)).await;
                    }
                }
            }
        ).await.unwrap();

        assert_eq!(manager.task_count().await, 1);

        // Let task start
        sleep(Duration::from_millis(50)).await;

        // Shutdown should timeout and force-cancel the task
        manager.shutdown().await.unwrap();

        // Task should be force-cancelled
        assert_eq!(manager.task_count().await, 0);

        // Counter should be less than 20 (task was force-cancelled)
        let final_count = counter.load(Ordering::SeqCst);
        assert!(final_count < 20, "Task should have been force-cancelled");
    }

    #[tokio::test]
    async fn test_double_shutdown() {
        let manager = TaskManager::with_defaults();

        // First shutdown
        manager.shutdown().await.unwrap();
        assert!(manager.is_shutting_down().await);

        // Second shutdown should not error
        manager.shutdown().await.unwrap();
        assert!(manager.is_shutting_down().await);
    }

    #[tokio::test]
    async fn test_cancel_nonexistent_task() {
        let manager = TaskManager::with_defaults();
        let fake_id = TaskId::new_v4();

        let result = manager.cancel_task(fake_id).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}
