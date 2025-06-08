//! Persistent storage for schedules, metrics, and alerts
//!
//! This module provides SQLite-based persistence for schedule definitions,
//! task execution metrics, and alert configurations with recovery capabilities.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde_json;
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

use crate::alerting::{Alert, AlertRule, AlertRuleId, NotificationChannel};
use crate::metrics::{AggregatedMetrics, TaskExecutionMetric};
use crate::schedule_config::{ScheduleConfig, ScheduleConfigCollection};

/// Database schema version for migrations
const SCHEMA_VERSION: i32 = 1;

/// Persistent storage manager for the scheduling system
pub struct PersistenceManager {
    /// SQLite connection pool
    pool: SqlitePool,
}

impl PersistenceManager {
    /// Create a new persistence manager with the given database path
    pub async fn new(database_path: &str) -> Result<Self> {
        info!("Initializing persistence manager with database: {}", database_path);

        // Create database connection
        let database_url = format!("sqlite:{}", database_path);
        let pool = SqlitePool::connect(&database_url)
            .await
            .with_context(|| format!("Failed to connect to database: {}", database_path))?;

        let manager = Self { pool };

        // Initialize database schema
        manager.initialize_schema().await?;

        info!("Persistence manager initialized successfully");
        Ok(manager)
    }

    /// Initialize the database schema
    async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema");

        // Create schema version table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schema_version (
                version INTEGER PRIMARY KEY,
                applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Check current schema version
        let current_version: Option<i32> = sqlx::query_scalar(
            "SELECT version FROM schema_version ORDER BY version DESC LIMIT 1"
        )
        .fetch_optional(&self.pool)
        .await?;

        match current_version {
            Some(version) if version >= SCHEMA_VERSION => {
                debug!("Database schema is up to date (version {})", version);
                return Ok(());
            }
            Some(version) => {
                info!("Upgrading database schema from version {} to {}", version, SCHEMA_VERSION);
            }
            None => {
                info!("Creating initial database schema (version {})", SCHEMA_VERSION);
            }
        }

