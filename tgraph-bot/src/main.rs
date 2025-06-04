//! TGraph Telegram Bot - Main Entry Point

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{self, EnvFilter};

use tgraph_config::ConfigLoader;
use tgraph_commands::CommandRegistry;
use tgraph_i18n::{I18nManager, Locale};

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

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(&args.log_level))
        .init();

    info!("Starting TGraph Telegram Bot");

    // Load configuration
    let config = match args.config {
        Some(path) => ConfigLoader::load_from_file(&path)?,
        None => ConfigLoader::load()?,
    };

    info!("Configuration loaded successfully");

    // Initialize i18n
    let i18n = I18nManager::new(Locale::default());
    info!("I18n manager initialized");

    // Initialize command registry
    let mut commands = CommandRegistry::new();
    commands.register_all()?;
    info!("Commands registered");

    // TODO: Initialize bot with poise framework
    // TODO: Set up bot handlers and start polling
    
    info!("Bot initialization complete");
    
    // For now, just log that we're ready
    info!("TGraph bot is ready! (TODO: Implement actual bot startup)");
    
    // Keep the application running
    tokio::signal::ctrl_c().await?;
    info!("Shutting down TGraph bot");

    Ok(())
} 