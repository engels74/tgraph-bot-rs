//! Command metrics and usage tracking system

use dashmap::DashMap;
use poise::serenity_prelude::{UserId, ChannelId};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};
use std::time::{Duration, Instant};
use tracing::{debug, info};
use chrono::{DateTime, Utc};

/// Individual command execution record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandExecution {
    /// Command name
    pub command: String,
    /// User ID who executed the command
    pub user_id: u64,
    /// Channel ID where command was executed
    pub channel_id: Option<u64>,
    /// Guild ID where command was executed
    pub guild_id: Option<u64>,
    /// Timestamp when command was executed
    pub timestamp: DateTime<Utc>,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Whether the command succeeded
    pub success: bool,
    /// Error message if command failed
    pub error: Option<String>,
    /// Additional metadata
    pub metadata: serde_json::Value,
}

/// Aggregated metrics for a specific command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandMetrics {
    /// Command name
    pub command: String,
    /// Total number of executions
    pub total_executions: u64,
    /// Number of successful executions
    pub successful_executions: u64,
    /// Number of failed executions
    pub failed_executions: u64,
    /// Average execution time in milliseconds
    pub avg_duration_ms: f64,
    /// Minimum execution time in milliseconds
    pub min_duration_ms: u64,
    /// Maximum execution time in milliseconds
    pub max_duration_ms: u64,
    /// Success rate as percentage
    pub success_rate: f64,
    /// Most recent execution timestamp
    pub last_execution: Option<DateTime<Utc>>,
    /// Unique users count
    pub unique_users: u64,
    /// Most active user ID
    pub most_active_user: Option<u64>,
    /// Executions in the last 24 hours
    pub executions_24h: u64,
    /// Executions in the last 7 days
    pub executions_7d: u64,
}

/// Complete metrics report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    /// Report generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Reporting period start
    pub period_start: DateTime<Utc>,
    /// Reporting period end
    pub period_end: DateTime<Utc>,
    /// Total number of command executions
    pub total_executions: u64,
    /// Overall success rate
    pub overall_success_rate: f64,
    /// Average response time across all commands
    pub avg_response_time_ms: f64,
    /// Command-specific metrics
    pub commands: Vec<CommandMetrics>,
    /// Top users by command usage
    pub top_users: Vec<(u64, u64)>, // (user_id, execution_count)
    /// Busiest channels
    pub top_channels: Vec<(u64, u64)>, // (channel_id, execution_count)
    /// Error frequency breakdown
    pub error_breakdown: Vec<(String, u64)>, // (error_type, count)
}

/// Thread-safe metrics manager
#[derive(Debug)]
pub struct MetricsManager {
    /// Storage for command executions (recent history)
    executions: Arc<DashMap<String, Vec<CommandExecution>>>,
    /// Aggregated metrics per command
    command_metrics: Arc<DashMap<String, CommandMetrics>>,
    /// Global counters
    total_executions: AtomicU64,
    total_successes: AtomicU64,
    total_failures: AtomicU64,
    /// Manager start time for uptime calculations
    start_time: Instant,
    /// Maximum number of executions to keep in memory per command
    max_history_per_command: usize,
}

impl Default for MetricsManager {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsManager {
    /// Create a new metrics manager
    pub fn new() -> Self {
        Self {
            executions: Arc::new(DashMap::new()),
            command_metrics: Arc::new(DashMap::new()),
            total_executions: AtomicU64::new(0),
            total_successes: AtomicU64::new(0),
            total_failures: AtomicU64::new(0),
            start_time: Instant::now(),
            max_history_per_command: 1000, // Keep last 1000 executions per command
        }
    }

