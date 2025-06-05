//! Command context and framework integration

use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tgraph_config::Config;
use tgraph_i18n::I18nManager;
use crate::{Permissions, CooldownManager};

/// Shared application state accessible across commands and event handlers
#[derive(Debug)]
pub struct CommandContext {
    /// Application configuration
    pub config: Arc<Config>,
    /// HTTP client for external API calls
    pub http_client: reqwest::Client,
    /// Internationalization manager
    pub i18n: Arc<I18nManager>,
    /// Permission manager
    pub permissions: Arc<Permissions>,
    /// Cooldown manager
    pub cooldown: Arc<CooldownManager>,
}

/// Error type for commands
pub type CommandError = Box<dyn std::error::Error + Send + Sync>;

/// Poise context type alias
pub type Context<'a> = poise::Context<'a, CommandContext, CommandError>;

/// Create a new command context with all required components
pub async fn create_command_context(config: Config) -> Result<CommandContext, CommandError> {
    // Initialize HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.discord.request_timeout_seconds))
        .build()?;

    // Initialize i18n
    let i18n = I18nManager::new(tgraph_i18n::Locale::default());

    // Initialize permissions from config
    let permissions = Permissions::new(&config);

    // Initialize cooldown manager
    let cooldown = CooldownManager::new();

    Ok(CommandContext {
        config: Arc::new(config),
        http_client,
        i18n: Arc::new(i18n),
        permissions: Arc::new(permissions),
        cooldown: Arc::new(cooldown),
    })
} 