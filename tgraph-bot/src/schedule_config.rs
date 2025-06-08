//! Schedule Configuration Parser and Validator
//! 
//! This module provides robust parsing and validation for schedule configurations
//! from various input sources (JSON, YAML) with comprehensive error handling.

use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono_tz::Tz;
use tracing::{info, debug};
use uuid::Uuid;

use crate::scheduler::{JobId, JobMetadata};

/// Type alias for schedule configuration identifiers
pub type ScheduleId = String;

/// Supported task types for scheduled execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TaskType {
    /// Generate and post graphs automatically
    AutoGraph,
    /// Clean up old data and temporary files
    Cleanup,
    /// Send periodic status reports
    StatusReport,
    /// Backup data to external storage
    DataBackup,
    /// Custom task with user-defined parameters
    Custom(String),
}

impl std::fmt::Display for TaskType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TaskType::AutoGraph => write!(f, "auto_graph"),
            TaskType::Cleanup => write!(f, "cleanup"),
            TaskType::StatusReport => write!(f, "status_report"),
            TaskType::DataBackup => write!(f, "data_backup"),
            TaskType::Custom(name) => write!(f, "custom_{}", name),
        }
    }
}

/// Priority levels for scheduled tasks
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SchedulePriority {
    Low,
    Normal,
    High,
    Critical,
}

impl Default for SchedulePriority {
    fn default() -> Self {
        SchedulePriority::Normal
    }
}

/// Configuration for a single scheduled task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    /// Unique identifier for this schedule
    pub id: ScheduleId,
    /// Human-readable name for the schedule
    pub name: String,
    /// Cron expression for scheduling (6-field format: second minute hour day month weekday)
    pub cron_expression: String,
    /// Type of task to execute
    pub task_type: TaskType,
    /// Priority level for execution
    #[serde(default)]
    pub priority: SchedulePriority,
    /// Whether this schedule is currently enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Optional description of what this schedule does
    pub description: Option<String>,
    /// Timezone for cron expression evaluation (defaults to UTC)
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Task-specific parameters as key-value pairs
    #[serde(default)]
    pub parameters: HashMap<String, serde_json::Value>,
    /// Maximum number of retries on failure
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Timeout for task execution in seconds
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

/// Collection of schedule configurations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfigCollection {
    /// Version of the configuration format
    #[serde(default = "default_version")]
    pub version: String,
    /// Global default settings
    #[serde(default)]
    pub defaults: ScheduleDefaults,
    /// Individual schedule configurations
    pub schedules: Vec<ScheduleConfig>,
}

/// Default settings applied to all schedules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleDefaults {
    /// Default timezone for all schedules
    #[serde(default = "default_timezone")]
    pub timezone: String,
    /// Default priority for all schedules
    #[serde(default)]
    pub priority: SchedulePriority,
    /// Default maximum retries
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Default timeout in seconds
    #[serde(default = "default_timeout_seconds")]
    pub timeout_seconds: u64,
}

impl Default for ScheduleDefaults {
    fn default() -> Self {
        Self {
            timezone: default_timezone(),
            priority: SchedulePriority::default(),
            max_retries: default_max_retries(),
            timeout_seconds: default_timeout_seconds(),
        }
    }
}

// Default value functions for serde
fn default_enabled() -> bool { true }
fn default_timezone() -> String { "UTC".to_string() }
fn default_max_retries() -> u32 { 3 }
fn default_timeout_seconds() -> u64 { 300 } // 5 minutes
fn default_version() -> String { "1.0".to_string() }

/// Custom error types for schedule configuration validation
#[derive(Debug, thiserror::Error)]
pub enum ScheduleConfigError {
    #[error("Invalid cron expression '{expression}': {reason}")]
    InvalidCronExpression { expression: String, reason: String },
    
    #[error("Invalid timezone '{timezone}': {reason}")]
    InvalidTimezone { timezone: String, reason: String },
    
    #[error("Duplicate schedule ID '{id}' found")]
    DuplicateScheduleId { id: String },
    