        // Create schedules table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS schedules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                cron_expression TEXT NOT NULL,
                task_type TEXT NOT NULL,
                priority TEXT NOT NULL DEFAULT 'normal',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                description TEXT,
                timezone TEXT NOT NULL DEFAULT 'UTC',
                parameters TEXT, -- JSON
                max_retries INTEGER NOT NULL DEFAULT 3,
                timeout_seconds INTEGER NOT NULL DEFAULT 300,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create task execution metrics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS task_metrics (
                id TEXT PRIMARY KEY,
                task_id TEXT NOT NULL,
                task_name TEXT NOT NULL,
                task_type TEXT NOT NULL,
                started_at DATETIME NOT NULL,
                finished_at DATETIME,
                duration_ms INTEGER,
                success BOOLEAN,
                error_message TEXT,
                retry_count INTEGER NOT NULL DEFAULT 0,
                memory_usage_mb REAL,
                cpu_usage_percent REAL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create alert rules table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alert_rules (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                task_type_filter TEXT NOT NULL DEFAULT '',
                task_name_filter TEXT NOT NULL DEFAULT '',
                condition_type TEXT NOT NULL,
                condition_config TEXT NOT NULL, -- JSON
                severity TEXT NOT NULL DEFAULT 'medium',
                enabled BOOLEAN NOT NULL DEFAULT 1,
                cooldown_minutes INTEGER NOT NULL DEFAULT 15,
                notification_channels TEXT, -- JSON array
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create alerts table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                rule_id TEXT NOT NULL,
                rule_name TEXT NOT NULL,
                task_type TEXT NOT NULL,
                task_name TEXT,
                severity TEXT NOT NULL,
                message TEXT NOT NULL,
                context TEXT, -- JSON
                triggered_at DATETIME NOT NULL,
                updated_at DATETIME NOT NULL,
                acknowledged BOOLEAN NOT NULL DEFAULT 0,
                acknowledged_at DATETIME,
                acknowledged_by TEXT,
                resolved BOOLEAN NOT NULL DEFAULT 0,
                resolved_at DATETIME,
                resolution_reason TEXT,
                FOREIGN KEY (rule_id) REFERENCES alert_rules (id)
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create notification channels table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS notification_channels (
                name TEXT PRIMARY KEY,
                channel_type TEXT NOT NULL,
                config TEXT NOT NULL, -- JSON
                enabled BOOLEAN NOT NULL DEFAULT 1,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create aggregated metrics table for historical data
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS aggregated_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                task_type TEXT NOT NULL,
                period_start DATETIME NOT NULL,
                period_end DATETIME NOT NULL,
                total_executions INTEGER NOT NULL,
                successful_executions INTEGER NOT NULL,
                failed_executions INTEGER NOT NULL,
                success_rate REAL NOT NULL,
                avg_duration_ms REAL NOT NULL,
                min_duration_ms INTEGER NOT NULL,
                max_duration_ms INTEGER NOT NULL,
                total_retries INTEGER NOT NULL,
                avg_memory_usage_mb REAL,
                avg_cpu_usage_percent REAL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_task_metrics_task_type ON task_metrics(task_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_task_metrics_started_at ON task_metrics(started_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_rule_id ON alerts(rule_id)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_alerts_triggered_at ON alerts(triggered_at)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_aggregated_metrics_task_type ON aggregated_metrics(task_type)")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_aggregated_metrics_period ON aggregated_metrics(period_start, period_end)")
            .execute(&self.pool)
            .await?;

        // Update schema version
        sqlx::query("INSERT OR REPLACE INTO schema_version (version) VALUES (?)")
            .bind(SCHEMA_VERSION)
            .execute(&self.pool)
            .await?;

        info!("Database schema initialized successfully");
        Ok(())
    }

    /// Save a schedule configuration
    pub async fn save_schedule(&self, schedule: &ScheduleConfig) -> Result<()> {
        let parameters_json = serde_json::to_string(&schedule.parameters)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO schedules (
                id, name, cron_expression, task_type, priority, enabled,
                description, timezone, parameters, max_retries, timeout_seconds,
                updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&schedule.id)
        .bind(&schedule.name)
        .bind(&schedule.cron_expression)
        .bind(serde_json::to_string(&schedule.task_type)?)
        .bind(serde_json::to_string(&schedule.priority)?)
        .bind(schedule.enabled)
        .bind(&schedule.description)
        .bind(&schedule.timezone)
        .bind(parameters_json)
        .bind(schedule.max_retries as i64)
        .bind(schedule.timeout_seconds as i64)
        .execute(&self.pool)
        .await?;

        debug!("Saved schedule: {} ({})", schedule.name, schedule.id);
        Ok(())
    }

    /// Load all schedule configurations
    pub async fn load_schedules(&self) -> Result<Vec<ScheduleConfig>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, cron_expression, task_type, priority, enabled,
                   description, timezone, parameters, max_retries, timeout_seconds
            FROM schedules
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut schedules = Vec::new();
        for row in rows {
            let parameters_json: String = row.get("parameters");
            let parameters: HashMap<String, serde_json::Value> = serde_json::from_str(&parameters_json)
                .unwrap_or_default();

            let task_type_json: String = row.get("task_type");
            let task_type = serde_json::from_str(&task_type_json)?;

            let priority_json: String = row.get("priority");
            let priority = serde_json::from_str(&priority_json)?;

            let schedule = ScheduleConfig {
                id: row.get("id"),
                name: row.get("name"),
                cron_expression: row.get("cron_expression"),
                task_type,
                priority,
                enabled: row.get("enabled"),
                description: row.get("description"),
                timezone: row.get("timezone"),
                parameters,
                max_retries: row.get::<i64, _>("max_retries") as u32,
                timeout_seconds: row.get::<i64, _>("timeout_seconds") as u64,
            };

            schedules.push(schedule);
        }

        info!("Loaded {} schedules from database", schedules.len());
        Ok(schedules)
    }

    /// Delete a schedule configuration
    pub async fn delete_schedule(&self, schedule_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM schedules WHERE id = ?")
            .bind(schedule_id)
            .execute(&self.pool)
            .await?;

        let deleted = result.rows_affected() > 0;
        if deleted {
            info!("Deleted schedule: {}", schedule_id);
        } else {
            warn!("Attempted to delete non-existent schedule: {}", schedule_id);
        }

        Ok(deleted)
    }

    /// Save a task execution metric
    pub async fn save_task_metric(&self, metric: &TaskExecutionMetric) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO task_metrics (
                id, task_id, task_name, task_type, started_at, finished_at,
                duration_ms, success, error_message, retry_count,
                memory_usage_mb, cpu_usage_percent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(metric.id.to_string())
        .bind(&metric.task_id)
        .bind(&metric.task_name)
        .bind(&metric.task_type)
        .bind(metric.started_at)
        .bind(metric.finished_at)
        .bind(metric.duration_ms.map(|d| d as i64))
        .bind(metric.success)
        .bind(&metric.error_message)
        .bind(metric.retry_count as i64)
        .bind(metric.memory_usage_mb)
        .bind(metric.cpu_usage_percent)
        .execute(&self.pool)
        .await?;

        debug!("Saved task metric: {} ({})", metric.task_name, metric.id);
        Ok(())
    }

    /// Load task execution metrics within a time range
    pub async fn load_task_metrics(
        &self,
        task_type: Option<&str>,
        start_time: Option<DateTime<Utc>>,
        end_time: Option<DateTime<Utc>>,
        limit: Option<u32>,
    ) -> Result<Vec<TaskExecutionMetric>> {
        let mut query = "SELECT * FROM task_metrics WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(task_type) = task_type {
            query.push_str(" AND task_type = ?");
            params.push(task_type);
        }

        let start_time_str;
        let end_time_str;

        if let Some(start_time) = start_time {
            query.push_str(" AND started_at >= ?");
            start_time_str = start_time.to_rfc3339();
            params.push(&start_time_str);
        }

        if let Some(end_time) = end_time {
            query.push_str(" AND started_at <= ?");
            end_time_str = end_time.to_rfc3339();
            params.push(&end_time_str);
        }

        query.push_str(" ORDER BY started_at DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut metrics = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str)?;

            let started_at_str: String = row.get("started_at");
            let started_at = DateTime::parse_from_rfc3339(&started_at_str)?.with_timezone(&Utc);

            let finished_at = if let Some(finished_at_str) = row.get::<Option<String>, _>("finished_at") {
                Some(DateTime::parse_from_rfc3339(&finished_at_str)?.with_timezone(&Utc))
            } else {
                None
            };

            let metric = TaskExecutionMetric {
                id,
                task_id: row.get("task_id"),
                task_name: row.get("task_name"),
                task_type: row.get("task_type"),
                started_at,
                finished_at,
                duration_ms: row.get::<Option<i64>, _>("duration_ms").map(|d| d as u64),
                success: row.get("success"),
                error_message: row.get("error_message"),
                retry_count: row.get::<i64, _>("retry_count") as u32,
                memory_usage_mb: row.get("memory_usage_mb"),
                cpu_usage_percent: row.get("cpu_usage_percent"),
            };

            metrics.push(metric);
        }

        debug!("Loaded {} task metrics from database", metrics.len());
        Ok(metrics)
    }

    /// Save an alert rule
    pub async fn save_alert_rule(&self, rule: &AlertRule) -> Result<()> {
        let condition_config = serde_json::to_string(&rule.condition)?;
        let notification_channels = serde_json::to_string(&rule.notification_channels)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO alert_rules (
                id, name, description, task_type_filter, task_name_filter,
                condition_type, condition_config, severity, enabled,
                cooldown_minutes, notification_channels, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(rule.id.to_string())
        .bind(&rule.name)
        .bind(&rule.description)
        .bind(&rule.task_type_filter)
        .bind(&rule.task_name_filter)
        .bind("alert_condition") // condition_type for future extensibility
        .bind(condition_config)
        .bind(serde_json::to_string(&rule.severity)?)
        .bind(rule.enabled)
        .bind(rule.cooldown_minutes as i64)
        .bind(notification_channels)
        .execute(&self.pool)
        .await?;

        debug!("Saved alert rule: {} ({})", rule.name, rule.id);
        Ok(())
    }

    /// Load all alert rules
    pub async fn load_alert_rules(&self) -> Result<Vec<AlertRule>> {
        let rows = sqlx::query(
            r#"
            SELECT id, name, description, task_type_filter, task_name_filter,
                   condition_config, severity, enabled, cooldown_minutes,
                   notification_channels, created_at, updated_at
            FROM alert_rules
            ORDER BY name
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        let mut rules = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str)?;

            let condition_config_str: String = row.get("condition_config");
            let condition = serde_json::from_str(&condition_config_str)?;

            let severity_str: String = row.get("severity");
            let severity = serde_json::from_str(&severity_str)?;

            let notification_channels_str: String = row.get("notification_channels");
            let notification_channels = serde_json::from_str(&notification_channels_str)?;

            let created_at_str: String = row.get("created_at");
            let created_at = DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc);

            let updated_at_str: String = row.get("updated_at");
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc);

            let rule = AlertRule {
                id,
                name: row.get("name"),
                description: row.get("description"),
                task_type_filter: row.get("task_type_filter"),
                task_name_filter: row.get("task_name_filter"),
                condition,
                severity,
                enabled: row.get("enabled"),
                cooldown_minutes: row.get::<i64, _>("cooldown_minutes") as u32,
                notification_channels,
                created_at,
                updated_at,
            };

            rules.push(rule);
        }

