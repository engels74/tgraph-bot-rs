//! User commands for the TGraph bot

use crate::context::{Context, CommandError, record_command_execution};
use crate::cooldown::CooldownConfig;
use std::time::{Duration, Instant};
use tracing::info;

/// About command - shows bot information
#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), CommandError> {
    let start_time = Instant::now();
    
    let result = async {
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
    }.await;

    // Record metrics
    record_command_execution(&ctx, "about", start_time, &result);
    
    result
}

/// Uptime command - shows how long the bot has been running
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), CommandError> {
    let start_time = Instant::now();
    
    let result = async {
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

        // Get actual bot uptime from metrics manager
        let bot_uptime = ctx.data().metrics.get_uptime();
        let hours = bot_uptime.as_secs() / 3600;
        let minutes = (bot_uptime.as_secs() % 3600) / 60;
        let seconds = bot_uptime.as_secs() % 60;

        // Get metrics summary for additional context
        let (total_executions, successes, failures) = ctx.data().metrics.get_global_counts();

        let response = format!(
            "⏰ **Bot Uptime & Statistics**\n\
            🕐 Uptime: {}h {}m {}s\n\
            📊 Commands executed: {} (✅ {} succeeded, ❌ {} failed)\n\
            ✅ Status: Online and ready!",
            hours, minutes, seconds, total_executions, successes, failures
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
    }.await;

    // Record metrics
    record_command_execution(&ctx, "uptime", start_time, &result);
    
    result
} 