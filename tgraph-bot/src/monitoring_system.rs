//! Integrated monitoring, alerting, and persistence system
//!
//! This module provides a unified interface for the complete monitoring system,
//! integrating metrics collection, alerting, persistence, and admin API.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::interval;
use tracing::{error, info};

use crate::admin_api::{AdminApiState, start_admin_api_server};
use crate::alerting::{AlertManager, AlertRule, AlertCondition, AlertSeverity};
use crate::metrics::MetricsCollector;
use crate::persistence::PersistenceManager;
use crate::scheduler::SchedulerService;
use crate::task_manager::TaskManager;
use crate::task_queue::TaskQueue;
use crate::timezone_support::TimezoneManager;

/// Configuration for the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    /// Database path for persistence
    pub database_path: String,
    /// Admin API bind address
    pub admin_api_address: String,
    /// Maximum metrics to keep in memory
    pub max_metrics_in_memory: usize,
    /// How often to evaluate alert rules (in seconds)
    pub alert_evaluation_interval: u64,
    /// How often to persist metrics to database (in seconds)
    pub metrics_persistence_interval: u64,
    /// Data retention period in days
    pub data_retention_days: u32,
    /// How often to run cleanup (in hours)
    pub cleanup_interval_hours: u64,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            database_path: "./data/monitoring.db".to_string(),
            admin_api_address: "127.0.0.1:8080".to_string(),
            max_metrics_in_memory: 10000,
            alert_evaluation_interval: 60, // 1 minute
            metrics_persistence_interval: 300, // 5 minutes
            data_retention_days: 30,
            cleanup_interval_hours: 24, // Daily cleanup
        }
    }
}

/// Integrated monitoring system that coordinates all components
pub struct MonitoringSystem {
    /// Configuration for the monitoring system
    config: MonitoringConfig,
    /// Metrics collector for task execution data
    metrics_collector: Arc<MetricsCollector>,
    /// Alert manager for monitoring and alerting
    alert_manager: Arc<AlertManager>,
    /// Persistence manager for database operations
    persistence_manager: Arc<PersistenceManager>,
    /// Timezone manager for global timezone support
    timezone_manager: Arc<TimezoneManager>,
    /// Scheduler service reference
    scheduler_service: Arc<SchedulerService>,
    /// Task manager reference
    task_manager: Arc<TaskManager>,
    /// Task queue reference
    task_queue: Arc<TaskQueue>,
}

impl MonitoringSystem {
    /// Create a new monitoring system
    pub async fn new(
        config: MonitoringConfig,
        scheduler_service: Arc<SchedulerService>,
        task_manager: Arc<TaskManager>,
        task_queue: Arc<TaskQueue>,
    ) -> Result<Self> {
        info!("Initializing monitoring system");

        // Initialize persistence manager
        let persistence_manager = Arc::new(
            PersistenceManager::new(&config.database_path)
                .await
                .context("Failed to initialize persistence manager")?
        );

        // Initialize metrics collector
        let metrics_collector = Arc::new(
            MetricsCollector::new(config.max_metrics_in_memory)
                .context("Failed to initialize metrics collector")?
        );

        // Initialize alert manager
        let alert_manager = Arc::new(AlertManager::new(metrics_collector.clone()));

        // Initialize timezone manager
        let timezone_manager = Arc::new(TimezoneManager::new());

        // Load existing alert rules from database
        let existing_rules = persistence_manager.load_alert_rules().await
            .context("Failed to load existing alert rules")?;
        
        for rule in existing_rules {
            alert_manager.add_rule(rule).await
                .context("Failed to restore alert rule")?;
        }

        info!("Monitoring system initialized successfully");

        Ok(Self {
            config,
            metrics_collector,
            alert_manager,
            persistence_manager,
            timezone_manager,
            scheduler_service,
            task_manager,
            task_queue,
        })
    }

    /// Start the monitoring system with all background tasks
    pub async fn start(&self) -> Result<()> {
        info!("Starting monitoring system");

        // Start alert evaluation loop
        self.start_alert_evaluation_loop().await;

        // Start metrics persistence loop
        self.start_metrics_persistence_loop().await;

        // Start cleanup loop
        self.start_cleanup_loop().await;

        // Start admin API server
        self.start_admin_api().await?;

        // Set up default alert rules if none exist
        self.setup_default_alert_rules().await?;

        info!("Monitoring system started successfully");
        Ok(())
    }

