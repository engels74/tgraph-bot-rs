//! Config subcommands with Poise's built-in subcommand support.

use crate::framework::{Context, Error};

/// Configuration management commands.
#[poise::command(slash_command, subcommands("view", "edit"))]
pub async fn config(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}

/// View current configuration.
#[poise::command(slash_command)]
pub async fn view(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Configuration viewing not implemented yet").await?;
    Ok(())
}

/// Edit configuration.
#[poise::command(slash_command)]
pub async fn edit(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Configuration editing not implemented yet").await?;
    Ok(())
}
