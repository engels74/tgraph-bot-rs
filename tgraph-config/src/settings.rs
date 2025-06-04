//! Application configuration structures

use serde::{Deserialize, Serialize};
use validator::Validate;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Config {
    /// Discord-related configuration
    pub discord: DiscordConfig,
    
    /// Tautulli-related configuration  
    pub tautulli: TautulliConfig,
    
    /// Scheduling configuration
    pub scheduling: SchedulingConfig,
    
    /// Graph rendering settings
    pub graph: GraphConfig,
    
    /// Database configuration
    pub database: DatabaseConfig,
    
    /// Logging configuration
    pub logging: LoggingConfig,
}

/// Discord bot configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DiscordConfig {
    /// Discord bot token
    #[validate(length(min = 1, message = "Discord token cannot be empty"))]
    #[validate(custom(function = "crate::validation::validate_discord_token", message = "Invalid Discord token format"))]
    pub token: String,
    
    /// List of allowed channel IDs where the bot can operate
    pub channels: Vec<String>,
    
    /// Maximum number of concurrent requests to Discord API
    #[validate(range(min = 1, max = 100, message = "Concurrent requests must be between 1 and 100"))]
    pub max_concurrent_requests: u32,
    
    /// Request timeout in seconds
    #[validate(range(min = 1, max = 300, message = "Timeout must be between 1 and 300 seconds"))]
    pub request_timeout_seconds: u64,
}

/// Tautulli API configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct TautulliConfig {
    /// Tautulli server base URL
    #[validate(url(message = "Tautulli URL must be a valid URL"))]
    pub url: String,
    
    /// Tautulli API key
    #[validate(length(min = 1, message = "Tautulli API key cannot be empty"))]
    pub api_key: String,
    
    /// Request timeout in seconds
    #[validate(range(min = 1, max = 300, message = "Timeout must be between 1 and 300 seconds"))]
    pub timeout_seconds: u64,
    
    /// Maximum number of retries for failed requests
    #[validate(range(max = 10, message = "Max retries cannot exceed 10"))]
    pub max_retries: u32,
}

/// Scheduling configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SchedulingConfig {
    /// Cron expression for automatic graph generation
    /// Example: "0 0 * * *" for daily at midnight
    pub auto_graph_cron: Option<String>,
    
    /// Cron expression for statistics cleanup
    /// Example: "0 2 * * 0" for weekly on Sunday at 2 AM
    pub cleanup_cron: Option<String>,
    
    /// Timezone for cron expressions (IANA timezone name)
    /// Example: "America/New_York" or "UTC"
    #[validate(length(min = 1, message = "Timezone cannot be empty if specified"))]
    pub timezone: Option<String>,
    
    /// Whether scheduling is enabled
    pub enabled: bool,
}

