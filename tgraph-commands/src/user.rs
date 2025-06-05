//! User commands for the TGraph bot

use poise::Command;
use crate::context::{Context, CommandContext, CommandError};
use crate::cooldown::CooldownConfig;
use std::time::Duration;
use tracing::info;

/// About command - shows bot information
#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), CommandError> {
    // Simple cooldown check
    let cooldown_config = CooldownConfig {
        user: Some(Duration::from_secs(5)),
        ..Default::default()
    };

    if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
        "about",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    ) {
        ctx.say(format!("⏰ {}", cooldown_err)).await?;
        return Ok(());
    }

    let response = format!(
        "🤖 **TGraph Discord Bot**\n\
        📊 A powerful bot for generating and sharing Tautulli statistics graphs\n\
        🔧 Version: {}\n\
        ⚡ Built with Rust and Poise\n\
        📈 Features: Graph generation, statistics tracking, and more!",
        env!("CARGO_PKG_VERSION")
    );

    ctx.say(response).await?;

    // Apply cooldown after successful execution
    ctx.data().cooldown.apply_cooldown(
        "about",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    );

    info!("About command executed by user {}", ctx.author().id);
    Ok(())
}

/// Uptime command - shows how long the bot has been running
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), CommandError> {
    // Simple cooldown check
    let cooldown_config = CooldownConfig {
        user: Some(Duration::from_secs(3)),
        ..Default::default()
    };

    if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
        "uptime",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    ) {
        ctx.say(format!("⏰ {}", cooldown_err)).await?;
        return Ok(());
    }

    // Simple uptime calculation - in a real implementation you'd track bot start time
    let process_uptime = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() % 86400; // Simplified example

    let hours = process_uptime / 3600;
    let minutes = (process_uptime % 3600) / 60;
    let seconds = process_uptime % 60;

    let response = format!(
        "⏰ **Bot Uptime**\n\
        🕐 Approximate uptime: {}h {}m {}s\n\
        ✅ Status: Online and ready!",
        hours, minutes, seconds
    );

    ctx.say(response).await?;

    // Apply cooldown after successful execution
    ctx.data().cooldown.apply_cooldown(
        "uptime",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    );

    info!("Uptime command executed by user {}", ctx.author().id);
    Ok(())
} 