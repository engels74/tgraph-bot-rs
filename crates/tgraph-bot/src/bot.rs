//! Core bot logic using the Poise framework.

use crate::error::{BotError, BotResult};
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tgraph_commands::{create_framework, Data};
use tgraph_config::Config;

/// Main bot structure.
pub struct TGraphBot {
    config: Arc<Config>,
}

impl TGraphBot {
    /// Creates a new bot instance.
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(config),
        }
    }

    /// Starts the bot.
    pub async fn start(&self) -> BotResult<()> {
        let config_clone = self.config.clone();

        let framework = create_framework()
            .setup(move |ctx, _ready, framework| {
                let config = config_clone.clone();
                Box::pin(async move {
                    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                    Ok(Data { config })
                })
            })
            .build();

        let mut client = serenity::ClientBuilder::new(
            &self.config.discord.token,
            serenity::GatewayIntents::non_privileged(),
        )
        .framework(framework)
        .await
        .map_err(|e| BotError::Framework(format!("{:?}", e)))?;

        client
            .start()
            .await
            .map_err(|e| BotError::Framework(format!("{:?}", e)))?;
        Ok(())
    }
}
