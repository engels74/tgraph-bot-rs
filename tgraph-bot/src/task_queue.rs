//! Task Queue with Priority and Retry Logic
//!
//! This module provides a priority-based task queue system with automatic retry capabilities,
//! persistence for crash recovery, and integration with the existing TaskManager.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BinaryHeap, HashMap};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::{mpsc, RwLock};
use tokio::time::timeout;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::task_manager::{TaskManager, TaskPriority};

/// Unique identifier for queued tasks
pub type QueuedTaskId = Uuid;

/// Result of task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskResult {
    /// Task completed successfully
    Success,
    /// Task failed with an error message
    Failed(String),
    /// Task was cancelled
    Cancelled,
    /// Task timed out
    TimedOut,
}

/// Retry strategy for failed tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RetryStrategy {
    /// No retries
    None,
    /// Fixed delay between retries
    FixedDelay {
        delay: Duration,
        max_attempts: u32,
    },
    /// Exponential backoff with jitter
    ExponentialBackoff {
        initial_delay: Duration,
        max_delay: Duration,
        multiplier: f64,
        max_attempts: u32,
    },
    /// Linear backoff
    LinearBackoff {
        initial_delay: Duration,
        increment: Duration,
        max_attempts: u32,
    },
}

impl Default for RetryStrategy {
    fn default() -> Self {
        RetryStrategy::ExponentialBackoff {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(300), // 5 minutes max
            multiplier: 2.0,
            max_attempts: 3,
        }
    }
}

/// Status of a queued task
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    /// Task is waiting to be executed
    Pending,
    /// Task is currently being executed
    Running,
    /// Task completed successfully
    Completed,
    /// Task failed and will be retried
    Failed,
    /// Task failed permanently (no more retries)
    FailedPermanently,
    /// Task was cancelled
    Cancelled,
}

/// A task in the queue with priority and retry information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueuedTask {
    /// Unique identifier for this queued task
    pub id: QueuedTaskId,
    /// Human-readable name for the task
    pub name: String,
    /// Priority level for execution order
    pub priority: TaskPriority,
    /// Optional description of what the task does
    pub description: Option<String>,
    /// Current status of the task
    pub status: TaskStatus,
    /// Retry strategy for this task
    pub retry_strategy: RetryStrategy,
    /// Number of attempts made so far
    pub attempts: u32,
    /// Timestamp when the task was created
    pub created_at: SystemTime,
    /// Timestamp when the task was last updated
    pub updated_at: SystemTime,
    /// Timestamp when the task should be executed (for delayed execution)
    pub execute_at: SystemTime,
    /// Last execution result
    pub last_result: Option<TaskResult>,
    /// Task-specific parameters (JSON serialized)
    pub parameters: serde_json::Value,
    /// Maximum execution timeout
    pub timeout: Option<Duration>,
}

impl QueuedTask {
    /// Create a new queued task
    pub fn new(
        name: String,
        priority: TaskPriority,
        description: Option<String>,
        parameters: serde_json::Value,
    ) -> Self {
        let now = SystemTime::now();
        Self {
            id: QueuedTaskId::new_v4(),
            name,
            priority,
            description,
            status: TaskStatus::Pending,
            retry_strategy: RetryStrategy::default(),
            attempts: 0,
            created_at: now,
            updated_at: now,
            execute_at: now,
            last_result: None,
            parameters,
            timeout: Some(Duration::from_secs(300)), // 5 minutes default
        }
    }

    /// Set the retry strategy for this task
    pub fn with_retry_strategy(mut self, strategy: RetryStrategy) -> Self {
        self.retry_strategy = strategy;
        self
    }

    /// Set the execution timeout for this task
    pub fn with_timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    /// Set when this task should be executed
    pub fn with_execute_at(mut self, execute_at: SystemTime) -> Self {
        self.execute_at = execute_at;
        self
    }

    /// Check if this task is ready to be executed
    pub fn is_ready(&self) -> bool {
        self.status == TaskStatus::Pending && self.execute_at <= SystemTime::now()
    }