    /// Start the alert evaluation background task
    async fn start_alert_evaluation_loop(&self) {
        let alert_manager = self.alert_manager.clone();
        let persistence_manager = self.persistence_manager.clone();
        let interval_duration = Duration::from_secs(self.config.alert_evaluation_interval);

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                match alert_manager.evaluate_rules().await {
                    Ok(new_alerts) => {
                        if !new_alerts.is_empty() {
                            info!("Generated {} new alerts", new_alerts.len());
                            
                            // Persist new alerts to database
                            for alert in &new_alerts {
                                if let Err(e) = persistence_manager.save_alert(alert).await {
                                    error!("Failed to persist alert {}: {}", alert.id, e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        error!("Failed to evaluate alert rules: {}", e);
                    }
                }
            }
        });
    }

    /// Start the metrics persistence background task
    async fn start_metrics_persistence_loop(&self) {
        let metrics_collector = self.metrics_collector.clone();
        let persistence_manager = self.persistence_manager.clone();
        let interval_duration = Duration::from_secs(self.config.metrics_persistence_interval);

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                // Get recent metrics and persist them
                let recent_metrics = metrics_collector.get_recent_metrics(1000).await;
                
                for metric in recent_metrics {
                    if let Err(e) = persistence_manager.save_task_metric(&metric).await {
                        error!("Failed to persist task metric {}: {}", metric.id, e);
                    }
                }
            }
        });
    }

    /// Start the cleanup background task
    async fn start_cleanup_loop(&self) {
        let persistence_manager = self.persistence_manager.clone();
        let retention_days = self.config.data_retention_days;
        let interval_duration = Duration::from_secs(self.config.cleanup_interval_hours * 3600);

        tokio::spawn(async move {
            let mut interval = interval(interval_duration);
            
            loop {
                interval.tick().await;
                
                if let Err(e) = persistence_manager.cleanup_old_data(retention_days).await {
                    error!("Failed to cleanup old data: {}", e);
                } else {
                    info!("Completed data cleanup (retention: {} days)", retention_days);
                }
            }
        });
    }

    /// Start the admin API server
    async fn start_admin_api(&self) -> Result<()> {
        let state = AdminApiState {
            metrics_collector: self.metrics_collector.clone(),
            alert_manager: self.alert_manager.clone(),
            persistence_manager: self.persistence_manager.clone(),
            scheduler_service: self.scheduler_service.clone(),
            task_queue: self.task_queue.clone(),
        };

        let bind_address = self.config.admin_api_address.clone();
        
        tokio::spawn(async move {
            if let Err(e) = start_admin_api_server(state, &bind_address).await {
                error!("Admin API server failed: {}", e);
            }
        });

        Ok(())
    }

    /// Set up default alert rules if none exist
    async fn setup_default_alert_rules(&self) -> Result<()> {
        let existing_rules = self.alert_manager.get_rules().await;
        
        if existing_rules.is_empty() {
            info!("Setting up default alert rules");

            // High failure rate alert
            let failure_rate_rule = AlertRule::new(
                "High Failure Rate".to_string(),
                "".to_string(), // All task types
                "".to_string(), // All task names
                AlertCondition::FailureRate {
                    threshold_percent: 50.0,
                    window_minutes: 15,
                    min_executions: 5,
                },
                AlertSeverity::High,
                vec!["log".to_string()],
            );

            // Consecutive failures alert
            let consecutive_failures_rule = AlertRule::new(
                "Consecutive Failures".to_string(),
                "".to_string(),
                "".to_string(),
                AlertCondition::ConsecutiveFailures {
                    threshold: 3,
                },
                AlertSeverity::Critical,
                vec!["log".to_string()],
            );

            // Long execution duration alert
            let long_duration_rule = AlertRule::new(
                "Long Execution Duration".to_string(),
                "".to_string(),
                "".to_string(),
                AlertCondition::ExecutionDuration {
                    threshold_seconds: 600, // 10 minutes
                    violations_in_window: 2,
                    window_minutes: 60,
                },
                AlertSeverity::Medium,
                vec!["log".to_string()],
            );

            // High retry rate alert
            let high_retry_rule = AlertRule::new(
                "High Retry Rate".to_string(),
                "".to_string(),
                "".to_string(),
                AlertCondition::HighRetryRate {
                    threshold_ratio: 2.0, // More than 2 retries per execution
                    window_minutes: 30,
                    min_executions: 3,
                },
                AlertSeverity::Medium,
                vec!["log".to_string()],
            );

            // Add all default rules
            for rule in [failure_rate_rule, consecutive_failures_rule, long_duration_rule, high_retry_rule] {
                self.alert_manager.add_rule(rule.clone()).await?;
                self.persistence_manager.save_alert_rule(&rule).await?;
            }

            info!("Default alert rules created successfully");
        }

        Ok(())
    }

    /// Get the metrics collector
    pub fn metrics_collector(&self) -> Arc<MetricsCollector> {
        self.metrics_collector.clone()
    }

    /// Get the alert manager
    pub fn alert_manager(&self) -> Arc<AlertManager> {
        self.alert_manager.clone()
    }

    /// Get the persistence manager
    pub fn persistence_manager(&self) -> Arc<PersistenceManager> {
        self.persistence_manager.clone()
    }

    /// Get the timezone manager
    pub fn timezone_manager(&self) -> Arc<TimezoneManager> {
        self.timezone_manager.clone()
    }

    /// Get system health status
    pub async fn get_health_status(&self) -> Result<MonitoringHealthStatus> {
        let active_alerts = self.alert_manager.get_active_alerts().await.len();
        let database_stats = self.persistence_manager.get_database_stats().await?;
        let scheduler_running = self.scheduler_service.is_running().await;
        let queue_stats = self.task_queue.get_stats().await;

        let status = if scheduler_running && active_alerts == 0 {
            "healthy"
        } else if scheduler_running {
            "warning"
        } else {
            "critical"
        };

        Ok(MonitoringHealthStatus {
            status: status.to_string(),
            active_alerts,
            database_stats,
            scheduler_running,
            queue_pending_tasks: queue_stats.pending_tasks,
            queue_running_tasks: queue_stats.running_tasks,
            last_check: Utc::now(),
        })
    }
}

