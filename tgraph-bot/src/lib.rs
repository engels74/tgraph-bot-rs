//! TGraph Bot Library
//!
//! This library provides the core functionality for the TGraph Telegram bot,
//! including scheduling, task management, and task queue systems.

pub mod scheduler;
pub mod task_manager;
pub mod schedule_config;
pub mod task_queue;
pub mod discord;

// Re-export commonly used types
pub use scheduler::{SchedulerService, JobMetadata};
pub use task_manager::{TaskManager, TaskPriority, TaskMetadata};
pub use schedule_config::{ScheduleConfig, ScheduleConfigParser, TaskType};
pub use task_queue::{TaskQueue, QueuedTask, TaskResult, RetryStrategy, QueueStats};
