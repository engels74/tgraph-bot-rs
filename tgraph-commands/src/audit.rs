//! Audit logging for GDPR compliance and data protection

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};
use std::collections::VecDeque;

/// Types of auditable events
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum AuditEventType {
    /// User data was exported
    DataExport,
    /// User data was deleted
    DataDeletion,
    /// User preferences were accessed
    PreferencesAccess,
    /// User preferences were modified
    PreferencesModification,
    /// User statistics were accessed
    StatisticsAccess,
    /// User command history was accessed
    CommandHistoryAccess,
    /// Privacy settings were changed
    PrivacySettingsChange,
    /// Data retention policy was applied
    DataRetentionApplied,
    /// GDPR request was processed
    GdprRequest,
}

impl AuditEventType {
    /// Get a human-readable description of the event type
    pub fn description(&self) -> &'static str {
        match self {
            AuditEventType::DataExport => "User data export requested",
            AuditEventType::DataDeletion => "User data deletion requested",
            AuditEventType::PreferencesAccess => "User preferences accessed",
            AuditEventType::PreferencesModification => "User preferences modified",
            AuditEventType::StatisticsAccess => "User statistics accessed",
            AuditEventType::CommandHistoryAccess => "User command history accessed",
            AuditEventType::PrivacySettingsChange => "Privacy settings changed",
            AuditEventType::DataRetentionApplied => "Data retention policy applied",
            AuditEventType::GdprRequest => "GDPR request processed",
        }
    }
}

/// An individual audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Unique ID for this log entry
    pub id: String,
    /// When the event occurred
    pub timestamp: DateTime<Utc>,
    /// Type of event
    pub event_type: AuditEventType,
    /// User ID affected (if applicable)
    pub user_id: Option<u64>,
    /// User who performed the action (if different from affected user)
    pub actor_user_id: Option<u64>,
    /// Additional details about the event
    pub details: String,
    /// Source of the event (e.g., "discord_command", "api_call", "scheduled_task")
    pub source: String,
    /// Success or failure status
    pub success: bool,
    /// Error message if the event failed
    pub error_message: Option<String>,
    /// Additional metadata as JSON
    pub metadata: serde_json::Value,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        user_id: Option<u64>,
        actor_user_id: Option<u64>,
        details: String,
        source: String,
    ) -> Self {
        let id = uuid::Uuid::new_v4().to_string();
        Self {
            id,
            timestamp: Utc::now(),
            event_type,
            user_id,
            actor_user_id,
            details,
            source,
            success: true,
            error_message: None,
            metadata: serde_json::Value::Null,
        }
    }

    /// Mark this entry as failed with an error message
    pub fn with_error(mut self, error_message: String) -> Self {
        self.success = false;
        self.error_message = Some(error_message);
        self
    }

    /// Add metadata to this entry
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = metadata;
        self
    }
}

/// Audit logger for tracking data protection events
#[derive(Debug)]
pub struct AuditLogger {
    /// In-memory log entries (recent events only)
    entries: Arc<RwLock<VecDeque<AuditLogEntry>>>,
    /// Maximum number of entries to keep in memory
    max_entries: usize,
}

impl AuditLogger {
    /// Create a new audit logger
    pub fn new(max_entries: usize) -> Self {
        Self {
            entries: Arc::new(RwLock::new(VecDeque::with_capacity(max_entries))),
            max_entries,
        }
    }

    /// Log an audit event
    pub async fn log_event(&self, entry: AuditLogEntry) {
        // Log to tracing system for external logging integration
        if entry.success {
            info!(
                event_type = ?entry.event_type,
                user_id = entry.user_id,
                actor_user_id = entry.actor_user_id,
                source = %entry.source,
                details = %entry.details,
                "Audit event: {}",
                entry.event_type.description()
            );
        } else {
            warn!(
                event_type = ?entry.event_type,
                user_id = entry.user_id,
                actor_user_id = entry.actor_user_id,
                source = %entry.source,
                details = %entry.details,
                error = entry.error_message.as_deref().unwrap_or("Unknown error"),
                "Failed audit event: {}",
                entry.event_type.description()
            );
        }

        // Store in memory (with size limit)
        let mut entries = self.entries.write().await;
        
        // Remove oldest entries if we're at capacity
        while entries.len() >= self.max_entries {
            entries.pop_front();
        }
        
        entries.push_back(entry);
    }