/// Health status of the monitoring system
#[derive(Debug, Clone)]
pub struct MonitoringHealthStatus {
    /// Overall system status
    pub status: String,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Database statistics
    pub database_stats: std::collections::HashMap<String, i64>,
    /// Whether the scheduler is running
    pub scheduler_running: bool,
    /// Number of pending tasks in queue
    pub queue_pending_tasks: usize,
    /// Number of running tasks in queue
    pub queue_running_tasks: usize,
    /// When this status was last checked
    pub last_check: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    use tokio::time::sleep;

    async fn create_test_monitoring_system() -> Result<MonitoringSystem> {
        let config = MonitoringConfig {
            database_path: ":memory:".to_string(), // Use in-memory database for testing
            admin_api_address: "127.0.0.1:0".to_string(), // Use random port
            max_metrics_in_memory: 100,
            alert_evaluation_interval: 1, // 1 second for testing
            metrics_persistence_interval: 1,
            data_retention_days: 1,
            cleanup_interval_hours: 1,
        };

        let scheduler = Arc::new(SchedulerService::new().await?);
        let task_manager = Arc::new(TaskManager::with_defaults());
        let task_queue = Arc::new(TaskQueue::with_defaults(task_manager.clone())?);

        MonitoringSystem::new(config, scheduler, task_manager, task_queue).await
    }

    #[tokio::test]
    async fn test_monitoring_system_creation() {
        let monitoring_system = create_test_monitoring_system().await.unwrap();

        // Verify components are initialized
        assert!(!monitoring_system.metrics_collector.prometheus_registry().gather().is_empty());

        let health_status = monitoring_system.get_health_status().await.unwrap();
        assert_eq!(health_status.active_alerts, 0);
        assert!(!health_status.scheduler_running); // Scheduler not started yet
    }