    /// Record a command execution
    pub fn record_execution(
        &self,
        command: &str,
        user_id: UserId,
        channel_id: Option<ChannelId>,
        guild_id: Option<u64>,
        duration: Duration,
        success: bool,
        error: Option<String>,
        metadata: serde_json::Value,
    ) {
        let execution = CommandExecution {
            command: command.to_string(),
            user_id: user_id.get(),
            channel_id: channel_id.map(|c| c.get()),
            guild_id,
            timestamp: Utc::now(),
            duration_ms: duration.as_millis() as u64,
            success,
            error,
            metadata,
        };

        // Update global counters
        self.total_executions.fetch_add(1, Ordering::Relaxed);
        if success {
            self.total_successes.fetch_add(1, Ordering::Relaxed);
        } else {
            self.total_failures.fetch_add(1, Ordering::Relaxed);
        }

        // Store execution history
        let mut executions = self.executions.entry(command.to_string()).or_default();
        executions.push(execution.clone());

        // Trim history if it exceeds max size
        if executions.len() > self.max_history_per_command {
            let excess = executions.len() - self.max_history_per_command;
            executions.drain(0..excess);
        }

        // Update aggregated metrics
        self.update_command_metrics(command);

        debug!(
            "Recorded execution: command={}, user={}, success={}, duration={}ms",
            command, user_id, success, duration.as_millis()
        );
    }

    /// Update aggregated metrics for a command
    fn update_command_metrics(&self, command: &str) {
        // We need to recalculate metrics from all executions for this command
        if let Some(executions) = self.executions.get(command) {
            let new_metrics = self.calculate_metrics_for_command(command, &executions);
            self.command_metrics.insert(command.to_string(), new_metrics);
        }
    }