    /// Check if this task can be retried
    pub fn can_retry(&self) -> bool {
        match &self.retry_strategy {
            RetryStrategy::None => false,
            RetryStrategy::FixedDelay { max_attempts, .. } => self.attempts < *max_attempts,
            RetryStrategy::ExponentialBackoff { max_attempts, .. } => self.attempts < *max_attempts,
            RetryStrategy::LinearBackoff { max_attempts, .. } => self.attempts < *max_attempts,
        }
    }

    /// Calculate the next retry delay
    pub fn next_retry_delay(&self) -> Option<Duration> {
        if !self.can_retry() {
            return None;
        }

        match &self.retry_strategy {
            RetryStrategy::None => None,
            RetryStrategy::FixedDelay { delay, .. } => Some(*delay),
            RetryStrategy::ExponentialBackoff {
                initial_delay,
                max_delay,
                multiplier,
                ..
            } => {
                let delay = initial_delay.as_secs_f64() * multiplier.powi(self.attempts as i32);
                let delay = Duration::from_secs_f64(delay.min(max_delay.as_secs_f64()));
                Some(delay)
            }
            RetryStrategy::LinearBackoff {
                initial_delay,
                increment,
                ..
            } => {
                let delay = *initial_delay + *increment * self.attempts;
                Some(delay)
            }
        }
    }

    /// Mark this task as failed and schedule retry if possible
    pub fn mark_failed(&mut self, error: String) -> bool {
        self.attempts += 1;
        self.last_result = Some(TaskResult::Failed(error));
        self.updated_at = SystemTime::now();

        if self.can_retry() {
            if let Some(delay) = self.next_retry_delay() {
                self.execute_at = SystemTime::now() + delay;
                self.status = TaskStatus::Pending;
                true // Will be retried
            } else {
                self.status = TaskStatus::FailedPermanently;
                false // No more retries
            }
        } else {
            self.status = TaskStatus::FailedPermanently;
            false // No more retries
        }
    }

    /// Mark this task as completed successfully
    pub fn mark_completed(&mut self) {
        self.status = TaskStatus::Completed;
        self.last_result = Some(TaskResult::Success);
        self.updated_at = SystemTime::now();
    }

    /// Mark this task as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = TaskStatus::Cancelled;
        self.last_result = Some(TaskResult::Cancelled);
        self.updated_at = SystemTime::now();
    }
}

/// Wrapper for priority queue ordering
#[derive(Debug, Clone)]
struct PriorityQueueItem {
    task: QueuedTask,
    /// Priority score for ordering (higher = more priority)
    priority_score: u64,
}

impl PartialEq for PriorityQueueItem {
    fn eq(&self, other: &Self) -> bool {
        self.priority_score == other.priority_score
    }
}

impl Eq for PriorityQueueItem {}

impl PartialOrd for PriorityQueueItem {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PriorityQueueItem {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Higher priority score comes first
        self.priority_score.cmp(&other.priority_score)
    }
}

impl PriorityQueueItem {
    fn new(task: QueuedTask) -> Self {
        let priority_score = Self::calculate_priority_score(&task);
        Self { task, priority_score }
    }

    fn calculate_priority_score(task: &QueuedTask) -> u64 {
        // Base priority from TaskPriority enum
        let base_priority = match task.priority {
            TaskPriority::Critical => 1000,
            TaskPriority::High => 750,
            TaskPriority::Normal => 500,
            TaskPriority::Low => 250,
        };

        // Adjust for execution time (earlier execution gets higher priority)
        let now = SystemTime::now();
        let time_adjustment = if task.execute_at <= now {
            // Ready to execute now - boost priority
            100
        } else {
            // Future execution - reduce priority based on delay
            let delay = task.execute_at.duration_since(now).unwrap_or_default();
            let delay_penalty = (delay.as_secs().min(3600) / 60) as u64; // Max 1 hour penalty
            100_u64.saturating_sub(delay_penalty)
        };

        // Adjust for retry attempts (more attempts = lower priority)
        let retry_penalty = task.attempts as u64 * 10;

        base_priority + time_adjustment - retry_penalty
    }
}

/// Task execution function type
pub type TaskExecutor = Arc<dyn Fn(serde_json::Value) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> + Send + Sync>;

/// Configuration for the task queue
#[derive(Debug, Clone)]
pub struct TaskQueueConfig {
    /// Maximum number of concurrent workers
    pub max_workers: usize,
    /// How often to check for ready tasks (in milliseconds)
    pub poll_interval: Duration,
    /// Database path for persistence
    pub db_path: String,
    /// Maximum queue size (0 = unlimited)
    pub max_queue_size: usize,
}