        info!("Loaded {} alert rules from database", rules.len());
        Ok(rules)
    }

    /// Save an alert
    pub async fn save_alert(&self, alert: &Alert) -> Result<()> {
        let context = serde_json::to_string(&alert.context)?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO alerts (
                id, rule_id, rule_name, task_type, task_name, severity,
                message, context, triggered_at, updated_at, acknowledged,
                acknowledged_at, acknowledged_by, resolved, resolved_at,
                resolution_reason
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(alert.id.to_string())
        .bind(alert.rule_id.to_string())
        .bind(&alert.rule_name)
        .bind(&alert.task_type)
        .bind(&alert.task_name)
        .bind(serde_json::to_string(&alert.severity)?)
        .bind(&alert.message)
        .bind(context)
        .bind(alert.triggered_at)
        .bind(alert.updated_at)
        .bind(alert.acknowledged)
        .bind(alert.acknowledged_at)
        .bind(&alert.acknowledged_by)
        .bind(alert.resolved)
        .bind(alert.resolved_at)
        .bind(&alert.resolution_reason)
        .execute(&self.pool)
        .await?;

        debug!("Saved alert: {} ({})", alert.rule_name, alert.id);
        Ok(())
    }

    /// Load alerts with optional filtering
    pub async fn load_alerts(
        &self,
        rule_id: Option<AlertRuleId>,
        active_only: bool,
        limit: Option<u32>,
    ) -> Result<Vec<Alert>> {
        let mut query = "SELECT * FROM alerts WHERE 1=1".to_string();
        let mut params = Vec::new();

        if let Some(rule_id) = rule_id {
            query.push_str(" AND rule_id = ?");
            params.push(rule_id.to_string());
        }

        if active_only {
            query.push_str(" AND acknowledged = 0 AND resolved = 0");
        }

        query.push_str(" ORDER BY triggered_at DESC");

        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
        }

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(&self.pool).await?;

        let mut alerts = Vec::new();
        for row in rows {
            let id_str: String = row.get("id");
            let id = Uuid::parse_str(&id_str)?;

            let rule_id_str: String = row.get("rule_id");
            let rule_id = Uuid::parse_str(&rule_id_str)?;

            let context_str: String = row.get("context");
            let context = serde_json::from_str(&context_str)?;

            let severity_str: String = row.get("severity");
            let severity = serde_json::from_str(&severity_str)?;

            let triggered_at_str: String = row.get("triggered_at");
            let triggered_at = DateTime::parse_from_rfc3339(&triggered_at_str)?.with_timezone(&Utc);

            let updated_at_str: String = row.get("updated_at");
            let updated_at = DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc);

            let acknowledged_at = if let Some(ack_str) = row.get::<Option<String>, _>("acknowledged_at") {
                Some(DateTime::parse_from_rfc3339(&ack_str)?.with_timezone(&Utc))
            } else {
                None
            };

            let resolved_at = if let Some(res_str) = row.get::<Option<String>, _>("resolved_at") {
                Some(DateTime::parse_from_rfc3339(&res_str)?.with_timezone(&Utc))
            } else {
                None
            };

            let alert = Alert {
                id,
                rule_id,
                rule_name: row.get("rule_name"),
                task_type: row.get("task_type"),
                task_name: row.get("task_name"),
                severity,
                message: row.get("message"),
                context,
                triggered_at,
                updated_at,
                acknowledged: row.get("acknowledged"),
                acknowledged_at,
                acknowledged_by: row.get("acknowledged_by"),
                resolved: row.get("resolved"),
                resolved_at,
                resolution_reason: row.get("resolution_reason"),
            };

            alerts.push(alert);
        }

        debug!("Loaded {} alerts from database", alerts.len());
        Ok(alerts)
    }

    /// Save aggregated metrics for historical analysis
    pub async fn save_aggregated_metrics(&self, metrics: &AggregatedMetrics) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO aggregated_metrics (
                task_type, period_start, period_end, total_executions,
                successful_executions, failed_executions, success_rate,
                avg_duration_ms, min_duration_ms, max_duration_ms,
                total_retries, avg_memory_usage_mb, avg_cpu_usage_percent
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&metrics.task_type)
        .bind(metrics.period_start)
        .bind(metrics.period_end)
        .bind(metrics.total_executions as i64)
        .bind(metrics.successful_executions as i64)
        .bind(metrics.failed_executions as i64)
        .bind(metrics.success_rate)
        .bind(metrics.avg_duration_ms)
        .bind(metrics.min_duration_ms as i64)
        .bind(metrics.max_duration_ms as i64)
        .bind(metrics.total_retries as i64)
        .bind(metrics.avg_memory_usage_mb)
        .bind(metrics.avg_cpu_usage_percent)
        .execute(&self.pool)
        .await?;

        debug!("Saved aggregated metrics for task type: {}", metrics.task_type);
        Ok(())
    }

    /// Clean up old data to prevent database bloat
    pub async fn cleanup_old_data(&self, retention_days: u32) -> Result<()> {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        // Clean up old task metrics
        let metrics_deleted = sqlx::query("DELETE FROM task_metrics WHERE started_at < ?")
            .bind(cutoff_date)
            .execute(&self.pool)
            .await?
            .rows_affected();

        // Clean up old resolved alerts
        let alerts_deleted = sqlx::query(
            "DELETE FROM alerts WHERE resolved = 1 AND resolved_at < ?"
        )
        .bind(cutoff_date)
        .execute(&self.pool)
        .await?
        .rows_affected();

        info!(
            "Cleaned up old data: {} task metrics, {} resolved alerts (retention: {} days)",
            metrics_deleted, alerts_deleted, retention_days
        );

        Ok(())
    }

    /// Get database statistics
    pub async fn get_database_stats(&self) -> Result<HashMap<String, i64>> {
        let mut stats = HashMap::new();

        // Count schedules
        let schedule_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM schedules")
            .fetch_one(&self.pool)
            .await?;
        stats.insert("schedules".to_string(), schedule_count);

        // Count task metrics
        let metrics_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM task_metrics")
            .fetch_one(&self.pool)
            .await?;
        stats.insert("task_metrics".to_string(), metrics_count);

        // Count alert rules
        let rules_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alert_rules")
            .fetch_one(&self.pool)
            .await?;
        stats.insert("alert_rules".to_string(), rules_count);

        // Count alerts
        let alerts_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM alerts")
            .fetch_one(&self.pool)
            .await?;
        stats.insert("alerts".to_string(), alerts_count);

        // Count active alerts
        let active_alerts_count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM alerts WHERE acknowledged = 0 AND resolved = 0"
        )
        .fetch_one(&self.pool)
        .await?;
        stats.insert("active_alerts".to_string(), active_alerts_count);

        Ok(stats)
    }
}
