//! Application-wide error types using thiserror.

use poise::serenity_prelude as serenity;
use tgraph_common::TGraphError;

/// Main application error type.
#[derive(thiserror::Error, Debug)]
pub enum BotError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(#[from] tgraph_common::TGraphError),

    /// Discord/Serenity error.
    #[error("Discord error: {0}")]
    Discord(#[from] serenity::Error),

    /// Poise framework error.
    #[error("Framework error: {0}")]
    Framework(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

/// Result type for the bot application.
pub type BotResult<T> = Result<T, BotError>;