    /// Get recent audit entries
    pub async fn get_recent_entries(&self, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let limit = limit.unwrap_or(entries.len());
        
        entries
            .iter()
            .rev() // Most recent first
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit entries for a specific user
    pub async fn get_user_entries(&self, user_id: u64, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let limit = limit.unwrap_or(entries.len());
        
        entries
            .iter()
            .rev() // Most recent first
            .filter(|entry| entry.user_id == Some(user_id) || entry.actor_user_id == Some(user_id))
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit entries by event type
    pub async fn get_entries_by_type(&self, event_type: AuditEventType, limit: Option<usize>) -> Vec<AuditLogEntry> {
        let entries = self.entries.read().await;
        let limit = limit.unwrap_or(entries.len());
        
        entries
            .iter()
            .rev() // Most recent first
            .filter(|entry| entry.event_type == event_type)
            .take(limit)
            .cloned()
            .collect()
    }

    /// Get audit statistics
    pub async fn get_statistics(&self) -> AuditStatistics {
        let entries = self.entries.read().await;
        
        let total_entries = entries.len();
        let successful_entries = entries.iter().filter(|e| e.success).count();
        let failed_entries = total_entries - successful_entries;
        
        let mut event_type_counts = std::collections::HashMap::new();
        for entry in entries.iter() {
            *event_type_counts.entry(entry.event_type.clone()).or_insert(0) += 1;
        }
        
        let oldest_entry = entries.front().map(|e| e.timestamp);
        let newest_entry = entries.back().map(|e| e.timestamp);
        
        AuditStatistics {
            total_entries,
            successful_entries,
            failed_entries,
            event_type_counts,
            oldest_entry,
            newest_entry,
            max_entries: self.max_entries,
        }
    }

    /// Export audit log for a specific user (GDPR compliance)
    pub async fn export_user_audit_log(&self, user_id: u64) -> serde_json::Value {
        let user_entries = self.get_user_entries(user_id, None).await;
        
        serde_json::json!({
            "export_timestamp": Utc::now().to_rfc3339(),
            "user_id": user_id,
            "total_entries": user_entries.len(),
            "entries": user_entries
        })
    }

    /// Clear audit entries for a specific user (GDPR compliance)
    pub async fn clear_user_entries(&self, user_id: u64) -> usize {
        let mut entries = self.entries.write().await;
        let initial_count = entries.len();
        
        entries.retain(|entry| {
            entry.user_id != Some(user_id) && entry.actor_user_id != Some(user_id)
        });
        
        let removed_count = initial_count - entries.len();
        
        if removed_count > 0 {
            info!("Cleared {} audit entries for user {} (GDPR compliance)", removed_count, user_id);
        }
        
        removed_count
    }
}

/// Audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditStatistics {
    /// Total number of entries in the log
    pub total_entries: usize,
    /// Number of successful operations
    pub successful_entries: usize,
    /// Number of failed operations
    pub failed_entries: usize,
    /// Count of each event type
    pub event_type_counts: std::collections::HashMap<AuditEventType, usize>,
    /// Timestamp of oldest entry
    pub oldest_entry: Option<DateTime<Utc>>,
    /// Timestamp of newest entry
    pub newest_entry: Option<DateTime<Utc>>,
    /// Maximum entries that can be stored
    pub max_entries: usize,
}

/// Convenience functions for common audit events
impl AuditLogger {
    /// Log a data export event
    pub async fn log_data_export(&self, user_id: u64, actor_user_id: Option<u64>, export_type: &str) {
        let entry = AuditLogEntry::new(
            AuditEventType::DataExport,
            Some(user_id),
            actor_user_id,
            format!("Data export requested for user {}: {}", user_id, export_type),
            "discord_command".to_string(),
        );
        self.log_event(entry).await;
    }

