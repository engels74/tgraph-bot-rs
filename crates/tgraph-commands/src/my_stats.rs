//! My stats command with user parameter parsing and DM support.

use crate::framework::{Context, Error};
use poise::serenity_prelude as serenity;

/// Get your personal Tautulli statistics.
#[poise::command(slash_command)]
pub async fn my_stats(ctx: Context<'_>, user: Option<serenity::User>) -> Result<(), Error> {
    let target_user = user.as_ref().unwrap_or_else(|| ctx.author());

    let response = format!(
        "Personal statistics for {} are not implemented yet",
        target_user.name
    );

    // For now, just respond in channel
    // DM functionality will be implemented later
    ctx.say(&response).await?;

    Ok(())
}
