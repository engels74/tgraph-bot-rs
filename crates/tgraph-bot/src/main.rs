//! Main entry point for TGraph Bot.

use std::env;
use tgraph_bot::{BotResult, TGraphBot};
use tgraph_config::Config;
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> BotResult<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "tgraph_bot=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("Starting TGraph Bot Rust Edition");

    // Load configuration
    let config = load_config().await?;

    // Create and start bot
    let bot = TGraphBot::new(config);

    if let Err(e) = bot.start().await {
        error!("Bot failed to start: {}", e);
        return Err(e);
    }

    Ok(())
}

async fn load_config() -> BotResult<Config> {
    // For now, create a default config
    // This will be replaced with proper config loading later
    let mut config = Config::default();

    // Try to get token from environment
    if let Ok(token) = env::var("DISCORD_TOKEN") {
        config.discord.token = token;
    }

    if let Ok(api_key) = env::var("TAUTULLI_API_KEY") {
        config.tautulli.api_key = api_key;
    }

    if let Ok(url) = env::var("TAUTULLI_URL") {
        config.tautulli.url = url;
    }

    config.validate()?;
    Ok(config)
}
