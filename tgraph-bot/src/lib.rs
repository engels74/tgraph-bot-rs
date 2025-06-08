//! TGraph Bot Library
//!
//! This library provides the core functionality for the TGraph Telegram bot,
//! including scheduling, task management, and task queue systems.

pub mod scheduler;
pub mod task_manager;
pub mod schedule_config;
pub mod task_queue;
pub mod metrics;
pub mod alerting;
pub mod persistence;
pub mod admin_api;
pub mod timezone_support;
pub mod monitoring_system;
pub mod discord;

// Re-export commonly used types
pub use scheduler::{SchedulerService, JobMetadata};
pub use task_manager::{TaskManager, TaskPriority, TaskMetadata};
pub use schedule_config::{ScheduleConfig, ScheduleConfigParser, TaskType};
pub use task_queue::{TaskQueue, QueuedTask, TaskResult, RetryStrategy, QueueStats};
pub use metrics::{MetricsCollector, TaskExecutionMetric, AggregatedMetrics};
pub use alerting::{AlertManager, Alert, AlertRule, AlertSeverity};
pub use persistence::PersistenceManager;
pub use admin_api::{AdminApiState, create_admin_api_router, start_admin_api_server};
pub use timezone_support::{TimezoneManager, TimezoneConfig, TimezoneInfo};
pub use monitoring_system::{MonitoringSystem, MonitoringConfig, MonitoringHealthStatus};
