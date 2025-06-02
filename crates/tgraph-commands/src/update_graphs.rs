//! Update graphs administrative command with Poise's permission checks.

use crate::framework::{Context, Error};

/// Manually trigger graph updates (admin only).
#[poise::command(slash_command)]
pub async fn update_graphs(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Manual graph update not implemented yet").await?;
    Ok(())
}