    #[error("Schedule ID '{id}' is empty or invalid")]
    InvalidScheduleId { id: String },
    
    #[error("Schedule name '{name}' is empty or invalid")]
    InvalidScheduleName { name: String },
    
    #[error("Invalid timeout value: {timeout_seconds} seconds")]
    InvalidTimeout { timeout_seconds: u64 },
    
    #[error("Invalid retry count: {max_retries}")]
    InvalidRetryCount { max_retries: u32 },
    
    #[error("Configuration file parsing error: {0}")]
    ParseError(#[from] serde_yaml::Error),
    
    #[error("JSON parsing error: {0}")]
    JsonParseError(#[from] serde_json::Error),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Schedule configuration parser and validator
pub struct ScheduleConfigParser {
    /// Whether to apply strict validation rules
    strict_validation: bool,
}

impl ScheduleConfigParser {
    /// Create a new parser with default settings
    pub fn new() -> Self {
        Self {
            strict_validation: true,
        }
    }
    
    /// Create a new parser with custom validation settings
    pub fn with_strict_validation(strict: bool) -> Self {
        Self {
            strict_validation: strict,
        }
    }
}

impl Default for ScheduleConfigParser {
    fn default() -> Self {
        Self::new()
    }
}

impl ScheduleConfigParser {
    /// Parse schedule configuration from a YAML file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the YAML configuration file
    ///
    /// # Returns
    ///
    /// A validated `ScheduleConfigCollection`
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed, or if validation fails
    pub async fn parse_yaml_file<P: AsRef<Path>>(&self, path: P) -> Result<ScheduleConfigCollection, ScheduleConfigError> {
        let path = path.as_ref();
        info!("Parsing schedule configuration from YAML file: {}", path.display());

        let content = tokio::fs::read_to_string(path).await?;
        self.parse_yaml_string(&content).await
    }

    /// Parse schedule configuration from a YAML string
    ///
    /// # Arguments
    ///
    /// * `yaml_content` - YAML content as a string
    ///
    /// # Returns
    ///
    /// A validated `ScheduleConfigCollection`
    pub async fn parse_yaml_string(&self, yaml_content: &str) -> Result<ScheduleConfigCollection, ScheduleConfigError> {
        debug!("Parsing YAML content ({} bytes)", yaml_content.len());

        let mut config: ScheduleConfigCollection = serde_yaml::from_str(yaml_content)?;
        self.apply_defaults_and_validate(&mut config).await?;

        info!("Successfully parsed {} schedules from YAML", config.schedules.len());
        Ok(config)
    }

    /// Parse schedule configuration from a JSON file
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the JSON configuration file
    ///
    /// # Returns
    ///
    /// A validated `ScheduleConfigCollection`
    pub async fn parse_json_file<P: AsRef<Path>>(&self, path: P) -> Result<ScheduleConfigCollection, ScheduleConfigError> {
        let path = path.as_ref();
        info!("Parsing schedule configuration from JSON file: {}", path.display());

        let content = tokio::fs::read_to_string(path).await?;
        self.parse_json_string(&content).await
    }

    /// Parse schedule configuration from a JSON string
    ///
    /// # Arguments
    ///
    /// * `json_content` - JSON content as a string
    ///
    /// # Returns
    ///
    /// A validated `ScheduleConfigCollection`
    pub async fn parse_json_string(&self, json_content: &str) -> Result<ScheduleConfigCollection, ScheduleConfigError> {
        debug!("Parsing JSON content ({} bytes)", json_content.len());

        let mut config: ScheduleConfigCollection = serde_json::from_str(json_content)?;
        self.apply_defaults_and_validate(&mut config).await?;

        info!("Successfully parsed {} schedules from JSON", config.schedules.len());
        Ok(config)
    }

    /// Apply default values and validate the configuration
    async fn apply_defaults_and_validate(&self, config: &mut ScheduleConfigCollection) -> Result<(), ScheduleConfigError> {
        debug!("Applying defaults and validating configuration");

        // Apply defaults to individual schedules
        for schedule in &mut config.schedules {
            self.apply_schedule_defaults(schedule, &config.defaults);
        }

        // Validate the entire configuration
        self.validate_config(config).await?;

        debug!("Configuration validation completed successfully");
        Ok(())
    }

    /// Apply default values to a schedule configuration
    fn apply_schedule_defaults(&self, schedule: &mut ScheduleConfig, defaults: &ScheduleDefaults) {
        // Apply timezone default if not set or empty
        if schedule.timezone.is_empty() {
            schedule.timezone = defaults.timezone.clone();
        }

        // Apply other defaults if they match the default values (indicating they weren't explicitly set)
        if schedule.max_retries == default_max_retries() {
            schedule.max_retries = defaults.max_retries;
        }

        if schedule.timeout_seconds == default_timeout_seconds() {
            schedule.timeout_seconds = defaults.timeout_seconds;
        }
    }

    /// Validate the entire configuration collection
    async fn validate_config(&self, config: &ScheduleConfigCollection) -> Result<(), ScheduleConfigError> {
        debug!("Validating configuration with {} schedules", config.schedules.len());

        // Check for duplicate schedule IDs
        let mut seen_ids = std::collections::HashSet::new();

        for schedule in &config.schedules {
            // Validate individual schedule
            self.validate_schedule(schedule).await?;

            // Check for duplicate IDs
            if !seen_ids.insert(&schedule.id) {
                return Err(ScheduleConfigError::DuplicateScheduleId {
                    id: schedule.id.clone(),
                });
            }
        }

        info!("Configuration validation passed for {} schedules", config.schedules.len());
        Ok(())
    }

    /// Validate a single schedule configuration
    async fn validate_schedule(&self, schedule: &ScheduleConfig) -> Result<(), ScheduleConfigError> {
        // Validate schedule ID
        if schedule.id.trim().is_empty() {
            return Err(ScheduleConfigError::InvalidScheduleId {
                id: schedule.id.clone(),
            });
        }

        // Validate schedule name
        if schedule.name.trim().is_empty() {
            return Err(ScheduleConfigError::InvalidScheduleName {
                name: schedule.name.clone(),
            });
        }

        // Validate cron expression
        self.validate_cron_expression(&schedule.cron_expression)?;

        // Validate timezone
        self.validate_timezone(&schedule.timezone)?;

        // Validate timeout
        if self.strict_validation && schedule.timeout_seconds == 0 {
            return Err(ScheduleConfigError::InvalidTimeout {
                timeout_seconds: schedule.timeout_seconds,
            });
        }

        // Validate retry count
        if self.strict_validation && schedule.max_retries > 10 {
            return Err(ScheduleConfigError::InvalidRetryCount {
                max_retries: schedule.max_retries,
            });
        }

        debug!("Schedule '{}' validation passed", schedule.name);
        Ok(())
    }

    /// Validate a cron expression using tokio-cron-scheduler format
    ///
    /// # Arguments
    ///
    /// * `cron_expr` - The cron expression to validate (6-field format)
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, error otherwise
    pub fn validate_cron_expression(&self, cron_expr: &str) -> Result<(), ScheduleConfigError> {
        if cron_expr.trim().is_empty() {
            return Err(ScheduleConfigError::InvalidCronExpression {
                expression: cron_expr.to_string(),
                reason: "Cron expression cannot be empty".to_string(),
            });
        }

        // Validate using tokio-cron-scheduler's format (6 fields)
        // We'll try to create a job to validate the expression
        match tokio_cron_scheduler::Job::new(cron_expr, |_uuid, _scheduler| {}) {
            Ok(_) => {
                debug!("Cron expression '{}' is valid", cron_expr);
                Ok(())
            }
            Err(e) => Err(ScheduleConfigError::InvalidCronExpression {
                expression: cron_expr.to_string(),
                reason: format!("tokio-cron-scheduler validation failed: {}", e),
            }),
        }
    }

    /// Validate a timezone string
    ///
    /// # Arguments
    ///
    /// * `timezone` - The timezone string to validate (IANA format)
    ///
    /// # Returns
    ///
    /// Ok(()) if valid, error otherwise
    pub fn validate_timezone(&self, timezone: &str) -> Result<(), ScheduleConfigError> {
        if timezone.trim().is_empty() {
            return Err(ScheduleConfigError::InvalidTimezone {
                timezone: timezone.to_string(),
                reason: "Timezone cannot be empty".to_string(),
            });
        }

        // Special case for UTC
        if timezone == "UTC" {
            return Ok(());
        }

        // Validate using chrono-tz
        match timezone.parse::<Tz>() {
            Ok(_) => {
                debug!("Timezone '{}' is valid", timezone);
                Ok(())
            }
            Err(e) => Err(ScheduleConfigError::InvalidTimezone {
                timezone: timezone.to_string(),
                reason: format!("Invalid IANA timezone: {}", e),
            }),
        }
    }

    /// Convert a ScheduleConfig to JobMetadata for the scheduler
    ///
    /// # Arguments
    ///
    /// * `schedule` - The schedule configuration to convert
    /// * `job_id` - Optional job ID to use (generates new UUID if None)
    ///
    /// # Returns
    ///
    /// JobMetadata that can be used with SchedulerService
    pub fn to_job_metadata(&self, schedule: &ScheduleConfig, job_id: Option<JobId>) -> JobMetadata {
        let id = job_id.unwrap_or_else(|| Uuid::new_v4());

        JobMetadata {
            id,
            name: schedule.name.clone(),
            cron_expression: schedule.cron_expression.clone(),
            description: schedule.description.clone(),
            enabled: schedule.enabled,
        }
    }

    /// Convert multiple ScheduleConfigs to JobMetadata vector
    ///
    /// # Arguments
    ///
    /// * `schedules` - Vector of schedule configurations to convert
    ///
    /// # Returns
    ///
    /// Vector of JobMetadata that can be used with SchedulerService
    pub fn to_job_metadata_vec(&self, schedules: &[ScheduleConfig]) -> Vec<JobMetadata> {
        schedules
            .iter()
            .filter(|schedule| schedule.enabled) // Only include enabled schedules
            .map(|schedule| self.to_job_metadata(schedule, None))
            .collect()
    }

    /// Create a schedule configuration from JobMetadata
    ///
    /// # Arguments
    ///
    /// * `job_metadata` - The job metadata to convert
    /// * `task_type` - The type of task this schedule represents
    ///
    /// # Returns
    ///
    /// ScheduleConfig created from the job metadata
    pub fn from_job_metadata(&self, job_metadata: &JobMetadata, task_type: TaskType) -> ScheduleConfig {
        ScheduleConfig {
            id: job_metadata.id.to_string(),
            name: job_metadata.name.clone(),
            cron_expression: job_metadata.cron_expression.clone(),
            task_type,
            priority: SchedulePriority::default(),
            enabled: job_metadata.enabled,
            description: job_metadata.description.clone(),
            timezone: default_timezone(),
            parameters: HashMap::new(),
            max_retries: default_max_retries(),
            timeout_seconds: default_timeout_seconds(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tempfile::NamedTempFile;
    use tokio::io::AsyncWriteExt;

    fn create_test_schedule() -> ScheduleConfig {
        ScheduleConfig {
            id: "test_schedule_1".to_string(),
            name: "Test Schedule".to_string(),
            cron_expression: "0 0 12 * * *".to_string(), // Daily at noon
            task_type: TaskType::AutoGraph,
            priority: SchedulePriority::Normal,
            enabled: true,
            description: Some("Test schedule description".to_string()),
            timezone: "UTC".to_string(),
            parameters: HashMap::new(),
            max_retries: 3,
            timeout_seconds: 300,
        }
    }

    fn create_test_config_collection() -> ScheduleConfigCollection {
        ScheduleConfigCollection {
            version: "1.0".to_string(),
            defaults: ScheduleDefaults::default(),
            schedules: vec![create_test_schedule()],
        }
    }

    #[tokio::test]
    async fn test_schedule_config_creation() {
        let schedule = create_test_schedule();

        assert_eq!(schedule.id, "test_schedule_1");
        assert_eq!(schedule.name, "Test Schedule");
        assert_eq!(schedule.cron_expression, "0 0 12 * * *");
        assert_eq!(schedule.task_type, TaskType::AutoGraph);
        assert_eq!(schedule.priority, SchedulePriority::Normal);
        assert!(schedule.enabled);
        assert_eq!(schedule.timezone, "UTC");
        assert_eq!(schedule.max_retries, 3);
        assert_eq!(schedule.timeout_seconds, 300);
    }

    #[tokio::test]
    async fn test_task_type_display() {
        assert_eq!(TaskType::AutoGraph.to_string(), "auto_graph");
        assert_eq!(TaskType::Cleanup.to_string(), "cleanup");
        assert_eq!(TaskType::StatusReport.to_string(), "status_report");
        assert_eq!(TaskType::DataBackup.to_string(), "data_backup");
        assert_eq!(TaskType::Custom("test".to_string()).to_string(), "custom_test");
    }

    #[tokio::test]
    async fn test_cron_expression_validation() {
        let parser = ScheduleConfigParser::new();

        // Valid cron expressions (6-field format for tokio-cron-scheduler)
        assert!(parser.validate_cron_expression("0 0 12 * * *").is_ok()); // Daily at noon
        assert!(parser.validate_cron_expression("0 */15 * * * *").is_ok()); // Every 15 minutes
        assert!(parser.validate_cron_expression("0 0 0 1 * *").is_ok()); // First day of month

        // Invalid cron expressions
        assert!(parser.validate_cron_expression("").is_err()); // Empty
        assert!(parser.validate_cron_expression("invalid").is_err()); // Invalid format
        assert!(parser.validate_cron_expression("0 0 25 * * *").is_err()); // Invalid hour
    }

    #[tokio::test]
    async fn test_timezone_validation() {
        let parser = ScheduleConfigParser::new();

        // Valid timezones
        assert!(parser.validate_timezone("UTC").is_ok());
        assert!(parser.validate_timezone("America/New_York").is_ok());
        assert!(parser.validate_timezone("Europe/London").is_ok());
        assert!(parser.validate_timezone("Asia/Tokyo").is_ok());

        // Invalid timezones
        assert!(parser.validate_timezone("").is_err()); // Empty
        assert!(parser.validate_timezone("Invalid/Timezone").is_err()); // Invalid
        assert!(parser.validate_timezone("Not_A_Timezone").is_err()); // Invalid format
    }

    #[tokio::test]
    async fn test_schedule_validation() {
        let parser = ScheduleConfigParser::new();
        let mut schedule = create_test_schedule();

        // Valid schedule should pass
        assert!(parser.validate_schedule(&schedule).await.is_ok());

        // Invalid schedule ID
        schedule.id = "".to_string();
        assert!(parser.validate_schedule(&schedule).await.is_err());
        schedule.id = "test_schedule_1".to_string(); // Reset

        // Invalid schedule name
        schedule.name = "".to_string();
        assert!(parser.validate_schedule(&schedule).await.is_err());
        schedule.name = "Test Schedule".to_string(); // Reset

        // Invalid cron expression
        schedule.cron_expression = "invalid".to_string();
        assert!(parser.validate_schedule(&schedule).await.is_err());
        schedule.cron_expression = "0 0 12 * * *".to_string(); // Reset

        // Invalid timezone
        schedule.timezone = "Invalid/Timezone".to_string();
        assert!(parser.validate_schedule(&schedule).await.is_err());
    }

    #[tokio::test]
    async fn test_yaml_parsing() {
        let yaml_content = r#"
version: "1.0"
defaults:
  timezone: "UTC"
  priority: "normal"
  max_retries: 3
  timeout_seconds: 300
schedules:
  - id: "auto_graph_daily"
    name: "Daily Auto Graph"
    cron_expression: "0 0 12 * * *"
    task_type: "auto_graph"
    priority: "high"
    enabled: true
    description: "Generate daily graphs automatically"
    timezone: "America/New_York"
    parameters:
      graph_type: "daily_stats"
      include_trends: true
    max_retries: 5
    timeout_seconds: 600
"#;

        let parser = ScheduleConfigParser::new();
        let result = parser.parse_yaml_string(yaml_content).await;

        assert!(result.is_ok());
        let config = result.unwrap();

        assert_eq!(config.version, "1.0");
        assert_eq!(config.schedules.len(), 1);

        let schedule = &config.schedules[0];
        assert_eq!(schedule.id, "auto_graph_daily");
        assert_eq!(schedule.name, "Daily Auto Graph");
        assert_eq!(schedule.cron_expression, "0 0 12 * * *");
        assert_eq!(schedule.task_type, TaskType::AutoGraph);
        assert_eq!(schedule.priority, SchedulePriority::High);
        assert!(schedule.enabled);
        assert_eq!(schedule.timezone, "America/New_York");
        assert_eq!(schedule.max_retries, 5);
        assert_eq!(schedule.timeout_seconds, 600);
    }

    #[tokio::test]
    async fn test_json_parsing() {
        let json_content = r#"
{
  "version": "1.0",
  "defaults": {
    "timezone": "UTC",
    "priority": "normal",
    "max_retries": 3,
    "timeout_seconds": 300
  },
  "schedules": [
    {
      "id": "cleanup_weekly",
      "name": "Weekly Cleanup",
      "cron_expression": "0 0 2 * * 7",
      "task_type": "cleanup",
      "priority": "low",
      "enabled": true,
      "description": "Weekly cleanup of old data",
      "timezone": "UTC",
      "parameters": {
        "retention_days": 30,
        "cleanup_temp_files": true
      },
      "max_retries": 2,
      "timeout_seconds": 1800
    }
  ]
}
"#;

        let parser = ScheduleConfigParser::new();
        let result = parser.parse_json_string(json_content).await;

        if let Err(ref e) = result {
            eprintln!("JSON parsing failed: {:?}", e);
        }
        assert!(result.is_ok());
        let config = result.unwrap();

        assert_eq!(config.schedules.len(), 1);

        let schedule = &config.schedules[0];
        assert_eq!(schedule.id, "cleanup_weekly");
        assert_eq!(schedule.task_type, TaskType::Cleanup);
        assert_eq!(schedule.priority, SchedulePriority::Low);
        assert_eq!(schedule.max_retries, 2);
        assert_eq!(schedule.timeout_seconds, 1800);
    }

    #[tokio::test]
    async fn test_duplicate_schedule_id_validation() {
        let yaml_content = r#"
version: "1.0"
schedules:
  - id: "duplicate_id"
    name: "First Schedule"
    cron_expression: "0 0 12 * * *"
    task_type: "auto_graph"
  - id: "duplicate_id"
    name: "Second Schedule"
    cron_expression: "0 0 18 * * *"
    task_type: "cleanup"
"#;

        let parser = ScheduleConfigParser::new();
        let result = parser.parse_yaml_string(yaml_content).await;

        assert!(result.is_err());
        match result.unwrap_err() {
            ScheduleConfigError::DuplicateScheduleId { id } => {
                assert_eq!(id, "duplicate_id");
            }
            _ => panic!("Expected DuplicateScheduleId error"),
        }
    }

    #[tokio::test]
    async fn test_job_metadata_conversion() {
        let parser = ScheduleConfigParser::new();
        let schedule = create_test_schedule();

        // Convert to JobMetadata
        let job_metadata = parser.to_job_metadata(&schedule, None);

        assert_eq!(job_metadata.name, schedule.name);
        assert_eq!(job_metadata.cron_expression, schedule.cron_expression);
        assert_eq!(job_metadata.description, schedule.description);
        assert_eq!(job_metadata.enabled, schedule.enabled);

        // Convert back to ScheduleConfig
        let converted_schedule = parser.from_job_metadata(&job_metadata, TaskType::AutoGraph);

        assert_eq!(converted_schedule.name, schedule.name);
        assert_eq!(converted_schedule.cron_expression, schedule.cron_expression);
        assert_eq!(converted_schedule.description, schedule.description);
        assert_eq!(converted_schedule.enabled, schedule.enabled);
        assert_eq!(converted_schedule.task_type, TaskType::AutoGraph);
    }

    #[tokio::test]
    async fn test_job_metadata_vec_conversion() {
        let parser = ScheduleConfigParser::new();
        let mut schedules = vec![create_test_schedule()];

        // Add a disabled schedule
        let mut disabled_schedule = create_test_schedule();
        disabled_schedule.id = "disabled_schedule".to_string();
        disabled_schedule.enabled = false;
        schedules.push(disabled_schedule);

        // Convert to JobMetadata vector (should only include enabled schedules)
        let job_metadata_vec = parser.to_job_metadata_vec(&schedules);

        assert_eq!(job_metadata_vec.len(), 1); // Only enabled schedule
        assert_eq!(job_metadata_vec[0].name, "Test Schedule");
        assert!(job_metadata_vec[0].enabled);
    }

    #[tokio::test]
    async fn test_file_parsing() {
        let parser = ScheduleConfigParser::new();
        let config = create_test_config_collection();

        // Test YAML file parsing
        let yaml_content = serde_yaml::to_string(&config).unwrap();
        let yaml_file = NamedTempFile::new().unwrap();
        let mut yaml_file_async = tokio::fs::File::create(yaml_file.path()).await.unwrap();
        yaml_file_async.write_all(yaml_content.as_bytes()).await.unwrap();
        yaml_file_async.flush().await.unwrap();

        let yaml_result = parser.parse_yaml_file(yaml_file.path()).await;
        assert!(yaml_result.is_ok());

        // Test JSON file parsing
        let json_content = serde_json::to_string_pretty(&config).unwrap();
        let json_file = NamedTempFile::new().unwrap();
        let mut json_file_async = tokio::fs::File::create(json_file.path()).await.unwrap();
        json_file_async.write_all(json_content.as_bytes()).await.unwrap();
        json_file_async.flush().await.unwrap();

        let json_result = parser.parse_json_file(json_file.path()).await;
        assert!(json_result.is_ok());
    }

    #[tokio::test]
    async fn test_strict_validation() {
        let strict_parser = ScheduleConfigParser::with_strict_validation(true);
        let lenient_parser = ScheduleConfigParser::with_strict_validation(false);

        let mut schedule = create_test_schedule();

        // Test timeout validation
        schedule.timeout_seconds = 0;
        assert!(strict_parser.validate_schedule(&schedule).await.is_err());
        assert!(lenient_parser.validate_schedule(&schedule).await.is_ok());

        // Test retry count validation
        schedule.timeout_seconds = 300; // Reset
        schedule.max_retries = 15; // Too many retries
        assert!(strict_parser.validate_schedule(&schedule).await.is_err());
        assert!(lenient_parser.validate_schedule(&schedule).await.is_ok());
    }

    #[tokio::test]
    async fn test_custom_task_type() {
        let custom_task = TaskType::Custom("my_custom_task".to_string());
        assert_eq!(custom_task.to_string(), "custom_my_custom_task");

        // Test with JSON format which is easier to control for custom enum variants
        let json_content = r#"
{
  "version": "1.0",
  "schedules": [
    {
      "id": "custom_task",
      "name": "Custom Task",
      "cron_expression": "0 0 * * * *",
      "task_type": {
        "custom": "backup_database"
      }
    }
  ]
}
"#;

        let parser = ScheduleConfigParser::new();
        let result = parser.parse_json_string(json_content).await;

        if let Err(ref e) = result {
            eprintln!("Custom task type parsing failed: {:?}", e);
        }
        assert!(result.is_ok());
        let config = result.unwrap();

        match &config.schedules[0].task_type {
            TaskType::Custom(name) => assert_eq!(name, "backup_database"),
            _ => panic!("Expected Custom task type"),
        }
    }
}
