//! Admin API for monitoring and managing the scheduling system
//!
//! This module provides HTTP endpoints for viewing task status, metrics,
//! alerts, and system health information.

use anyhow::Result;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::Json,
    routing::{delete, get, post, put},
    Router,
};
use chrono::{DateTime, Utc};
use prometheus::{Encoder, TextEncoder};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use uuid::Uuid;

use crate::alerting::{Alert, AlertManager, AlertRule, AlertRuleId, AlertSeverity};
use crate::metrics::{AggregatedMetrics, MetricsCollector, TaskExecutionMetric};
use crate::persistence::PersistenceManager;
use crate::schedule_config::ScheduleConfig;
use crate::scheduler::SchedulerService;
use crate::task_queue::{QueueStats, TaskQueue};

/// Shared application state for the admin API
#[derive(Clone)]
pub struct AdminApiState {
    /// Metrics collector for task execution data
    pub metrics_collector: Arc<MetricsCollector>,
    /// Alert manager for monitoring and alerting
    pub alert_manager: Arc<AlertManager>,
    /// Persistence manager for database operations
    pub persistence_manager: Arc<PersistenceManager>,
    /// Scheduler service for job management
    pub scheduler_service: Arc<SchedulerService>,
    /// Task queue for priority-based execution
    pub task_queue: Arc<TaskQueue>,
}

/// Query parameters for metrics endpoints
#[derive(Debug, Deserialize)]
pub struct MetricsQuery {
    /// Task type filter
    pub task_type: Option<String>,
    /// Start time for metrics (ISO 8601)
    pub start_time: Option<DateTime<Utc>>,
    /// End time for metrics (ISO 8601)
    pub end_time: Option<DateTime<Utc>>,
    /// Maximum number of results
    pub limit: Option<u32>,
}

/// Query parameters for alerts endpoints
#[derive(Debug, Deserialize)]
pub struct AlertsQuery {
    /// Filter by rule ID
    pub rule_id: Option<Uuid>,
    /// Show only active alerts
    pub active_only: Option<bool>,
    /// Maximum number of results
    pub limit: Option<u32>,
}

/// Request body for acknowledging alerts
#[derive(Debug, Deserialize)]
pub struct AcknowledgeAlertRequest {
    /// Who is acknowledging the alert
    pub acknowledged_by: String,
}

/// Request body for resolving alerts
#[derive(Debug, Deserialize)]
pub struct ResolveAlertRequest {
    /// Reason for resolving the alert
    pub reason: String,
}

/// System health status response
#[derive(Debug, Serialize)]
pub struct SystemHealthResponse {
    /// Overall system status
    pub status: String,
    /// Scheduler status
    pub scheduler_running: bool,
    /// Number of active jobs
    pub active_jobs: usize,
    /// Task queue statistics
    pub queue_stats: QueueStats,
    /// Number of active alerts
    pub active_alerts: usize,
    /// Database statistics
    pub database_stats: HashMap<String, i64>,
    /// System uptime information
    pub uptime_seconds: u64,
}

/// Create the admin API router with all endpoints
pub fn create_admin_api_router(state: AdminApiState) -> Router {
    Router::new()
        // Health and status endpoints
        .route("/health", get(get_system_health))
        .route("/metrics/prometheus", get(get_prometheus_metrics))
        
        // Task execution metrics endpoints
        .route("/metrics/tasks", get(get_task_metrics))
        .route("/metrics/aggregated/:task_type", get(get_aggregated_metrics))
        
        // Schedule management endpoints
        .route("/schedules", get(get_schedules))
        .route("/schedules/:id", get(get_schedule))
        .route("/schedules/:id", delete(delete_schedule))
        
        // Alert management endpoints
        .route("/alerts", get(get_alerts))
        .route("/alerts/:id", get(get_alert))
        .route("/alerts/:id/acknowledge", post(acknowledge_alert))
        .route("/alerts/:id/resolve", post(resolve_alert))
        
        // Alert rule management endpoints
        .route("/alert-rules", get(get_alert_rules))
        .route("/alert-rules", post(create_alert_rule))
        .route("/alert-rules/:id", get(get_alert_rule))
        .route("/alert-rules/:id", put(update_alert_rule))
        .route("/alert-rules/:id", delete(delete_alert_rule))
        
        // Task queue endpoints
        .route("/queue/stats", get(get_queue_stats))
        
        // Scheduler endpoints
        // .route("/scheduler/jobs", get(get_scheduler_jobs)) // Temporarily disabled due to type issues
        
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive())
        )
        .with_state(state)
}

