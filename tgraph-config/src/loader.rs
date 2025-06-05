//! Configuration loading utilities

use crate::Config;
use std::path::Path;
use std::env;
use serde_yaml;
use thiserror::Error;
use tgraph_common::Result as TGraphResult;

/// Configuration loading errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// I/O error when reading configuration file
    #[error("Failed to read configuration file: {0}")]
    IoError(#[from] std::io::Error),
    
    /// YAML parsing error
    #[error("Failed to parse YAML configuration: {0}")]
    ParseError(#[from] serde_yaml::Error),
    
    /// Configuration validation error
    #[error("Configuration validation failed: {0}")]
    ValidationError(#[from] validator::ValidationErrors),
    
    /// Environment variable parsing error
    #[error("Failed to parse environment variable '{var}': {source}")]
    EnvParseError {
        var: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    
    /// Missing required configuration
    #[error("Missing required configuration: {0}")]
    MissingConfig(String),
}

impl From<ConfigError> for tgraph_common::TGraphError {
    fn from(err: ConfigError) -> Self {
        tgraph_common::TGraphError::config(err.to_string())
    }
}

/// Configuration loader for the application
pub struct ConfigLoader;

impl ConfigLoader {
    /// Load configuration from a YAML file with environment variable overrides
    pub fn load_config<P: AsRef<Path>>(path: P) -> Result<Config, ConfigError> {
        // Read and parse the YAML file
        let content = std::fs::read_to_string(path.as_ref())?;
        let mut config: Config = serde_yaml::from_str(&content)?;
        
        // Apply environment variable overrides
        Self::apply_env_overrides(&mut config)?;
        
        // Validate the final configuration
        config.validate_all().map_err(ConfigError::ValidationError)?;
        
        Ok(config)
    }
    
    /// Load configuration from environment variables and files
    pub fn load() -> TGraphResult<Config> {
        // Try to load from default config file first, fall back to defaults
        let config = if let Ok(config_path) = env::var("TGRAPH_CONFIG_PATH") {
            Self::load_config(&config_path)?
        } else if Path::new("config.yaml").exists() {
            Self::load_config("config.yaml")?
        } else if Path::new("config.yml").exists() {
            Self::load_config("config.yml")?
        } else {
            // No config file found, use defaults with env overrides
            let mut config = Config::default();
            Self::apply_env_overrides(&mut config)?;
            config.validate_all().map_err(ConfigError::ValidationError)?;
            config
        };
        
        Ok(config)
    }

    /// Load configuration from a specific file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> TGraphResult<Config> {
        Ok(Self::load_config(path)?)
    }
    
    /// Apply environment variable overrides to configuration
    fn apply_env_overrides(config: &mut Config) -> Result<(), ConfigError> {
        // Discord configuration overrides
        if let Ok(token) = env::var("DISCORD_TOKEN") {
            config.discord.token = token;
        }
        
        if let Ok(channels) = env::var("DISCORD_CHANNELS") {
            config.discord.channels = channels
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
        
        if let Ok(timeout) = env::var("DISCORD_TIMEOUT") {
            config.discord.request_timeout_seconds = timeout.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "DISCORD_TIMEOUT".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(max_requests) = env::var("DISCORD_MAX_REQUESTS") {
            config.discord.max_concurrent_requests = max_requests.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "DISCORD_MAX_REQUESTS".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        // Tautulli configuration overrides
        if let Ok(url) = env::var("TAUTULLI_URL") {
            config.tautulli.url = url;
        }
        
        if let Ok(api_key) = env::var("TAUTULLI_API_KEY") {
            config.tautulli.api_key = api_key;
        }
        
        if let Ok(timeout) = env::var("TAUTULLI_TIMEOUT") {
            config.tautulli.timeout_seconds = timeout.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "TAUTULLI_TIMEOUT".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(retries) = env::var("TAUTULLI_MAX_RETRIES") {
            config.tautulli.max_retries = retries.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "TAUTULLI_MAX_RETRIES".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        // Scheduling configuration overrides
        if let Ok(cron) = env::var("AUTO_GRAPH_CRON") {
            config.scheduling.auto_graph_cron = Some(cron);
        }
        
        if let Ok(cron) = env::var("CLEANUP_CRON") {
            config.scheduling.cleanup_cron = Some(cron);
        }
        
        if let Ok(timezone) = env::var("TIMEZONE") {
            config.scheduling.timezone = Some(timezone);
        }
        
        if let Ok(enabled) = env::var("SCHEDULING_ENABLED") {
            config.scheduling.enabled = enabled.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "SCHEDULING_ENABLED".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        // Graph configuration overrides
        if let Ok(width) = env::var("GRAPH_WIDTH") {
            config.graph.width = width.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_WIDTH".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(height) = env::var("GRAPH_HEIGHT") {
            config.graph.height = height.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_HEIGHT".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(bg_color) = env::var("GRAPH_BACKGROUND_COLOR") {
            config.graph.background_color = bg_color;
        }
        
        if let Ok(primary_color) = env::var("GRAPH_PRIMARY_COLOR") {
            config.graph.primary_color = primary_color;
        }
        
        if let Ok(secondary_color) = env::var("GRAPH_SECONDARY_COLOR") {
            config.graph.secondary_color = secondary_color;
        }
        
        if let Ok(font_family) = env::var("GRAPH_FONT_FAMILY") {
            config.graph.font_family = font_family;
        }
        
        if let Ok(font_size) = env::var("GRAPH_FONT_SIZE") {
            config.graph.font_size = font_size.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_FONT_SIZE".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(show_grid) = env::var("GRAPH_SHOW_GRID") {
            config.graph.show_grid = show_grid.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_SHOW_GRID".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(show_legend) = env::var("GRAPH_SHOW_LEGEND") {
            config.graph.show_legend = show_legend.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_SHOW_LEGEND".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(max_points) = env::var("GRAPH_MAX_DATA_POINTS") {
            config.graph.max_data_points = max_points.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "GRAPH_MAX_DATA_POINTS".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        // Database configuration overrides
        if let Ok(url) = env::var("DATABASE_URL") {
            config.database.url = url;
        }
        
        if let Ok(max_connections) = env::var("DATABASE_MAX_CONNECTIONS") {
            config.database.max_connections = max_connections.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "DATABASE_MAX_CONNECTIONS".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(connection_timeout) = env::var("DATABASE_CONNECTION_TIMEOUT") {
            config.database.connection_timeout_seconds = connection_timeout.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "DATABASE_CONNECTION_TIMEOUT".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(query_timeout) = env::var("DATABASE_QUERY_TIMEOUT") {
            config.database.query_timeout_seconds = query_timeout.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "DATABASE_QUERY_TIMEOUT".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        // Logging configuration overrides
        if let Ok(level) = env::var("LOG_LEVEL") {
            config.logging.level = level;
        }
        
        if let Ok(file) = env::var("LOG_FILE") {
            config.logging.file = Some(file);
        }
        
        if let Ok(colored) = env::var("LOG_COLORED") {
            config.logging.colored = colored.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "LOG_COLORED".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(timestamps) = env::var("LOG_INCLUDE_TIMESTAMPS") {
            config.logging.include_timestamps = timestamps.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "LOG_INCLUDE_TIMESTAMPS".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(location) = env::var("LOG_INCLUDE_LOCATION") {
            config.logging.include_location = location.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "LOG_INCLUDE_LOCATION".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(max_size) = env::var("LOG_MAX_FILE_SIZE_MB") {
            config.logging.max_file_size_mb = max_size.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "LOG_MAX_FILE_SIZE_MB".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        if let Ok(max_files) = env::var("LOG_MAX_FILES") {
            config.logging.max_files = max_files.parse()
                .map_err(|e| ConfigError::EnvParseError {
                    var: "LOG_MAX_FILES".to_string(),
                    source: Box::new(e),
                })?;
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    /// Create a temporary YAML config file for testing
    fn create_test_config_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().expect("Failed to create temp file");
        file.write_all(content.as_bytes()).expect("Failed to write to temp file");
        file
    }
    
    #[test]
    fn test_load_valid_yaml_config() {
        // Clean up any environment variables that might interfere
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_CHANNELS");
        env::remove_var("TAUTULLI_URL");
        env::remove_var("GRAPH_WIDTH");
        env::remove_var("LOG_LEVEL");
        
        let yaml_content = "discord:\n  token: \"MTIzNDU2Nzg5MDEyMzQ1Njc4OTA.AbCdEf.GhIjKlMnOpQrStUvWxYz123456\"\n  channels: [\"123456789\", \"987654321\"]\n  max_concurrent_requests: 5\n  request_timeout_seconds: 30\ntautulli:\n  url: \"https://tautulli.example.com\"\n  api_key: \"test_api_key_12345\"\n  timeout_seconds: 60\n  max_retries: 3\nscheduling:\n  auto_graph_cron: \"0 0 0 * * *\"\n  cleanup_cron: \"0 0 2 * * 7\"\n  timezone: \"UTC\"\n  enabled: true\ngraph:\n  width: 1200\n  height: 800\n  background_color: \"#FFFFFF\"\n  primary_color: \"#FF0000\"\n  secondary_color: \"#00FF00\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"postgresql://user:pass@localhost/db\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  file: \"/var/log/tgraph.log\"\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        
        let temp_file = create_test_config_file(yaml_content);
        let config = ConfigLoader::load_config(temp_file.path()).expect("Failed to load config");
        
        assert_eq!(config.discord.token, "MTIzNDU2Nzg5MDEyMzQ1Njc4OTA.AbCdEf.GhIjKlMnOpQrStUvWxYz123456");
        assert_eq!(config.discord.channels, vec!["123456789", "987654321"]);
        assert_eq!(config.tautulli.url, "https://tautulli.example.com");
        assert_eq!(config.graph.width, 1200);
    }
    
    #[test]
    fn test_load_minimal_config() {
        let yaml_content = "discord:\n  token: \"MTIzNDU2Nzg5MDEyMzQ1Njc4OTA.AbCdEf.GhIjKlMnOpQrStUvWxYz123456\"\n  channels: []\n  max_concurrent_requests: 10\n  request_timeout_seconds: 30\ntautulli:\n  url: \"https://tautulli.example.com\"\n  api_key: \"test_key\"\n  timeout_seconds: 60\n  max_retries: 3\nscheduling:\n  enabled: false\ngraph:\n  width: 1920\n  height: 800\n  background_color: \"#FFFFFF\"\n  primary_color: \"#007ACC\"\n  secondary_color: \"#FF6B6B\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"sqlite://tgraph.db\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        
        let temp_file = create_test_config_file(yaml_content);
        let config = ConfigLoader::load_config(temp_file.path()).expect("Failed to load config");
        
        // Should use defaults for unspecified values
        assert_eq!(config.discord.max_concurrent_requests, 10); // default value
        assert_eq!(config.graph.width, 1920); // default value
    }
    
    #[test]
    fn test_invalid_yaml() {
        let invalid_yaml = "discord:\n  token: \"valid_token\"\n  invalid_field: [unclosed array";
        
        let temp_file = create_test_config_file(invalid_yaml);
        let result = ConfigLoader::load_config(temp_file.path());
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::ParseError(_)));
    }
    
    #[test]
    fn test_validation_error() {
        // Clean up environment variables that might override our invalid values
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("TAUTULLI_URL");
        
        let invalid_config = "discord:\n  token: \"\"\n  channels: []\n  max_concurrent_requests: 10\n  request_timeout_seconds: 30\ntautulli:\n  url: \"not_a_url\"\n  api_key: \"test_key\"\n  timeout_seconds: 60\n  max_retries: 3\nscheduling:\n  enabled: false\ngraph:\n  width: 1920\n  height: 800\n  background_color: \"#FFFFFF\"\n  primary_color: \"#007ACC\"\n  secondary_color: \"#FF6B6B\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"sqlite://tgraph.db\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        
        let temp_file = create_test_config_file(invalid_config);
        let result = ConfigLoader::load_config(temp_file.path());
        
        assert!(result.is_err(), "Expected validation error but config loaded successfully");
        assert!(matches!(result.unwrap_err(), ConfigError::ValidationError(_)));
    }
    
    #[test]
    fn test_environment_variable_overrides() {
        // Set test environment variables  
        env::set_var("DISCORD_TOKEN", "792715454196088842.X-hvzA.Ovy4MCQywSkoMRRclStW4xAYK7I");
        env::set_var("DISCORD_CHANNELS", "111111,222222,333333");
        env::set_var("TAUTULLI_URL", "https://env.tautulli.com");
        env::set_var("GRAPH_WIDTH", "1500");
        env::set_var("LOG_LEVEL", "debug");
        
        let yaml_content = "discord:\n  token: \"original_token\"\n  channels: [\"original_channel\"]\n  max_concurrent_requests: 10\n  request_timeout_seconds: 30\ntautulli:\n  url: \"https://original.tautulli.com\"\n  api_key: \"test_key\"\n  timeout_seconds: 60\n  max_retries: 3\nscheduling:\n  enabled: false\ngraph:\n  width: 1920\n  height: 800\n  background_color: \"#FFFFFF\"\n  primary_color: \"#007ACC\"\n  secondary_color: \"#FF6B6B\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"sqlite://tgraph.db\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        
        let temp_file = create_test_config_file(yaml_content);
        let config = ConfigLoader::load_config(temp_file.path()).expect("Failed to load config");
        
        // Environment variables should override YAML values
        assert_eq!(config.discord.token, "792715454196088842.X-hvzA.Ovy4MCQywSkoMRRclStW4xAYK7I");
        assert_eq!(config.discord.channels, vec!["111111", "222222", "333333"]);
        assert_eq!(config.tautulli.url, "https://env.tautulli.com");
        assert_eq!(config.graph.width, 1500);
        assert_eq!(config.logging.level, "debug");
        
        // Clean up environment variables
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("DISCORD_CHANNELS");
        env::remove_var("TAUTULLI_URL");
        env::remove_var("GRAPH_WIDTH");
        env::remove_var("LOG_LEVEL");
    }
    
    #[test]
    fn test_env_parse_error() {
        // Clean up other environment variables that might interfere
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("TAUTULLI_URL");
        
        env::set_var("GRAPH_WIDTH", "not_a_number");
        
        let yaml_content = "discord:\n  token: \"MTIzNDU2Nzg5MDEyMzQ1Njc4OTA.AbCdEf.GhIjKlMnOpQrStUvWxYz123456\"\n  channels: []\n  max_concurrent_requests: 10\n  request_timeout_seconds: 30\ntautulli:\n  url: \"https://tautulli.example.com\"\n  api_key: \"test_key\"\n  timeout_seconds: 60\n  max_retries: 3\nscheduling:\n  enabled: false\ngraph:\n  width: 1920\n  height: 800\n  background_color: \"#FFFFFF\"\n  primary_color: \"#007ACC\"\n  secondary_color: \"#FF6B6B\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"sqlite://tgraph.db\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        
        let temp_file = create_test_config_file(yaml_content);
        let result = ConfigLoader::load_config(temp_file.path());
        
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::EnvParseError { .. }));
        
        env::remove_var("GRAPH_WIDTH");
    }
    
    #[test]
    fn test_missing_config_file() {
        let result = ConfigLoader::load_config("/nonexistent/path/config.yaml");
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::IoError(_)));
    }
    
    #[test]
    fn test_load_defaults_with_fallback() {
        // Remove any potential config files from current directory for this test
        let _ = fs::remove_file("config.yaml");
        let _ = fs::remove_file("config.yml");
        env::remove_var("TGRAPH_CONFIG_PATH");
        
        // Clean up any environment variables that might affect the test
        env::remove_var("GRAPH_WIDTH");
        env::remove_var("DISCORD_TOKEN");
        env::remove_var("TAUTULLI_URL");
        env::remove_var("LOG_LEVEL");
        
        // This should fall back to defaults
        let config = ConfigLoader::load().expect("Failed to load default config");
        
        // Should have sensible defaults
        assert!(!config.discord.token.is_empty()); // Default token is placeholder
        assert_eq!(config.discord.max_concurrent_requests, 10);
        assert_eq!(config.graph.width, 1920);
    }
} 