/// Graph rendering configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct GraphConfig {
    /// Graph width in pixels
    #[validate(range(min = 100, max = 4000, message = "Width must be between 100 and 4000 pixels"))]
    pub width: u32,
    
    /// Graph height in pixels  
    #[validate(range(min = 100, max = 4000, message = "Height must be between 100 and 4000 pixels"))]
    pub height: u32,
    
    /// Background color (hex format)
    #[validate(length(equal = 7, message = "Background color must be 7 characters (e.g., #FFFFFF)"))]
    #[validate(regex(path = "crate::validation::HEX_COLOR_REGEX", message = "Background color must be valid hex color"))]
    pub background_color: String,
    
    /// Primary color for graph elements (hex format)
    #[validate(length(equal = 7, message = "Primary color must be 7 characters (e.g., #FF0000)"))]
    #[validate(regex(path = "crate::validation::HEX_COLOR_REGEX", message = "Primary color must be valid hex color"))]
    pub primary_color: String,
    
    /// Secondary color for graph elements (hex format)
    #[validate(length(equal = 7, message = "Secondary color must be 7 characters (e.g., #00FF00)"))]
    #[validate(regex(path = "crate::validation::HEX_COLOR_REGEX", message = "Secondary color must be valid hex color"))]
    pub secondary_color: String,
    
    /// Font family for text rendering
    pub font_family: String,
    
    /// Font size for labels
    #[validate(range(min = 8, max = 72, message = "Font size must be between 8 and 72"))]
    pub font_size: u32,
    
    /// Whether to show grid lines
    pub show_grid: bool,
    
    /// Whether to show legend
    pub show_legend: bool,
    
    /// Maximum number of data points to display
    #[validate(range(min = 10, max = 10000, message = "Max data points must be between 10 and 10000"))]
    pub max_data_points: u32,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DatabaseConfig {
    /// Database connection URL
    #[validate(length(min = 1, message = "Database URL cannot be empty"))]
    pub url: String,
    
    /// Maximum number of database connections in the pool
    #[validate(range(min = 1, max = 100, message = "Max connections must be between 1 and 100"))]
    pub max_connections: u32,
    
    /// Connection timeout in seconds
    #[validate(range(min = 1, max = 60, message = "Connection timeout must be between 1 and 60 seconds"))]
    pub connection_timeout_seconds: u64,
    
    /// Query timeout in seconds
    #[validate(range(min = 1, max = 300, message = "Query timeout must be between 1 and 300 seconds"))]
    pub query_timeout_seconds: u64,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    #[validate(custom(function = "validate_log_level", message = "Log level must be one of: trace, debug, info, warn, error"))]
    pub level: String,
    
    /// Optional log file path
    pub file: Option<String>,
    
    /// Whether to use colored output (for console logging)
    pub colored: bool,
    
    /// Whether to include timestamps in log output
    pub include_timestamps: bool,
    
    /// Whether to include file/line information in logs
    pub include_location: bool,
    
    /// Maximum size of log files in MB before rotation
    #[validate(range(min = 1, max = 1000, message = "Max log file size must be between 1 and 1000 MB"))]
    pub max_file_size_mb: u32,
    
    /// Number of rotated log files to keep
    #[validate(range(max = 100, message = "Max log files to keep cannot exceed 100"))]
    pub max_files: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            discord: DiscordConfig::default(),
            tautulli: TautulliConfig::default(),
            scheduling: SchedulingConfig::default(),
            graph: GraphConfig::default(),
            database: DatabaseConfig::default(),
            logging: LoggingConfig::default(),
        }
    }
}

impl Config {
    /// Comprehensive validation of the entire configuration
    pub fn validate_all(&self) -> Result<(), validator::ValidationErrors> {
        // First run the standard validator validation
        self.validate()?;
        
        // Then run custom validation for scheduling
        self.scheduling.validate_scheduling()?;
        
        Ok(())
    }
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            channels: Vec::new(),
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
        }
    }
}

impl Default for TautulliConfig {
    fn default() -> Self {
        Self {
            url: String::new(),
            api_key: String::new(),
            timeout_seconds: 30,
            max_retries: 3,
        }
    }
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            auto_graph_cron: None,
            cleanup_cron: None,
            timezone: Some("UTC".to_string()),
            enabled: false,
        }
    }
}