/// Get overall system health status
async fn get_system_health(
    State(state): State<AdminApiState>,
) -> Result<Json<SystemHealthResponse>, StatusCode> {
    let scheduler_running = state.scheduler_service.is_running().await;
    let active_jobs = state.scheduler_service.job_count().await;
    
    let queue_stats = state.task_queue.get_stats().await;
    
    let active_alerts = state.alert_manager.get_active_alerts().await.len();
    
    let database_stats = state.persistence_manager.get_database_stats().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Simple uptime calculation (would be better to track actual start time)
    let uptime_seconds = 0; // TODO: Implement proper uptime tracking

    let status = if scheduler_running && active_alerts == 0 {
        "healthy"
    } else if scheduler_running {
        "warning"
    } else {
        "critical"
    };

    let response = SystemHealthResponse {
        status: status.to_string(),
        scheduler_running,
        active_jobs,
        queue_stats,
        active_alerts,
        database_stats,
        uptime_seconds,
    };

    Ok(Json(response))
}

/// Get Prometheus metrics for external monitoring
async fn get_prometheus_metrics(
    State(state): State<AdminApiState>,
) -> Result<String, StatusCode> {
    let registry = state.metrics_collector.prometheus_registry();
    let encoder = TextEncoder::new();
    let metric_families = registry.gather();
    
    encoder.encode_to_string(&metric_families)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

/// Get task execution metrics
async fn get_task_metrics(
    Query(query): Query<MetricsQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<TaskExecutionMetric>>, StatusCode> {
    let metrics = if let Some(task_type) = &query.task_type {
        state.metrics_collector.get_metrics_by_task_type(task_type).await
    } else {
        let limit = query.limit.unwrap_or(100);
        state.metrics_collector.get_recent_metrics(limit as usize).await
    };

    // Apply time filtering if specified
    let filtered_metrics = if query.start_time.is_some() || query.end_time.is_some() {
        metrics.into_iter()
            .filter(|m| {
                if let Some(start) = query.start_time {
                    if m.started_at < start {
                        return false;
                    }
                }
                if let Some(end) = query.end_time {
                    if m.started_at > end {
                        return false;
                    }
                }
                true
            })
            .collect()
    } else {
        metrics
    };

    Ok(Json(filtered_metrics))
}

/// Get aggregated metrics for a specific task type
async fn get_aggregated_metrics(
    Path(task_type): Path<String>,
    Query(query): Query<MetricsQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<AggregatedMetrics>, StatusCode> {
    let end_time = query.end_time.unwrap_or_else(Utc::now);
    let start_time = query.start_time.unwrap_or_else(|| {
        end_time - chrono::Duration::hours(24) // Default to last 24 hours
    });

    let aggregated = state.metrics_collector
        .calculate_aggregated_metrics(&task_type, start_time, end_time)
        .await;

    Ok(Json(aggregated))
}

/// Get all schedule configurations
async fn get_schedules(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<ScheduleConfig>>, StatusCode> {
    let schedules = state.persistence_manager.load_schedules().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(schedules))
}

/// Get a specific schedule configuration
async fn get_schedule(
    Path(id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<Json<ScheduleConfig>, StatusCode> {
    let schedules = state.persistence_manager.load_schedules().await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let schedule = schedules.into_iter()
        .find(|s| s.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(schedule))
}

/// Delete a schedule configuration
async fn delete_schedule(
    Path(id): Path<String>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    let deleted = state.persistence_manager.delete_schedule(&id).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if deleted {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(StatusCode::NOT_FOUND)
    }
}

/// Get alerts with optional filtering
async fn get_alerts(
    Query(query): Query<AlertsQuery>,
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<Alert>>, StatusCode> {
    let alerts = if query.active_only.unwrap_or(false) {
        state.alert_manager.get_active_alerts().await
    } else {
        state.alert_manager.get_all_alerts().await
    };

    // Apply additional filtering
    let filtered_alerts = if let Some(rule_id) = query.rule_id {
        alerts.into_iter()
            .filter(|a| a.rule_id == rule_id)
            .collect()
    } else {
        alerts
    };

    // Apply limit
    let limited_alerts = if let Some(limit) = query.limit {
        filtered_alerts.into_iter()
            .take(limit as usize)
            .collect()
    } else {
        filtered_alerts
    };

    Ok(Json(limited_alerts))
}

/// Get a specific alert
async fn get_alert(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
) -> Result<Json<Alert>, StatusCode> {
    let alerts = state.alert_manager.get_all_alerts().await;
    let alert = alerts.into_iter()
        .find(|a| a.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(alert))
}

/// Acknowledge an alert
async fn acknowledge_alert(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
    Json(request): Json<AcknowledgeAlertRequest>,
) -> Result<StatusCode, StatusCode> {
    state.alert_manager.acknowledge_alert(id, request.acknowledged_by).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK)
}

/// Resolve an alert
async fn resolve_alert(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
    Json(request): Json<ResolveAlertRequest>,
) -> Result<StatusCode, StatusCode> {
    state.alert_manager.resolve_alert(id, request.reason).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::OK)
}

/// Get all alert rules
async fn get_alert_rules(
    State(state): State<AdminApiState>,
) -> Result<Json<Vec<AlertRule>>, StatusCode> {
    let rules = state.alert_manager.get_rules().await;
    Ok(Json(rules))
}

/// Create a new alert rule
async fn create_alert_rule(
    State(state): State<AdminApiState>,
    Json(rule): Json<AlertRule>,
) -> Result<Json<AlertRule>, StatusCode> {
    let rule_id = state.alert_manager.add_rule(rule.clone()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save to persistence
    state.persistence_manager.save_alert_rule(&rule).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rule))
}

/// Get a specific alert rule
async fn get_alert_rule(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
) -> Result<Json<AlertRule>, StatusCode> {
    let rule = state.alert_manager.get_rule(id).await
        .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(rule))
}

/// Update an alert rule
async fn update_alert_rule(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
    Json(mut rule): Json<AlertRule>,
) -> Result<Json<AlertRule>, StatusCode> {
    // Ensure the ID matches
    rule.id = id;
    rule.updated_at = Utc::now();

    // Update in memory
    state.alert_manager.add_rule(rule.clone()).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Save to persistence
    state.persistence_manager.save_alert_rule(&rule).await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(rule))
}

/// Delete an alert rule
async fn delete_alert_rule(
    Path(id): Path<Uuid>,
    State(state): State<AdminApiState>,
) -> Result<StatusCode, StatusCode> {
    state.alert_manager.remove_rule(id).await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Get task queue statistics
async fn get_queue_stats(
    State(state): State<AdminApiState>,
) -> Result<Json<QueueStats>, StatusCode> {
    let stats = state.task_queue.get_stats().await;

    Ok(Json(stats))
}

/// Get scheduler job information
async fn get_scheduler_jobs(
    State(state): State<AdminApiState>,
) -> Json<Vec<crate::scheduler::JobMetadata>> {
    let jobs = state.scheduler_service.list_jobs().await;
    Json(jobs)
}

/// Start the admin API server
pub async fn start_admin_api_server(
    state: AdminApiState,
    bind_address: &str,
) -> Result<()> {
    info!("Starting admin API server on {}", bind_address);

    let app = create_admin_api_router(state);

    let listener = tokio::net::TcpListener::bind(bind_address).await?;

    info!("Admin API server listening on {}", bind_address);

    axum::serve(listener, app).await?;

    Ok(())
}