impl Default for TaskQueueConfig {
    fn default() -> Self {
        Self {
            max_workers: 4,
            poll_interval: Duration::from_millis(1000),
            db_path: "task_queue.db".to_string(),
            max_queue_size: 10000,
        }
    }
}

/// Task queue with priority and retry logic
pub struct TaskQueue {
    /// Configuration for the queue
    config: TaskQueueConfig,
    /// Priority queue for pending tasks
    pending_queue: Arc<RwLock<BinaryHeap<PriorityQueueItem>>>,
    /// Map of all tasks by ID for quick lookup
    tasks: Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
    /// Task manager for executing background tasks
    task_manager: Arc<TaskManager>,
    /// Persistent storage for tasks
    db: Arc<sled::Db>,
    /// Channel for sending commands to the queue processor
    command_tx: mpsc::UnboundedSender<QueueCommand>,
    /// Whether the queue is running
    is_running: Arc<RwLock<bool>>,
    /// Registered task executors by name
    executors: Arc<RwLock<HashMap<String, TaskExecutor>>>,
}

/// Commands that can be sent to the queue processor
#[derive(Debug)]
enum QueueCommand {
    /// Add a new task to the queue
    AddTask(QueuedTask),
    /// Cancel a task by ID
    CancelTask(QueuedTaskId),
    /// Update task status
    UpdateTaskStatus(QueuedTaskId, TaskStatus),
    /// Shutdown the queue
    Shutdown,
}

impl TaskQueue {
    /// Create a new task queue with the given configuration
    pub fn new(config: TaskQueueConfig, task_manager: Arc<TaskManager>) -> Result<Self> {
        info!("Creating new task queue with config: {:?}", config);

        // Open persistent database
        let db = sled::open(&config.db_path)
            .with_context(|| format!("Failed to open task queue database at {}", config.db_path))?;

        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let queue = Self {
            config: config.clone(),
            pending_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            tasks: Arc::new(RwLock::new(HashMap::new())),
            task_manager,
            db: Arc::new(db),
            command_tx,
            is_running: Arc::new(RwLock::new(false)),
            executors: Arc::new(RwLock::new(HashMap::new())),
        };

        // Start the queue processor
        queue.start_processor(command_rx)?;

        Ok(queue)
    }

    /// Create a task queue with default configuration
    pub fn with_defaults(task_manager: Arc<TaskManager>) -> Result<Self> {
        Self::new(TaskQueueConfig::default(), task_manager)
    }

    /// Register a task executor for a specific task type
    pub async fn register_executor<F, Fut>(&self, task_type: String, executor: F) -> Result<()>
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<()>> + Send + 'static,
    {
        info!("Registering executor for task type: {}", task_type);

        let boxed_executor: TaskExecutor = Arc::new(move |params| {
            Box::pin(executor(params))
        });

        let mut executors = self.executors.write().await;
        executors.insert(task_type, boxed_executor);

        Ok(())
    }

    /// Start the task queue processing
    pub async fn start(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if *is_running {
            warn!("Task queue is already running");
            return Ok(());
        }

        info!("Starting task queue");

        // Load persisted tasks from database
        self.load_persisted_tasks().await?;

        *is_running = true;
        info!("Task queue started successfully");

        Ok(())
    }

    /// Stop the task queue processing
    pub async fn stop(&self) -> Result<()> {
        let mut is_running = self.is_running.write().await;
        if !*is_running {
            warn!("Task queue is not running");
            return Ok(());
        }

        info!("Stopping task queue");

        // Send shutdown command
        self.command_tx.send(QueueCommand::Shutdown)
            .context("Failed to send shutdown command")?;

        // Persist all current tasks
        self.persist_all_tasks().await?;

        *is_running = false;
        info!("Task queue stopped successfully");

        Ok(())
    }