    #[tokio::test]
    async fn test_metrics_collection() {
        let monitoring_system = create_test_monitoring_system().await.unwrap();
        let metrics_collector = monitoring_system.metrics_collector();

        // Start tracking a task
        let metric_id = metrics_collector.start_task_execution(
            "test_task_1".to_string(),
            "Test Task".to_string(),
            "test_type".to_string(),
        ).await;

        // Simulate task completion
        sleep(Duration::from_millis(100)).await;
        metrics_collector.mark_task_success(metric_id).await.unwrap();

        // Verify metric was recorded
        let metric = metrics_collector.get_metric(metric_id).await.unwrap();
        assert_eq!(metric.task_name, "Test Task");
        assert_eq!(metric.task_type, "test_type");
        assert_eq!(metric.success, Some(true));
        assert!(metric.duration_ms.is_some());
    }

    #[tokio::test]
    async fn test_alert_generation() {
        let monitoring_system = create_test_monitoring_system().await.unwrap();
        let metrics_collector = monitoring_system.metrics_collector();
        let alert_manager = monitoring_system.alert_manager();

        // Create a failure rate alert rule
        let alert_rule = AlertRule::new(
            "Test Failure Rate".to_string(),
            "test_type".to_string(),
            "".to_string(),
            AlertCondition::FailureRate {
                threshold_percent: 50.0,
                window_minutes: 1,
                min_executions: 2,
            },
            AlertSeverity::High,
            vec!["log".to_string()],
        );

        alert_manager.add_rule(alert_rule).await.unwrap();

        // Generate some failed tasks to trigger the alert
        for i in 0..3 {
            let metric_id = metrics_collector.start_task_execution(
                format!("test_task_{}", i),
                "Test Task".to_string(),
                "test_type".to_string(),
            ).await;

            metrics_collector.mark_task_failure(metric_id, "Test failure".to_string()).await.unwrap();
        }

        // Evaluate alert rules
        let new_alerts = alert_manager.evaluate_rules().await.unwrap();
        assert!(!new_alerts.is_empty());

        let alert = &new_alerts[0];
        assert_eq!(alert.rule_name, "Test Failure Rate");
        assert_eq!(alert.severity, AlertSeverity::High);
        assert!(alert.message.contains("failure rate"));
    }

    #[tokio::test]
    async fn test_persistence() {
        let monitoring_system = create_test_monitoring_system().await.unwrap();
        let persistence_manager = monitoring_system.persistence_manager();

        // Create a test schedule
        let schedule = crate::schedule_config::ScheduleConfig {
            id: "test_schedule".to_string(),
            name: "Test Schedule".to_string(),
            cron_expression: "0 0 12 * * *".to_string(),
            task_type: crate::schedule_config::TaskType::AutoGraph,
            priority: crate::schedule_config::SchedulePriority::Normal,
            enabled: true,
            description: Some("Test schedule description".to_string()),
            timezone: "UTC".to_string(),
            parameters: std::collections::HashMap::new(),
            max_retries: 3,
            timeout_seconds: 300,
        };

        // Save and load the schedule
        persistence_manager.save_schedule(&schedule).await.unwrap();
        let loaded_schedules = persistence_manager.load_schedules().await.unwrap();

        assert_eq!(loaded_schedules.len(), 1);
        assert_eq!(loaded_schedules[0].id, "test_schedule");
        assert_eq!(loaded_schedules[0].name, "Test Schedule");
    }

    #[tokio::test]
    async fn test_timezone_support() {
        let monitoring_system = create_test_monitoring_system().await.unwrap();
        let timezone_manager = monitoring_system.timezone_manager();

        // Test timezone validation
        assert!(timezone_manager.validate_timezone("UTC").is_ok());
        assert!(timezone_manager.validate_timezone("America/New_York").is_ok());
        assert!(timezone_manager.validate_timezone("Invalid/Timezone").is_err());

        // Test timezone resolution
        let resolved = timezone_manager.resolve_timezone("EST").unwrap();
        assert_eq!(resolved, "America/New_York");

        // Test timezone info
        let info = timezone_manager.get_timezone_info("UTC").unwrap();
        assert_eq!(info.name, "UTC");
        assert_eq!(info.utc_offset_seconds, 0);
    }
}
