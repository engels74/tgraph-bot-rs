//! TGraph Telegram Bot - Main Entry Point

use anyhow::Result;
use clap::Parser;
use poise::serenity_prelude::{self as serenity, GatewayIntents};
use std::sync::Arc;
use tracing::{info, error};
use tracing_subscriber::{self, EnvFilter};

use tgraph_config::{ConfigLoader, Config};
use tgraph_commands::CommandRegistry;
use tgraph_i18n::{I18nManager, Locale};

/// Shared application state accessible across commands and event handlers
pub struct Data {
    /// Application configuration
    pub config: Arc<Config>,
    /// HTTP client for external API calls
    pub http_client: reqwest::Client,
    /// Internationalization manager
    pub i18n: Arc<I18nManager>,
    /// Command registry
    pub commands: Arc<CommandRegistry>,
}

impl std::fmt::Debug for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Data")
            .field("config", &"<Config>")
            .field("http_client", &"<reqwest::Client>")
            .field("i18n", &"<I18nManager>")
            .field("commands", &"<CommandRegistry>")
            .finish()
    }
}

type Error = Box<dyn std::error::Error + Send + Sync>;

/// Command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Configuration file path
    #[arg(short, long)]
    config: Option<String>,

    /// Log level
    #[arg(short, long, default_value = "info")]
    log_level: String,
}

/// Setup function for Poise framework - initializes shared data and handles bot ready event
async fn setup(
    ctx: &serenity::Context,
    ready: &serenity::Ready,
    framework: &poise::Framework<Data, Error>,
) -> Result<Data, Error> {
    info!("Bot connected as: {}", ready.user.name);
    info!("Bot ID: {}", ready.user.id);
    info!("Connected to {} guilds", ready.guilds.len());
    
    // Register slash commands globally
    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    info!("Slash commands registered globally");
    
    // Load configuration
    let config = ConfigLoader::load()?;
    
    // Initialize HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.discord.request_timeout_seconds))
        .build()?;

    // Initialize i18n
    let i18n = I18nManager::new(Locale::default());
    info!("I18n manager initialized");

    // Initialize command registry
    let mut commands = CommandRegistry::new();
    commands.register_all()?;
    info!("Commands registered");

    // Create shared application data
    let data = Data {
        config: Arc::new(config),
        http_client,
        i18n: Arc::new(i18n),
        commands: Arc::new(commands),
    };
    
    Ok(data)
}

/// Event handler for when the bot joins a guild
async fn guild_create(
    _ctx: &serenity::Context,
    guild: &serenity::Guild,
    _is_new: Option<bool>,
) -> Result<(), Error> {
    info!("Joined guild: {} (ID: {})", guild.name, guild.id);
    info!("Guild has {} members", guild.member_count);
    Ok(())
}

/// Global error handler for the framework
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => {
            error!("Failed to start bot: {:?}", error);
        }
        poise::FrameworkError::Command { error, ctx, .. } => {
            error!("Error in command '{}': {:?}", ctx.command().name, error);
        }
        poise::FrameworkError::EventHandler { error, event, .. } => {
            error!("Error in event handler for {:?}: {:?}", event.snake_case_name(), error);
        }
        error => {
            error!("Other error: {:?}", error);
        }
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log_level))
        .init();

    info!("Starting TGraph Discord Bot");

    // Load configuration
    let config = match args.config {
        Some(path) => ConfigLoader::load_from_file(&path)?,
        None => ConfigLoader::load()?,
    };

    info!("Configuration loaded successfully");

    // Validate Discord token
    if config.discord.token.is_empty() {
        anyhow::bail!("Discord token is required but not provided in configuration");
    }

    // Configure Discord intents
    let intents = GatewayIntents::GUILD_MESSAGES 
        | GatewayIntents::MESSAGE_CONTENT 
        | GatewayIntents::GUILDS;

    // Set up Poise framework
    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![], // TODO: Add actual commands from registry
            on_error: |error| Box::pin(on_error(error)),
            event_handler: |ctx, event, framework, data| {
                Box::pin(event_handler(ctx, event, framework, data))
            },
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("!".to_string()),
                mention_as_prefix: true,
                ..Default::default()
            },
            ..Default::default()
        })
        .setup(move |ctx, ready, framework| {
            Box::pin(setup(ctx, ready, framework))
        })
        .build();

    info!("Poise framework configured");

    // Create Discord client
    let mut client = serenity::ClientBuilder::new(&config.discord.token, intents)
        .framework(framework)
        .await?;

    info!("Discord client created");

    // Set up graceful shutdown handling
    let shard_manager = client.shard_manager.clone();
    
    tokio::spawn(async move {
        if let Err(e) = tokio::signal::ctrl_c().await {
            error!("Failed to listen for shutdown signal: {:?}", e);
            return;
        }
        
        info!("Received shutdown signal, starting graceful shutdown");
        
        // Shutdown Discord client
        shard_manager.shutdown_all().await;
        
        info!("Discord client shutdown complete");
    });

    info!("TGraph Discord bot is starting up...");

    // Start the bot
    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
        return Err(why.into());
    }

    info!("TGraph Discord bot has shut down");
    Ok(())
}

/// Central event handler for Discord events
async fn event_handler(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    _data: &Data,
) -> Result<(), Error> {
    match event {
        serenity::FullEvent::GuildCreate { guild, is_new } => {
            guild_create(ctx, guild, *is_new).await?;
        }
        serenity::FullEvent::Ready { data_about_bot } => {
            info!("Bot ready event received for: {}", data_about_bot.user.name);
        }
        _ => {} // Handle other events as needed
    }
    Ok(())
} 