    /// Add a new task to the queue
    pub async fn enqueue_task(&self, mut task: QueuedTask) -> Result<QueuedTaskId> {
        let task_id = task.id;
        info!("Enqueueing task: {} (ID: {:?}, Priority: {:?})", task.name, task_id, task.priority);

        // Check queue size limit
        if self.config.max_queue_size > 0 {
            let current_size = self.tasks.read().await.len();
            if current_size >= self.config.max_queue_size {
                return Err(anyhow::anyhow!("Task queue is full (max size: {})", self.config.max_queue_size));
            }
        }

        // Update timestamps
        task.updated_at = SystemTime::now();

        // Persist the task
        self.persist_task(&task).await?;

        // Send command to add task
        self.command_tx.send(QueueCommand::AddTask(task))
            .context("Failed to send add task command")?;

        Ok(task_id)
    }

    /// Cancel a task by ID
    pub async fn cancel_task(&self, task_id: QueuedTaskId) -> Result<()> {
        info!("Cancelling task with ID: {:?}", task_id);

        self.command_tx.send(QueueCommand::CancelTask(task_id))
            .context("Failed to send cancel task command")?;

        Ok(())
    }

    /// Get a task by ID
    pub async fn get_task(&self, task_id: QueuedTaskId) -> Option<QueuedTask> {
        let tasks = self.tasks.read().await;
        tasks.get(&task_id).cloned()
    }

    /// List all tasks with optional status filter
    pub async fn list_tasks(&self, status_filter: Option<TaskStatus>) -> Vec<QueuedTask> {
        let tasks = self.tasks.read().await;
        tasks.values()
            .filter(|task| status_filter.as_ref().map_or(true, |status| &task.status == status))
            .cloned()
            .collect()
    }

    /// Get queue statistics
    pub async fn get_stats(&self) -> QueueStats {
        let tasks = self.tasks.read().await;
        let pending_queue = self.pending_queue.read().await;

        let mut stats = QueueStats::default();
        stats.total_tasks = tasks.len();
        stats.pending_tasks = pending_queue.len();

        for task in tasks.values() {
            match task.status {
                TaskStatus::Pending => stats.pending_tasks += 1,
                TaskStatus::Running => stats.running_tasks += 1,
                TaskStatus::Completed => stats.completed_tasks += 1,
                TaskStatus::Failed => stats.failed_tasks += 1,
                TaskStatus::FailedPermanently => stats.failed_permanently_tasks += 1,
                TaskStatus::Cancelled => stats.cancelled_tasks += 1,
            }
        }

        stats
    }

    /// Check if the queue is running
    pub async fn is_running(&self) -> bool {
        *self.is_running.read().await
    }

    /// Load persisted tasks from database
    async fn load_persisted_tasks(&self) -> Result<()> {
        info!("Loading persisted tasks from database");

        let mut loaded_count = 0;
        for result in self.db.iter() {
            let (key, value) = result.context("Failed to read task from database")?;

            let _task_id_str = String::from_utf8(key.to_vec())
                .context("Invalid task ID in database")?;

            let task: QueuedTask = serde_json::from_slice(&value)
                .context("Failed to deserialize task from database")?;

            // Only load pending and failed tasks (others are completed/cancelled)
            if matches!(task.status, TaskStatus::Pending | TaskStatus::Failed) {
                let mut tasks = self.tasks.write().await;
                let mut pending_queue = self.pending_queue.write().await;

                tasks.insert(task.id, task.clone());
                if task.is_ready() {
                    pending_queue.push(PriorityQueueItem::new(task));
                }
                loaded_count += 1;
            }
        }

        info!("Loaded {} persisted tasks from database", loaded_count);
        Ok(())
    }

    /// Persist a single task to database
    async fn persist_task(&self, task: &QueuedTask) -> Result<()> {
        let key = task.id.to_string();
        let value = serde_json::to_vec(task)
            .context("Failed to serialize task for persistence")?;

        self.db.insert(key.as_bytes(), value)
            .context("Failed to persist task to database")?;

        Ok(())
    }

    /// Persist all current tasks to database
    async fn persist_all_tasks(&self) -> Result<()> {
        info!("Persisting all tasks to database");

        let tasks = self.tasks.read().await;
        for task in tasks.values() {
            self.persist_task(task).await?;
        }

        self.db.flush_async().await
            .context("Failed to flush database")?;

        info!("Successfully persisted {} tasks", tasks.len());
        Ok(())
    }

    /// Remove a task from persistence
    async fn remove_persisted_task(&self, task_id: QueuedTaskId) -> Result<()> {
        let key = task_id.to_string();
        self.db.remove(key.as_bytes())
            .context("Failed to remove task from database")?;
        Ok(())
    }

