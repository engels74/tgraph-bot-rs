//! Common types used across the TGraph application

use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Unique identifier for various entities
pub type EntityId = Uuid;

/// Timestamp type used throughout the application
pub type Timestamp = DateTime<Utc>;

/// Basic user information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: EntityId,
    pub telegram_id: i64,
    pub username: Option<String>,
    pub first_name: String,
    pub last_name: Option<String>,
    pub created_at: Timestamp,
}

/// Bot configuration settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotSettings {
    pub bot_token: String,
    pub default_language: String,
    pub max_graph_size: usize,
    pub cache_ttl_seconds: u64,
} 