    /// Log a data deletion event
    pub async fn log_data_deletion(&self, user_id: u64, actor_user_id: Option<u64>, deletion_details: &str) {
        let entry = AuditLogEntry::new(
            AuditEventType::DataDeletion,
            Some(user_id),
            actor_user_id,
            format!("Data deletion requested for user {}: {}", user_id, deletion_details),
            "discord_command".to_string(),
        );
        self.log_event(entry).await;
    }

    /// Log a preferences access event
    pub async fn log_preferences_access(&self, user_id: u64, actor_user_id: Option<u64>) {
        let entry = AuditLogEntry::new(
            AuditEventType::PreferencesAccess,
            Some(user_id),
            actor_user_id,
            format!("User preferences accessed for user {}", user_id),
            "system".to_string(),
        );
        self.log_event(entry).await;
    }

    /// Log a preferences modification event
    pub async fn log_preferences_modification(&self, user_id: u64, actor_user_id: Option<u64>, changes: &str) {
        let entry = AuditLogEntry::new(
            AuditEventType::PreferencesModification,
            Some(user_id),
            actor_user_id,
            format!("User preferences modified for user {}: {}", user_id, changes),
            "system".to_string(),
        );
        self.log_event(entry).await;
    }

    /// Log a statistics access event
    pub async fn log_statistics_access(&self, user_id: u64, actor_user_id: Option<u64>, access_type: &str) {
        let entry = AuditLogEntry::new(
            AuditEventType::StatisticsAccess,
            Some(user_id),
            actor_user_id,
            format!("User statistics accessed for user {}: {}", user_id, access_type),
            "discord_command".to_string(),
        );
        self.log_event(entry).await;
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000) // Default to 10,000 entries
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_audit_logger_basic_functionality() {
        let logger = AuditLogger::new(5);
        
        // Log some events
        logger.log_data_export(123, Some(456), "complete_user_data").await;
        logger.log_preferences_access(123, None).await;
        logger.log_data_deletion(789, Some(789), "user_requested").await;
        
        // Check entries
        let entries = logger.get_recent_entries(None).await;
        assert_eq!(entries.len(), 3);
        
        // Check user-specific entries
        let user_entries = logger.get_user_entries(123, None).await;
        assert_eq!(user_entries.len(), 2);
        
        // Check statistics
        let stats = logger.get_statistics().await;
        assert_eq!(stats.total_entries, 3);
        assert_eq!(stats.successful_entries, 3);
        assert_eq!(stats.failed_entries, 0);
    }

    #[tokio::test]
    async fn test_audit_logger_capacity_limit() {
        let logger = AuditLogger::new(2); // Small capacity for testing
        
        // Add more entries than capacity
        logger.log_data_export(1, None, "test1").await;
        logger.log_data_export(2, None, "test2").await;
        logger.log_data_export(3, None, "test3").await;
        
        // Should only keep the most recent 2
        let entries = logger.get_recent_entries(None).await;
        assert_eq!(entries.len(), 2);
        
        // Should be the most recent entries (3 and 2)
        assert!(entries[0].details.contains("test3"));
        assert!(entries[1].details.contains("test2"));
    }

    #[tokio::test]
    async fn test_clear_user_entries() {
        let logger = AuditLogger::new(10);
        
        // Add entries for different users
        logger.log_data_export(123, None, "test").await;
        logger.log_data_export(456, None, "test").await;
        logger.log_preferences_access(123, Some(789)).await;
        
        // Clear entries for user 123
        let removed = logger.clear_user_entries(123).await;
        assert_eq!(removed, 2); // Should remove 2 entries involving user 123
        
        // Check remaining entries
        let entries = logger.get_recent_entries(None).await;
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].user_id, Some(456));
    }

    #[tokio::test]
    async fn test_export_user_audit_log() {
        let logger = AuditLogger::new(10);
        
        // Add some entries for a user
        logger.log_data_export(123, None, "test_export").await;
        logger.log_preferences_modification(123, Some(123), "privacy_settings").await;
        
        // Export the audit log
        let export = logger.export_user_audit_log(123).await;
        
        // Verify export structure
        assert_eq!(export["user_id"], 123);
        assert_eq!(export["total_entries"], 2);
        assert!(export["entries"].is_array());
        assert!(export["export_timestamp"].is_string());
    }
} 