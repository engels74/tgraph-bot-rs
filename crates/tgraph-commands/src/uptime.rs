//! Uptime command accessing framework data through Poise's context.

use crate::framework::{Context, Error};

/// Shows bot uptime.
#[poise::command(slash_command)]
pub async fn uptime(ctx: Context<'_>) -> Result<(), Error> {
    let response = "Bot uptime: Not implemented yet".to_string();
    ctx.say(response).await?;
    Ok(())
}
