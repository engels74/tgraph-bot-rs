//! About command implementation using Poise's command macro.

use crate::framework::{Context, Error};

/// Shows information about the bot.
#[poise::command(slash_command)]
pub async fn about(ctx: Context<'_>) -> Result<(), Error> {
    let response = "**TGraph Bot Rust Edition**\n\
                   High-performance Discord bot for automated Tautulli graph generation.\n\
                   Built with Rust and the Poise framework.";

    ctx.say(response).await?;
    Ok(())
}