    /// Start the queue processor task
    fn start_processor(&self, mut command_rx: mpsc::UnboundedReceiver<QueueCommand>) -> Result<()> {
        let pending_queue = self.pending_queue.clone();
        let tasks = self.tasks.clone();
        let task_manager = self.task_manager.clone();
        let executors = self.executors.clone();
        let db = self.db.clone();
        let config = self.config.clone();

        // Spawn the main processor task
        tokio::spawn(async move {
            info!("Task queue processor started");

            let mut shutdown_requested = false;
            let mut poll_interval = tokio::time::interval(config.poll_interval);

            loop {
                tokio::select! {
                    // Handle commands
                    command = command_rx.recv() => {
                        match command {
                            Some(QueueCommand::AddTask(task)) => {
                                Self::handle_add_task(&pending_queue, &tasks, task).await;
                            }
                            Some(QueueCommand::CancelTask(task_id)) => {
                                Self::handle_cancel_task(&tasks, &db, task_id).await;
                            }
                            Some(QueueCommand::UpdateTaskStatus(task_id, status)) => {
                                Self::handle_update_task_status(&tasks, &db, task_id, status).await;
                            }
                            Some(QueueCommand::Shutdown) => {
                                info!("Shutdown command received");
                                shutdown_requested = true;
                                break;
                            }
                            None => {
                                warn!("Command channel closed");
                                break;
                            }
                        }
                    }

                    // Process ready tasks
                    _ = poll_interval.tick() => {
                        if shutdown_requested {
                            break;
                        }
                        Self::process_ready_tasks(&pending_queue, &tasks, &task_manager, &executors, &db, &config).await;
                    }
                }
            }

            info!("Task queue processor stopped");
        });

        Ok(())
    }

    /// Handle adding a task to the queue
    async fn handle_add_task(
        pending_queue: &Arc<RwLock<BinaryHeap<PriorityQueueItem>>>,
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
        task: QueuedTask,
    ) {
        let task_id = task.id;
        debug!("Adding task to queue: {} (ID: {:?})", task.name, task_id);

        // Add to tasks map
        {
            let mut tasks_map = tasks.write().await;
            tasks_map.insert(task_id, task.clone());
        }

        // Add to pending queue if ready
        if task.is_ready() {
            let mut queue = pending_queue.write().await;
            queue.push(PriorityQueueItem::new(task));
            debug!("Task {:?} added to pending queue", task_id);
        } else {
            debug!("Task {:?} scheduled for future execution", task_id);
        }
    }

    /// Handle cancelling a task
    async fn handle_cancel_task(
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
        db: &Arc<sled::Db>,
        task_id: QueuedTaskId,
    ) {
        debug!("Cancelling task: {:?}", task_id);

        let mut tasks_map = tasks.write().await;
        if let Some(task) = tasks_map.get_mut(&task_id) {
            task.mark_cancelled();

            // Persist the updated task
            if let Ok(value) = serde_json::to_vec(&task) {
                let _ = db.insert(task_id.to_string().as_bytes(), value);
            }

            debug!("Task {:?} marked as cancelled", task_id);
        } else {
            warn!("Attempted to cancel non-existent task: {:?}", task_id);
        }
    }

    /// Handle updating task status
    async fn handle_update_task_status(
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
        db: &Arc<sled::Db>,
        task_id: QueuedTaskId,
        status: TaskStatus,
    ) {
        debug!("Updating task {:?} status to {:?}", task_id, status);

        let mut tasks_map = tasks.write().await;
        if let Some(task) = tasks_map.get_mut(&task_id) {
            task.status = status;
            task.updated_at = SystemTime::now();

            // Persist the updated task
            if let Ok(value) = serde_json::to_vec(&task) {
                let _ = db.insert(task_id.to_string().as_bytes(), value);
            }
        }
    }

