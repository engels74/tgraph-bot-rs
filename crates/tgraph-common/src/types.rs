//! Common type definitions and newtype wrappers for domain modeling.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A Discord channel ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u64);

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Discord user ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A Tautulli user ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TautulliUserId(pub u64);

impl fmt::Display for TautulliUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Common result type for the application.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// Application-wide error type.
#[derive(thiserror::Error, Debug)]
pub enum TGraphError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Discord API error.
    #[error("Discord API error: {0}")]
    Discord(String),

    /// Tautulli API error.
    #[error("Tautulli API error: {0}")]
    Tautulli(String),

    /// Graph generation error.
    #[error("Graph generation error: {0}")]
    Graph(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}