    /// Calculate comprehensive metrics for a specific command
    fn calculate_metrics_for_command(&self, command: &str, executions: &[CommandExecution]) -> CommandMetrics {
        if executions.is_empty() {
            return CommandMetrics {
                command: command.to_string(),
                total_executions: 0,
                successful_executions: 0,
                failed_executions: 0,
                avg_duration_ms: 0.0,
                min_duration_ms: 0,
                max_duration_ms: 0,
                success_rate: 0.0,
                last_execution: None,
                unique_users: 0,
                most_active_user: None,
                executions_24h: 0,
                executions_7d: 0,
            };
        }

        let total_executions = executions.len() as u64;
        let successful_executions = executions.iter().filter(|e| e.success).count() as u64;
        let failed_executions = total_executions - successful_executions;

        let durations: Vec<u64> = executions.iter().map(|e| e.duration_ms).collect();
        let avg_duration_ms = durations.iter().sum::<u64>() as f64 / durations.len() as f64;
        let min_duration_ms = *durations.iter().min().unwrap_or(&0);
        let max_duration_ms = *durations.iter().max().unwrap_or(&0);

        let success_rate = if total_executions > 0 {
            (successful_executions as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let last_execution = executions.iter().max_by_key(|e| e.timestamp).map(|e| e.timestamp);

        // Count unique users
        let mut user_counts: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
        for execution in executions {
            *user_counts.entry(execution.user_id).or_insert(0) += 1;
        }
        let unique_users = user_counts.len() as u64;
        let most_active_user = user_counts.iter().max_by_key(|(_, &count)| count).map(|(&user_id, _)| user_id);

        // Calculate recent activity
        let now = Utc::now();
        let twenty_four_hours_ago = now - chrono::Duration::hours(24);
        let seven_days_ago = now - chrono::Duration::days(7);

        let executions_24h = executions.iter().filter(|e| e.timestamp > twenty_four_hours_ago).count() as u64;
        let executions_7d = executions.iter().filter(|e| e.timestamp > seven_days_ago).count() as u64;

        CommandMetrics {
            command: command.to_string(),
            total_executions,
            successful_executions,
            failed_executions,
            avg_duration_ms,
            min_duration_ms,
            max_duration_ms,
            success_rate,
            last_execution,
            unique_users,
            most_active_user,
            executions_24h,
            executions_7d,
        }
    }

    /// Get metrics for a specific command
    pub fn get_command_metrics(&self, command: &str) -> Option<CommandMetrics> {
        self.command_metrics.get(command).map(|m| m.clone())
    }

    /// Get metrics for all commands
    pub fn get_all_command_metrics(&self) -> Vec<CommandMetrics> {
        self.command_metrics.iter().map(|entry| entry.value().clone()).collect()
    }

    /// Generate a comprehensive metrics report
    pub fn generate_report(&self, period_start: Option<DateTime<Utc>>, period_end: Option<DateTime<Utc>>) -> MetricsReport {
        let now = Utc::now();
        let period_start = period_start.unwrap_or_else(|| now - chrono::Duration::days(30));
        let period_end = period_end.unwrap_or(now);

        let mut total_executions = 0u64;
        let mut total_successes = 0u64;
        let mut total_duration = 0u64;
        let mut user_counts: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
        let mut channel_counts: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
        let mut error_counts: std::collections::HashMap<String, u64> = std::collections::HashMap::new();

        // Collect metrics from all executions within the period
        for entry in self.executions.iter() {
            for execution in entry.value().iter() {
                if execution.timestamp >= period_start && execution.timestamp <= period_end {
                    total_executions += 1;
                    total_duration += execution.duration_ms;

                    if execution.success {
                        total_successes += 1;
                    } else if let Some(error) = &execution.error {
                        *error_counts.entry(error.clone()).or_insert(0) += 1;
                    }

                    *user_counts.entry(execution.user_id).or_insert(0) += 1;

                    if let Some(channel_id) = execution.channel_id {
                        *channel_counts.entry(channel_id).or_insert(0) += 1;
                    }
                }
            }
        }

        let overall_success_rate = if total_executions > 0 {
            (total_successes as f64 / total_executions as f64) * 100.0
        } else {
            0.0
        };

        let avg_response_time_ms = if total_executions > 0 {
            total_duration as f64 / total_executions as f64
        } else {
            0.0
        };

        // Get command metrics
        let commands = self.get_all_command_metrics();

        // Sort top users and channels
        let mut top_users: Vec<(u64, u64)> = user_counts.into_iter().collect();
        top_users.sort_by(|a, b| b.1.cmp(&a.1));
        top_users.truncate(10); // Top 10 users

        let mut top_channels: Vec<(u64, u64)> = channel_counts.into_iter().collect();
        top_channels.sort_by(|a, b| b.1.cmp(&a.1));
        top_channels.truncate(10); // Top 10 channels

        // Sort error breakdown
        let mut error_breakdown: Vec<(String, u64)> = error_counts.into_iter().collect();
        error_breakdown.sort_by(|a, b| b.1.cmp(&a.1));

        MetricsReport {
            generated_at: now,
            period_start,
            period_end,
            total_executions,
            overall_success_rate,
            avg_response_time_ms,
            commands,
            top_users,
            top_channels,
            error_breakdown,
        }
    }

    /// Get manager uptime
    pub fn get_uptime(&self) -> Duration {
        self.start_time.elapsed()
    }

    /// Get global execution counts
    pub fn get_global_counts(&self) -> (u64, u64, u64) {
        (
            self.total_executions.load(Ordering::Relaxed),
            self.total_successes.load(Ordering::Relaxed),
            self.total_failures.load(Ordering::Relaxed),
        )
    }

    /// Clear old execution history beyond retention period
    pub fn cleanup_old_executions(&self, retention_days: u32) {
        let cutoff = Utc::now() - chrono::Duration::days(retention_days as i64);
        let mut cleaned_count = 0;

        for mut entry in self.executions.iter_mut() {
            let initial_len = entry.len();
            entry.retain(|execution| execution.timestamp > cutoff);
            cleaned_count += initial_len - entry.len();
        }

        if cleaned_count > 0 {
            info!("Cleaned up {} old execution records older than {} days", cleaned_count, retention_days);
        }
    }

    /// Get execution history for a specific command
    pub fn get_command_history(&self, command: &str, limit: Option<usize>) -> Vec<CommandExecution> {
        if let Some(executions) = self.executions.get(command) {
            let mut history = executions.clone();
            history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp)); // Most recent first
            
            if let Some(limit) = limit {
                history.truncate(limit);
            }
            
            history
        } else {
            Vec::new()
        }
    }

    /// Export metrics to JSON string
    pub fn export_metrics(&self) -> Result<String, serde_json::Error> {
        let report = self.generate_report(None, None);
        serde_json::to_string_pretty(&report)
    }

    /// Get real-time metrics summary for monitoring
    pub fn get_realtime_summary(&self) -> serde_json::Value {
        let (total, successes, failures) = self.get_global_counts();
        let uptime = self.get_uptime();
        
        serde_json::json!({
            "uptime_seconds": uptime.as_secs(),
            "total_executions": total,
            "successful_executions": successes,
            "failed_executions": failures,
            "success_rate": if total > 0 { (successes as f64 / total as f64) * 100.0 } else { 0.0 },
            "commands_registered": self.command_metrics.len(),
            "timestamp": Utc::now()
        })
    }
} 