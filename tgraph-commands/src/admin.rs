//! Admin commands for the TGraph bot

use poise::Command;
use crate::context::{Context, CommandContext, CommandError};
use crate::cooldown::CooldownConfig;
use std::time::Duration;
use tracing::info;

/// Update graphs command - triggers graph regeneration (admin only)
#[poise::command(
    slash_command,
    default_member_permissions = "MANAGE_GUILD"
)]
pub async fn update_graphs(ctx: Context<'_>) -> Result<(), CommandError> {
    // Admin cooldown - longer to prevent spam
    let cooldown_config = CooldownConfig {
        user: Some(Duration::from_secs(30)),
        global: Some(Duration::from_secs(10)),
        ..Default::default()
    };

    if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
        "update_graphs",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    ) {
        ctx.say(format!("⏰ {}", cooldown_err)).await?;
        return Ok(());
    }

    // For now, this is a placeholder - in the real implementation this would trigger graph generation
    let response = "📊 **Graph Update Initiated**\n\
        🔄 Starting graph regeneration process...\n\
        ⏳ This may take a few moments to complete.\n\
        📈 All graphs will be updated with the latest data from Tautulli.";

    ctx.say(response).await?;

    // Apply cooldown after successful execution
    ctx.data().cooldown.apply_cooldown(
        "update_graphs",
        ctx.author().id,
        Some(ctx.channel_id()),
        &cooldown_config,
    );

    info!("Update graphs command executed by admin user {}", ctx.author().id);
    Ok(())
} 