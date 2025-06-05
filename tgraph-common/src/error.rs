//! Error types and utilities for TGraph

use thiserror::Error;

/// Result type alias for TGraph operations
pub type Result<T> = std::result::Result<T, TGraphError>;

/// Main error type for TGraph operations
#[derive(Error, Debug)]
pub enum TGraphError {
    /// Configuration related errors
    #[error("Configuration error: {message}")]
    Config { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// I/O related errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Network related errors (HTTP requests, etc.)
    #[error("Network error: {message}")]
    Network {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Discord API related errors
    #[error("Discord API error: {message}")]
    Discord {
        message: String,
        error_code: Option<u16>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Tautulli API related errors
    #[error("Tautulli API error: {message}")]
    Tautulli {
        message: String,
        status_code: Option<u16>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Database related errors
    #[error("Database error: {message}")]
    Database {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Serialization/deserialization errors
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// Graph generation and plotting errors
    #[error("Graph error: {message}")]
    Graph {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Internationalization and localization errors
    #[error("Localization error: {message}")]
    Localization {
        message: String,
        locale: Option<String>,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Authentication and authorization errors
    #[error("Auth error: {message}")]
    Auth {
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },

    /// Validation errors for user input or data
    #[error("Validation error: {message}")]
    Validation {
        message: String,
        field: Option<String>,
    },

    /// Generic error with custom message
    #[error("{message}")]
    Generic { 
        message: String,
        #[source]
        source: Option<Box<dyn std::error::Error + Send + Sync>>,
    },
}

impl TGraphError {
    /// Create a new generic error with a custom message
    pub fn new(msg: impl Into<String>) -> Self {
        Self::Generic { 
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new generic error with a custom message and source
    pub fn with_source(
        msg: impl Into<String>, 
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Generic {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        Self::Config { 
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new configuration error with source
    pub fn config_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Config {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new network error
    pub fn network(msg: impl Into<String>) -> Self {
        Self::Network {
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new network error with source
    pub fn network_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Network {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new Discord API error
    pub fn discord(msg: impl Into<String>) -> Self {
        Self::Discord {
            message: msg.into(),
            error_code: None,
            source: None,
        }
    }

    /// Create a new Discord API error with error code
    pub fn discord_with_code(msg: impl Into<String>, code: u16) -> Self {
        Self::Discord {
            message: msg.into(),
            error_code: Some(code),
            source: None,
        }
    }

    /// Create a new Tautulli API error
    pub fn tautulli(msg: impl Into<String>) -> Self {
        Self::Tautulli {
            message: msg.into(),
            status_code: None,
            source: None,
        }
    }

    /// Create a new Tautulli API error with status code
    pub fn tautulli_with_status(msg: impl Into<String>, status: u16) -> Self {
        Self::Tautulli {
            message: msg.into(),
            status_code: Some(status),
            source: None,
        }
    }

    /// Create a new database error
    pub fn database(msg: impl Into<String>) -> Self {
        Self::Database {
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new database error with source
    pub fn database_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Database {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new graph error
    pub fn graph(msg: impl Into<String>) -> Self {
        Self::Graph {
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new graph error with source
    pub fn graph_with_source(
        msg: impl Into<String>,
        source: impl std::error::Error + Send + Sync + 'static
    ) -> Self {
        Self::Graph {
            message: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Create a new localization error
    pub fn localization(msg: impl Into<String>) -> Self {
        Self::Localization {
            message: msg.into(),
            locale: None,
            source: None,
        }
    }

    /// Create a new localization error with locale
    pub fn localization_with_locale(msg: impl Into<String>, locale: impl Into<String>) -> Self {
        Self::Localization {
            message: msg.into(),
            locale: Some(locale.into()),
            source: None,
        }
    }

    /// Create a new auth error
    pub fn auth(msg: impl Into<String>) -> Self {
        Self::Auth {
            message: msg.into(),
            source: None,
        }
    }

    /// Create a new validation error
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation {
            message: msg.into(),
            field: None,
        }
    }

    /// Create a new validation error with field name
    pub fn validation_field(msg: impl Into<String>, field: impl Into<String>) -> Self {
        Self::Validation {
            message: msg.into(),
            field: Some(field.into()),
        }
    }
}

// Error conversion implementations for external types

/// Convert from reqwest::Error to TGraphError
impl From<reqwest::Error> for TGraphError {
    fn from(err: reqwest::Error) -> Self {
        if err.is_timeout() {
            Self::network_with_source("Request timeout", err)
        } else if err.is_connect() {
            Self::network_with_source("Connection failed", err)
        } else if err.is_status() {
            let status_code = err.status().map(|s| s.as_u16()).unwrap_or(0);
            Self::network_with_source(
                format!("HTTP error: {}", status_code),
                err
            )
        } else {
            Self::network_with_source("Network request failed", err)
        }
    }
}

/// Convert from toml::de::Error to TGraphError
impl From<toml::de::Error> for TGraphError {
    fn from(err: toml::de::Error) -> Self {
        Self::config_with_source("TOML parsing error", err)
    }
}

/// Convert from config::ConfigError to TGraphError  
impl From<config::ConfigError> for TGraphError {
    fn from(err: config::ConfigError) -> Self {
        Self::config_with_source("Configuration loading error", err)
    }
}

#[cfg(feature = "plotters")]
/// Convert from plotters drawing errors to TGraphError
impl<T> From<plotters::drawing::DrawingAreaErrorKind<T>> for TGraphError 
where 
    T: std::error::Error + Send + Sync + 'static
{
    fn from(err: plotters::drawing::DrawingAreaErrorKind<T>) -> Self {
        Self::graph_with_source("Graph rendering failed", err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{error::Error, io};

    #[test]
    fn test_error_creation() {
        let error = TGraphError::new("test message");
        assert!(error.to_string().contains("test message"));

        let config_error = TGraphError::config("config issue");
        assert!(config_error.to_string().contains("Configuration error"));
        assert!(config_error.to_string().contains("config issue"));

        let discord_error = TGraphError::discord_with_code("API error", 429);
        assert!(discord_error.to_string().contains("Discord API error"));
        assert!(discord_error.to_string().contains("API error"));

        let tautulli_error = TGraphError::tautulli_with_status("Server error", 500);
        assert!(tautulli_error.to_string().contains("Tautulli API error"));
        assert!(tautulli_error.to_string().contains("Server error"));

        let validation_error = TGraphError::validation_field("Invalid input", "username");
        assert!(validation_error.to_string().contains("Validation error"));
        assert!(validation_error.to_string().contains("Invalid input"));

        let localization_error = TGraphError::localization_with_locale("Translation missing", "en-US");
        assert!(localization_error.to_string().contains("Localization error"));
        assert!(localization_error.to_string().contains("Translation missing"));
    }

    #[test]
    fn test_error_with_source() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let wrapped_error = TGraphError::with_source("Failed to read file", io_error);

        assert!(wrapped_error.to_string().contains("Failed to read file"));
        assert!(wrapped_error.source().is_some());

        let config_source_error = TGraphError::config_with_source(
            "Config loading failed",
            io::Error::new(io::ErrorKind::PermissionDenied, "Access denied")
        );

        assert!(config_source_error.to_string().contains("Configuration error"));
        assert!(config_source_error.to_string().contains("Config loading failed"));
        assert!(config_source_error.source().is_some());
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "File not found");
        let tgraph_error: TGraphError = io_error.into();

        assert!(tgraph_error.to_string().contains("I/O error"));
        assert!(tgraph_error.source().is_some());
    }

    #[test]
    fn test_serde_error_conversion() {
        let invalid_json = r#"{"invalid": json}"#;
        let serde_error = serde_json::from_str::<serde_json::Value>(invalid_json).unwrap_err();
        let tgraph_error: TGraphError = serde_error.into();

        assert!(tgraph_error.to_string().contains("Serialization error"));
    }

    #[test]
    fn test_error_display_formatting() {
        let error = TGraphError::new("test error");
        let display_str = format!("{}", error);
        assert_eq!(display_str, "test error");

        let config_error = TGraphError::config("missing field");
        let config_display = format!("{}", config_error);
        assert_eq!(config_display, "Configuration error: missing field");

        let discord_error = TGraphError::discord_with_code("rate limited", 429);
        let discord_display = format!("{}", discord_error);
        assert_eq!(discord_display, "Discord API error: rate limited");
    }

    #[test]
    fn test_error_debug_formatting() {
        let error = TGraphError::new("debug test");
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("Generic"));
        assert!(debug_str.contains("debug test"));
    }

    #[test]
    fn test_result_type_alias() {
        fn returns_result() -> Result<String> {
            Ok("success".to_string())
        }

        fn returns_error() -> Result<String> {
            Err(TGraphError::new("failure"))
        }

        assert!(returns_result().is_ok());
        assert!(returns_error().is_err());

        let success = returns_result().unwrap();
        assert_eq!(success, "success");

        let error = returns_error().unwrap_err();
        assert!(error.to_string().contains("failure"));
    }

    #[test]
    fn test_error_chain_preservation() {
        let root_error = io::Error::new(io::ErrorKind::NotFound, "Root cause");
        let middle_error = TGraphError::config_with_source("Middle layer", root_error);
        let top_error = TGraphError::with_source("Top layer", middle_error);

        assert!(top_error.to_string().contains("Top layer"));
        
        // Check that we can walk the error chain
        let mut current_error: &dyn std::error::Error = &top_error;
        let mut error_count = 0;
        
        while let Some(source) = current_error.source() {
            current_error = source;
            error_count += 1;
        }
        
        assert!(error_count >= 1); // Should have at least one source
    }
} 