    /// Process ready tasks from the queue
    async fn process_ready_tasks(
        pending_queue: &Arc<RwLock<BinaryHeap<PriorityQueueItem>>>,
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
        task_manager: &Arc<TaskManager>,
        executors: &Arc<RwLock<HashMap<String, TaskExecutor>>>,
        db: &Arc<sled::Db>,
        config: &TaskQueueConfig,
    ) {
        // Check for tasks that are now ready to execute
        Self::check_delayed_tasks(pending_queue, tasks).await;

        // Process tasks from the priority queue
        let mut tasks_to_execute = Vec::new();

        // Get up to max_workers tasks from the queue
        {
            let mut queue = pending_queue.write().await;
            let current_workers = task_manager.task_count().await;
            let available_workers = config.max_workers.saturating_sub(current_workers);

            for _ in 0..available_workers {
                if let Some(item) = queue.pop() {
                    tasks_to_execute.push(item.task);
                } else {
                    break;
                }
            }
        }

        // Execute the tasks
        for task in tasks_to_execute {
            Self::execute_task(task, tasks, task_manager, executors, db).await;
        }
    }

    /// Check for delayed tasks that are now ready
    async fn check_delayed_tasks(
        pending_queue: &Arc<RwLock<BinaryHeap<PriorityQueueItem>>>,
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
    ) {
        let now = SystemTime::now();
        let mut ready_tasks = Vec::new();

        // Find tasks that are now ready
        {
            let tasks_map = tasks.read().await;
            for task in tasks_map.values() {
                if task.status == TaskStatus::Pending && task.execute_at <= now {
                    ready_tasks.push(task.clone());
                }
            }
        }

        // Add ready tasks to the pending queue
        if !ready_tasks.is_empty() {
            let mut queue = pending_queue.write().await;
            for task in ready_tasks {
                queue.push(PriorityQueueItem::new(task));
            }
        }
    }

    /// Execute a single task
    async fn execute_task(
        mut task: QueuedTask,
        tasks: &Arc<RwLock<HashMap<QueuedTaskId, QueuedTask>>>,
        task_manager: &Arc<TaskManager>,
        executors: &Arc<RwLock<HashMap<String, TaskExecutor>>>,
        db: &Arc<sled::Db>,
    ) {
        let task_id = task.id;
        let task_name = task.name.clone();

        info!("Executing task: {} (ID: {:?}, Attempt: {})", task_name, task_id, task.attempts + 1);

        // Mark task as running
        task.status = TaskStatus::Running;
        task.attempts += 1;
        task.updated_at = SystemTime::now();

        // Update in memory and persist
        {
            let mut tasks_map = tasks.write().await;
            tasks_map.insert(task_id, task.clone());
        }
        if let Ok(value) = serde_json::to_vec(&task) {
            let _ = db.insert(task_id.to_string().as_bytes(), value);
        }

        // Get the executor for this task type
        let executor = {
            let executors_map = executors.read().await;
            // For now, we'll use a default executor name based on task name
            // In a real implementation, you'd have a task_type field
            executors_map.get("default").cloned()
        };

        let tasks_clone = tasks.clone();
        let db_clone = db.clone();
        let task_clone = task.clone();

        // Spawn the task execution
        let execution_result = task_manager.spawn_task(
            format!("queue_task_{}", task_name),
            task.priority,
            Some(format!("Executing queued task: {}", task_name)),
            move |_shutdown_rx| {
                let task = task_clone;
                let tasks = tasks_clone;
                let db = db_clone;
                let executor = executor;

                async move {
                    let task_id = task.id;
                    let mut final_task = task;

                    let execution_result = if let Some(executor) = executor {
                        // Execute with timeout if specified
                        let execution_future = executor(final_task.parameters.clone());

                        if let Some(timeout_duration) = final_task.timeout {
                            match timeout(timeout_duration, execution_future).await {
                                Ok(Ok(())) => Ok(()),
                                Ok(Err(e)) => Err(e),
                                Err(_) => Err(anyhow::anyhow!("Task execution timed out")),
                            }
                        } else {
                            execution_future.await
                        }
                    } else {
                        Err(anyhow::anyhow!("No executor found for task"))
                    };

                    // Update task based on execution result
                    match execution_result {
                        Ok(()) => {
                            info!("Task {} completed successfully", final_task.name);
                            final_task.mark_completed();
                        }
                        Err(e) => {
                            let error_msg = e.to_string();
                            warn!("Task {} failed: {}", final_task.name, error_msg);

                            let will_retry = final_task.mark_failed(error_msg);
                            if will_retry {
                                info!("Task {} will be retried (attempt {} of max {})",
                                     final_task.name, final_task.attempts,
                                     Self::get_max_attempts(&final_task.retry_strategy));
                            } else {
                                error!("Task {} failed permanently after {} attempts",
                                      final_task.name, final_task.attempts);
                            }
                        }
                    }

                    // Update in memory and persist
                    {
                        let mut tasks_map = tasks.write().await;
                        tasks_map.insert(task_id, final_task.clone());
                    }
                    if let Ok(value) = serde_json::to_vec(&final_task) {
                        let _ = db.insert(task_id.to_string().as_bytes(), value);
                    }
                }
            }
        ).await;

        if let Err(e) = execution_result {
            error!("Failed to spawn task execution for {}: {}", task_name, e);

            // Mark task as failed
            let mut failed_task = task;
            failed_task.mark_failed(format!("Failed to spawn execution: {}", e));

            let mut tasks_map = tasks.write().await;
            tasks_map.insert(task_id, failed_task.clone());

            if let Ok(value) = serde_json::to_vec(&failed_task) {
                let _ = db.insert(task_id.to_string().as_bytes(), value);
            }
        }
    }

