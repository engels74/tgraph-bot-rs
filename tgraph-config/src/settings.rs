//! Application configuration structures

use serde::{Deserialize, Serialize};
use tgraph_common::BotSettings;

/// Main application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub bot: BotSettings,
    pub database: DatabaseConfig,
    pub logging: LoggingConfig,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub file: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            bot: BotSettings {
                bot_token: String::new(),
                default_language: "en".to_string(),
                max_graph_size: 1000,
                cache_ttl_seconds: 3600,
            },
            database: DatabaseConfig {
                url: "sqlite://tgraph.db".to_string(),
                max_connections: 10,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file: None,
            },
        }
    }
} 