impl SchedulingConfig {
    /// Custom validation for scheduling configuration
    pub fn validate_scheduling(&self) -> Result<(), validator::ValidationErrors> {
        let mut errors = validator::ValidationErrors::new();

        // Validate auto_graph_cron if present
        if let Some(ref cron_expr) = self.auto_graph_cron {
            if let Err(err) = crate::validation::validate_cron_expression(cron_expr) {
                errors.add("auto_graph_cron", err);
            }
        }

        // Validate cleanup_cron if present
        if let Some(ref cron_expr) = self.cleanup_cron {
            if let Err(err) = crate::validation::validate_cron_expression(cron_expr) {
                errors.add("cleanup_cron", err);
            }
        }

        // Validate timezone if present
        if let Some(ref timezone) = self.timezone {
            if let Err(err) = crate::validation::validate_timezone(timezone) {
                errors.add("timezone", err);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for GraphConfig {
    fn default() -> Self {
        Self {
            width: 1200,
            height: 800,
            background_color: "#FFFFFF".to_string(),
            primary_color: "#007ACC".to_string(),
            secondary_color: "#FF6B6B".to_string(),
            font_family: "Arial".to_string(),
            font_size: 12,
            show_grid: true,
            show_legend: true,
            max_data_points: 1000,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: "sqlite://tgraph.db".to_string(),
            max_connections: 10,
            connection_timeout_seconds: 30,
            query_timeout_seconds: 60,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            file: None,
            colored: true,
            include_timestamps: true,
            include_location: false,
            max_file_size_mb: 10,
            max_files: 5,
        }
    }
}

// Custom validation functions
fn validate_log_level(level: &str) -> Result<(), validator::ValidationError> {
    match level {
        "trace" | "debug" | "info" | "warn" | "error" => Ok(()),
        _ => Err(validator::ValidationError::new("invalid_log_level")),
    }
}



// Re-export for backward compatibility  
pub use Config as AppConfig;

#[cfg(test)]
mod tests {
    use super::*;
    use serde_yaml;

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.validate().is_ok());
        assert_eq!(config.discord.max_concurrent_requests, 10);
        assert_eq!(config.graph.width, 1200);
        assert_eq!(config.database.url, "sqlite://tgraph.db");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        
        // Test YAML serialization
        let yaml = serde_yaml::to_string(&config).expect("Failed to serialize to YAML");
        assert!(yaml.contains("discord:"));
        assert!(yaml.contains("tautulli:"));
        assert!(yaml.contains("scheduling:"));
        
        // Test YAML deserialization
        let deserialized: Config = serde_yaml::from_str(&yaml)
            .expect("Failed to deserialize from YAML");
        assert_eq!(config.discord.max_concurrent_requests, deserialized.discord.max_concurrent_requests);
        assert_eq!(config.graph.width, deserialized.graph.width);
    }

    #[test]
    fn test_discord_config_validation() {
        // Valid config
        let mut config = DiscordConfig::default();
        config.token = "123456789.abcdef.ghijklmnop".to_string();
        assert!(config.validate().is_ok());

        // Empty token should fail
        config.token = String::new();
        assert!(config.validate().is_err());

        // Invalid token format should fail
        config.token = "invalid_token".to_string();
        assert!(config.validate().is_err());

        // Invalid concurrent requests
        config.token = "123456789.abcdef.ghijklmnop".to_string();
        config.max_concurrent_requests = 0;
        assert!(config.validate().is_err());
        
        config.max_concurrent_requests = 101;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_tautulli_config_validation() {
        let mut config = TautulliConfig::default();
        
        // Valid config
        config.url = "https://example.com".to_string();
        config.api_key = "valid_key".to_string();
        assert!(config.validate().is_ok());

        // Invalid URL
        config.url = "not_a_url".to_string();
        assert!(config.validate().is_err());

        // Empty API key
        config.url = "https://example.com".to_string();
        config.api_key = String::new();
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_graph_config_validation() {
        let mut config = GraphConfig::default();
        assert!(config.validate().is_ok());

        // Invalid dimensions
        config.width = 50; // Too small
        assert!(config.validate().is_err());
        
        config.width = 1200;
        config.height = 5000; // Too large
        assert!(config.validate().is_err());

        // Invalid colors
        config.height = 800;
        config.background_color = "invalid".to_string();
        assert!(config.validate().is_err());

        config.background_color = "#GGGGGG".to_string(); // Invalid hex
        assert!(config.validate().is_err());

        config.background_color = "#FFFFFF".to_string();
        config.primary_color = "#FFF".to_string(); // Too short
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_logging_config_validation() {
        let mut config = LoggingConfig::default();
        assert!(config.validate().is_ok());

        // Invalid log level
        config.level = "invalid".to_string();
        assert!(config.validate().is_err());

        // Valid log levels
        for level in &["trace", "debug", "info", "warn", "error"] {
            config.level = level.to_string();
            assert!(config.validate().is_ok(), "Level {} should be valid", level);
        }
    }

    #[test]
    fn test_minimal_valid_config() {
        let yaml = r"
discord:
  token: 'test_token'
  channels: []
  max_concurrent_requests: 5
  request_timeout_seconds: 15

tautulli:
  url: 'https://tautulli.example.com'
  api_key: 'test_api_key'
  timeout_seconds: 30
  max_retries: 3

scheduling:
  auto_graph_cron: null
  cleanup_cron: null
  timezone: 'UTC'
  enabled: false

graph:
  width: 800
  height: 600
  background_color: '#FFFFFF'
  primary_color: '#007ACC'
  secondary_color: '#FF6B6B'
  font_family: 'Arial'
  font_size: 12
  show_grid: true
  show_legend: true
  max_data_points: 500

database:
  url: 'sqlite://test.db'
  max_connections: 5
  connection_timeout_seconds: 15
  query_timeout_seconds: 30

logging:
  level: 'debug'
  file: null
  colored: true
  include_timestamps: true
  include_location: false
  max_file_size_mb: 5
  max_files: 3
";

        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse minimal config");
        assert!(config.validate().is_ok());
        assert_eq!(config.discord.token, "test_token");
        assert_eq!(config.graph.width, 800);
    }

    #[test]
    fn test_full_config_example() {
        let yaml = r"
discord:
  token: 'NzkyNzE1NDU0MTk2MDg4ODQy.X-hvzA.Ovy4MCQywSkoMRRclStW4xAYK7I'
  channels: ['123456789', '987654321']
  max_concurrent_requests: 20
  request_timeout_seconds: 45

tautulli:
  url: 'https://tautulli.mydomain.com'
  api_key: 'abcdef1234567890'
  timeout_seconds: 60
  max_retries: 5

scheduling:
  auto_graph_cron: '0 0 0 * * *'
  cleanup_cron: '0 0 2 * * 7'
  timezone: 'America/New_York'
  enabled: true

graph:
  width: 1600
  height: 1200
  background_color: '#2F3136'
  primary_color: '#5865F2'
  secondary_color: '#EB459E'
  font_family: 'Roboto'
  font_size: 14
  show_grid: false
  show_legend: true
  max_data_points: 2000

database:
  url: 'postgresql://user:pass@localhost/tgraph'
  max_connections: 20
  connection_timeout_seconds: 30
  query_timeout_seconds: 120

logging:
  level: 'info'
  file: '/var/log/tgraph/app.log'
  colored: false
  include_timestamps: true
  include_location: true
  max_file_size_mb: 50
  max_files: 10
";

        let config: Config = serde_yaml::from_str(yaml).expect("Failed to parse full config");
        assert!(config.validate().is_ok());
        assert_eq!(config.discord.channels.len(), 2);
        assert_eq!(config.scheduling.enabled, true);
        assert_eq!(config.graph.background_color, "#2F3136");
    }

    #[test]
    fn test_comprehensive_validation() {
        // Test individual components first
        let discord_config = DiscordConfig {
            token: "123456789.abcdef.ghijklmnop".to_string(),
            channels: vec![],
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
        };
        assert!(discord_config.validate().is_ok());

        let tautulli_config = TautulliConfig {
            url: "https://tautulli.example.com".to_string(),
            api_key: "test_key".to_string(),
            timeout_seconds: 30,
            max_retries: 3,
        };
        assert!(tautulli_config.validate().is_ok());

                 let scheduling_config = SchedulingConfig {
            auto_graph_cron: Some("0 0 0 * * *".to_string()),
            cleanup_cron: Some("0 0 2 * * 7".to_string()), // Sunday at 2 AM
            timezone: Some("UTC".to_string()),
            enabled: true,
        };
        assert!(scheduling_config.validate().is_ok());
        assert!(scheduling_config.validate_scheduling().is_ok());

        // Test with invalid cron expression
        let mut invalid_scheduling = scheduling_config.clone();
        invalid_scheduling.auto_graph_cron = Some("invalid cron".to_string());
        assert!(invalid_scheduling.validate_scheduling().is_err());
        
        // Test with invalid timezone
        let mut invalid_scheduling2 = scheduling_config.clone();
        invalid_scheduling2.timezone = Some("InvalidTimezone".to_string());
        assert!(invalid_scheduling2.validate_scheduling().is_err());
    }
} 