    /// Get maximum attempts from retry strategy
    fn get_max_attempts(strategy: &RetryStrategy) -> u32 {
        match strategy {
            RetryStrategy::None => 1,
            RetryStrategy::FixedDelay { max_attempts, .. } => *max_attempts,
            RetryStrategy::ExponentialBackoff { max_attempts, .. } => *max_attempts,
            RetryStrategy::LinearBackoff { max_attempts, .. } => *max_attempts,
        }
    }
}

/// Statistics about the task queue
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QueueStats {
    /// Total number of tasks in the system
    pub total_tasks: usize,
    /// Number of tasks waiting to be executed
    pub pending_tasks: usize,
    /// Number of tasks currently being executed
    pub running_tasks: usize,
    /// Number of tasks that completed successfully
    pub completed_tasks: usize,
    /// Number of tasks that failed but can be retried
    pub failed_tasks: usize,
    /// Number of tasks that failed permanently
    pub failed_permanently_tasks: usize,
    /// Number of tasks that were cancelled
    pub cancelled_tasks: usize,
}

impl QueueStats {
    /// Get the total number of active tasks (pending + running)
    pub fn active_tasks(&self) -> usize {
        self.pending_tasks + self.running_tasks
    }

    /// Get the total number of finished tasks (completed + failed permanently + cancelled)
    pub fn finished_tasks(&self) -> usize {
        self.completed_tasks + self.failed_permanently_tasks + self.cancelled_tasks
    }

    /// Calculate success rate as a percentage
    pub fn success_rate(&self) -> f64 {
        let finished = self.finished_tasks();
        if finished == 0 {
            0.0
        } else {
            (self.completed_tasks as f64 / finished as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task_manager::TaskManager;

    use tempfile::TempDir;
    use tokio::time::sleep;

    async fn create_test_queue() -> (TaskQueue, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test_queue.db").to_string_lossy().to_string();

        let config = TaskQueueConfig {
            max_workers: 2,
            poll_interval: Duration::from_millis(100),
            db_path,
            max_queue_size: 100,
        };

        let task_manager = Arc::new(TaskManager::with_defaults());
        let queue = TaskQueue::new(config, task_manager).unwrap();

        (queue, temp_dir)
    }

    #[tokio::test]
    async fn test_queue_creation_and_basic_operations() {
        let (queue, _temp_dir) = create_test_queue().await;

        // Test initial state
        assert!(!queue.is_running().await);
        assert_eq!(queue.list_tasks(None).await.len(), 0);

        // Start the queue
        queue.start().await.unwrap();
        assert!(queue.is_running().await);

        // Stop the queue
        queue.stop().await.unwrap();
        assert!(!queue.is_running().await);
    }

    #[tokio::test]
    async fn test_task_enqueueing_and_retrieval() {
        let (queue, _temp_dir) = create_test_queue().await;
        queue.start().await.unwrap();

        // Create a test task
        let task = QueuedTask::new(
            "test_task".to_string(),
            TaskPriority::Normal,
            Some("Test task description".to_string()),
            serde_json::json!({"key": "value"}),
        );

        let task_id = task.id;

        // Enqueue the task
        let returned_id = queue.enqueue_task(task).await.unwrap();
        assert_eq!(task_id, returned_id);

        // Give some time for the task to be processed
        sleep(Duration::from_millis(200)).await;

        // Retrieve the task
        let retrieved_task = queue.get_task(task_id).await;
        assert!(retrieved_task.is_some());

        let task = retrieved_task.unwrap();
        assert_eq!(task.name, "test_task");
        assert_eq!(task.priority, TaskPriority::Normal);
        assert_eq!(task.status, TaskStatus::Pending);

        queue.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_priority_ordering() {
        let (queue, _temp_dir) = create_test_queue().await;
        queue.start().await.unwrap();

        // Create tasks with different priorities
        let low_task = QueuedTask::new(
            "low_priority".to_string(),
            TaskPriority::Low,
            None,
            serde_json::json!({}),
        );

        let high_task = QueuedTask::new(
            "high_priority".to_string(),
            TaskPriority::High,
            None,
            serde_json::json!({}),
        );

        let critical_task = QueuedTask::new(
            "critical_priority".to_string(),
            TaskPriority::Critical,
            None,
            serde_json::json!({}),
        );

        // Enqueue in reverse priority order
        queue.enqueue_task(low_task).await.unwrap();
        queue.enqueue_task(high_task).await.unwrap();
        queue.enqueue_task(critical_task).await.unwrap();

        // Give some time for processing
        sleep(Duration::from_millis(200)).await;

        let tasks = queue.list_tasks(None).await;
        assert_eq!(tasks.len(), 3);

        queue.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_task_cancellation() {
        let (queue, _temp_dir) = create_test_queue().await;
        queue.start().await.unwrap();

        let task = QueuedTask::new(
            "cancellable_task".to_string(),
            TaskPriority::Normal,
            None,
            serde_json::json!({}),
        );

        let task_id = task.id;
        queue.enqueue_task(task).await.unwrap();

        // Cancel the task
        queue.cancel_task(task_id).await.unwrap();

        // Give some time for processing
        sleep(Duration::from_millis(300)).await;

        // Check that task is cancelled
        let cancelled_task = queue.get_task(task_id).await;
        assert!(cancelled_task.is_some());
        assert_eq!(cancelled_task.unwrap().status, TaskStatus::Cancelled);

        queue.stop().await.unwrap();
    }

    #[tokio::test]
    async fn test_retry_strategy() {
        let mut task = QueuedTask::new(
            "retry_test".to_string(),
            TaskPriority::Normal,
            None,
            serde_json::json!({}),
        );

        task.retry_strategy = RetryStrategy::FixedDelay {
            delay: Duration::from_millis(100),
            max_attempts: 3,
        };

        // Test retry logic
        assert!(task.can_retry());
        assert_eq!(task.attempts, 0);

        // First failure
        let will_retry = task.mark_failed("Test error".to_string());
        assert!(will_retry);
        assert_eq!(task.attempts, 1);
        assert_eq!(task.status, TaskStatus::Pending);

        // Second failure
        let will_retry = task.mark_failed("Test error 2".to_string());
        assert!(will_retry);
        assert_eq!(task.attempts, 2);

        // Third failure (should be permanent)
        let will_retry = task.mark_failed("Test error 3".to_string());
        assert!(!will_retry);
        assert_eq!(task.attempts, 3);
        assert_eq!(task.status, TaskStatus::FailedPermanently);
    }

    #[tokio::test]
    async fn test_queue_stats() {
        let (queue, _temp_dir) = create_test_queue().await;
        queue.start().await.unwrap();

        // Initial stats should be empty
        let stats = queue.get_stats().await;
        assert_eq!(stats.total_tasks, 0);
        assert_eq!(stats.pending_tasks, 0);

        // Add some tasks
        for i in 0..5 {
            let task = QueuedTask::new(
                format!("task_{}", i),
                TaskPriority::Normal,
                None,
                serde_json::json!({}),
            );
            queue.enqueue_task(task).await.unwrap();
        }

        // Give some time for processing
        sleep(Duration::from_millis(100)).await;

        let stats = queue.get_stats().await;
        assert_eq!(stats.total_tasks, 5);

        queue.stop().await.unwrap();
    }
}
