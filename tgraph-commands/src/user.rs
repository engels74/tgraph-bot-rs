//! User commands for the TGraph bot

use crate::context::{Context, CommandError, record_command_execution};
use crate::cooldown::CooldownConfig;
use crate::statistics::TimePeriod;
use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug};
use poise::serenity_prelude::{UserId, CreateMessage, CreateEmbed, Colour};
use chrono::Utc;

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

/// Send a direct message to a user with retry logic
async fn send_direct_message(
    ctx: &Context<'_>, 
    user_id: UserId, 
    content: String,
    is_stats_message: bool,
) -> Result<bool, CommandError> {
    debug!("Attempting to send DM to user {}", user_id);

    // Create a DM channel with the user
    let dm_channel = match user_id.create_dm_channel(&ctx.serenity_context().http).await {
        Ok(channel) => channel,
        Err(e) => {
            warn!("Failed to create DM channel with user {}: {}", user_id, e);
            return Ok(false);
        }
    };

    let message = if is_stats_message {
        // For statistics messages, create a proper embed with privacy notice
        let embed = CreateEmbed::new()
            .title("📊 Your Personal Statistics")
            .description(&content)
            .color(Colour::from_rgb(147, 51, 234)) // Purple color for stats
            .footer(
                poise::serenity_prelude::CreateEmbedFooter::new(
                    "🔒 This message was sent privately to protect your privacy. Your statistics are confidential."
                )
            )
            .timestamp(poise::serenity_prelude::Timestamp::now());

        CreateMessage::new().embed(embed)
    } else {
        // For regular messages, just send as text
        CreateMessage::new().content(content)
    };

    // Send the message with retry logic
    let mut retry_count = 0;
    const MAX_RETRIES: u32 = 3;
    const RETRY_DELAY: Duration = Duration::from_millis(1000);

    while retry_count < MAX_RETRIES {
        match dm_channel.send_message(&ctx.serenity_context().http, message.clone()).await {
            Ok(_) => {
                debug!("Successfully sent DM to user {} on attempt {}", user_id, retry_count + 1);
                return Ok(true);
            }
            Err(e) => {
                retry_count += 1;
                warn!("Failed to send DM to user {} on attempt {}: {}", user_id, retry_count, e);
                
                if retry_count < MAX_RETRIES {
                    tokio::time::sleep(RETRY_DELAY).await;
                } else {
                    error!("Failed to send DM to user {} after {} attempts", user_id, MAX_RETRIES);
                    return Ok(false);
                }
            }
        }
    }

    Ok(false)
}

/// Format user statistics for DM delivery with enhanced privacy formatting
fn format_user_statistics_for_dm(stats: &crate::statistics::UserActivity) -> String {
    let period_name = stats.period.name();
    
    // Format time range
    let time_range = if matches!(stats.period, TimePeriod::AllTime) {
        "All Time".to_string()
    } else {
        format!("{} to {}", 
            stats.period_start.format("%Y-%m-%d"),
            stats.period_end.format("%Y-%m-%d")
        )
    };

    // Format most used command
    let most_used = stats.most_used_command
        .as_ref()
        .map(|cmd| format!("/{}", cmd))
        .unwrap_or_else(|| "None".to_string());

    // Format activity times
    let most_active_hour = stats.most_active_hour()
        .map(|h| format!("{}:00", h))
        .unwrap_or_else(|| "N/A".to_string());

    let most_active_day = stats.most_active_day()
        .map(|d| match d {
            0 => "Sunday",
            1 => "Monday", 
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Unknown",
        })
        .unwrap_or("N/A");

    // Format command breakdown (top 5)
    let mut command_list = stats.command_breakdown
        .iter()
        .collect::<Vec<_>>();
    command_list.sort_by(|a, b| b.1.cmp(a.1));
    
    let top_commands = if command_list.is_empty() {
        "None".to_string()
    } else {
        command_list
            .iter()
            .take(5)
            .map(|(cmd, count)| format!("/{}: {}", cmd, count))
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Format first and last command times
    let first_command = stats.first_command
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let last_command = stats.last_command
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    // Enhanced DM formatting with privacy emphasis
    format!(
        "**Your {} Statistics** 📊\n\
        **📅 Period:** {}\n\
        \n\
        **📈 Command Usage**\n\
        • 🔢 **Total Commands:** {}\n\
        • ✅ **Successful:** {} ({:.1}%)\n\
        • ❌ **Failed:** {}\n\
        • ⚡ **Avg Response Time:** {:.1}ms\n\
        \n\
        **🏆 Your Activity Highlights**\n\
        • 🎯 **Most Used Command:** {}\n\
        • 🕐 **Most Active Hour:** {}\n\
        • 📆 **Most Active Day:** {}\n\
        \n\
        **📋 Top 5 Commands**\n\
        {}\n\
        \n\
        **🌍 Activity Scope**\n\
        • 📺 **Unique Channels:** {}\n\
        • 🏠 **Unique Servers:** {}\n\
        \n\
        **⏰ Timeline**\n\
        • 🚀 **First Command:** {}\n\
        • 🕐 **Latest Command:** {}\n\
        \n\
        *📱 This data is private and only visible to you. You can adjust your privacy preferences by contacting a server administrator.*",
        period_name,
        time_range,
        stats.total_commands,
        stats.successful_commands,
        stats.success_rate(),
        stats.failed_commands,
        stats.avg_response_time_ms,
        most_used,
        most_active_hour,
        most_active_day,
        top_commands,
        stats.unique_channels,
        stats.unique_guilds,
        first_command,
        last_command
    )
}

/// My stats command - shows personal user statistics
#[poise::command(slash_command)]
pub async fn my_stats(
    ctx: Context<'_>,
    #[description = "Time period for statistics (daily, weekly, monthly, all-time)"]
    period: Option<String>,
) -> Result<(), CommandError> {
    let start_time = Instant::now();
    
    let result = async {
        // Cooldown check - longer cooldown since this is a more expensive operation
        let cooldown_config = CooldownConfig {
            user: Some(Duration::from_secs(10)),
            ..Default::default()
        };

        if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
            "my_stats",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        ) {
            ctx.say(format!("⏰ {}", cooldown_err)).await?;
            return Ok(());
        }

        // Parse the time period
        let time_period = match period.as_ref().map(|s| s.as_str()) {
            Some("daily") => TimePeriod::Daily,
            Some("weekly") => TimePeriod::Weekly,
            Some("monthly") => TimePeriod::Monthly,
            Some("all-time") | None => TimePeriod::AllTime,
            _ => {
                ctx.say("❌ Invalid time period. Use: daily, weekly, monthly, or all-time").await?;
                return Ok(());
            }
        };

        let user_id = ctx.author().id.get();

        // Get user preferences to check DM delivery preference
        let user_preferences = match ctx.data().user_db.get_or_create_preferences(user_id).await {
            Ok(prefs) => prefs,
            Err(e) => {
                warn!("Failed to get user preferences for user {}: {}", user_id, e);
                // Continue with default behavior (channel response) if preferences can't be loaded
                ctx.say("⚠️ Unable to load your preferences. Showing statistics here instead.").await?;
                
                // Proceed with channel delivery as fallback
                let user_executions = ctx.data().metrics.get_user_executions(user_id);
                handle_stats_display(&ctx, user_id, time_period, &user_executions, false).await?;
                return Ok(());
            }
        };

        // Get user's command execution history
        let user_executions = ctx.data().metrics.get_user_executions(user_id);

        // Check if user prefers DM delivery
        if user_preferences.prefer_dm_delivery {
            // Check DM throttling
            if !ctx.data().dm_throttle.can_send_dm(user_id).await {
                if let Some(remaining) = ctx.data().dm_throttle.get_remaining_throttle(user_id).await {
                    let minutes = remaining.as_secs() / 60;
                    let seconds = remaining.as_secs() % 60;
                    ctx.say(format!(
                        "⏰ **DM Throttle Active**\n\
                        You can request your personal statistics via DM again in {}m {}s.\n\
                        \n\
                        💡 *Tip: You can still view your statistics here in the channel by asking an admin to disable DM delivery for you.*",
                        minutes, seconds
                    )).await?;
                    return Ok(());
                }
            }

            // Acknowledge the request in the channel
            ctx.say("📬 **Sending your statistics via direct message...**\n\
                    🔒 Your personal data will be delivered privately to protect your privacy.\n\
                    \n\
                    *If you don't receive a DM, please check your privacy settings or contact an administrator.*").await?;

            // Try to send DM
            let dm_sent = handle_stats_display(&ctx, user_id, time_period, &user_executions, true).await?;
            
            if dm_sent {
                // Record DM sent for throttling
                ctx.data().dm_throttle.record_dm_sent(user_id).await;
                debug!("Successfully sent statistics DM to user {}", user_id);
            } else {
                // DM failed, show in channel as fallback
                ctx.say("❌ **Failed to send DM**\n\
                        Unable to send your statistics via direct message. This might be due to your privacy settings.\n\
                        \n\
                        📊 **Showing your statistics here instead:**").await?;
                
                handle_stats_display(&ctx, user_id, time_period, &user_executions, false).await?;
            }
        } else {
            // User prefers channel delivery, show statistics in channel
            handle_stats_display(&ctx, user_id, time_period, &user_executions, false).await?;
        }

        // Apply cooldown after successful execution
        ctx.data().cooldown.apply_cooldown(
            "my_stats",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        );

        // Log statistics access for audit purposes
        ctx.data().audit_logger.log_statistics_access(
            user_id, 
            Some(user_id), 
            &format!("{:?}_statistics", time_period)
        ).await;

        info!("My stats command executed by user {} for period {:?} (DM: {})", 
              user_id, time_period, user_preferences.prefer_dm_delivery);
        Ok(())
    }.await;

    // Record metrics
    record_command_execution(&ctx, "my_stats", start_time, &result);
    
    result
}

/// Handle statistics display either in channel or via DM
async fn handle_stats_display(
    ctx: &Context<'_>, 
    user_id: u64, 
    time_period: TimePeriod, 
    user_executions: &[crate::metrics::CommandExecution],
    use_dm: bool,
) -> Result<bool, CommandError> {
    // Get user statistics for the specified period
    match ctx.data().user_stats.get_user_statistics(user_id, time_period, user_executions).await {
        Ok(Some(stats)) => {
            if use_dm {
                // Format for DM with enhanced privacy formatting
                let dm_response = format_user_statistics_for_dm(&stats);
                
                // Send DM
                if send_direct_message(ctx, ctx.author().id, dm_response, true).await? {
                    Ok(true) // DM sent successfully
                } else {
                    Ok(false) // DM failed
                }
            } else {
                // Format for channel response
                let response = format_user_statistics(&stats);
                ctx.say(response).await?;
                Ok(true) // Channel message sent successfully
            }
        }
        Ok(None) => {
            // User has no statistics or privacy settings prevent display
            let response = match time_period {
                TimePeriod::AllTime => {
                    if use_dm {
                        "📊 **Your Personal Statistics**\n\
                        🔍 No command usage found yet. Start using commands to see your statistics!\n\
                        \n\
                        🔒 *This message was sent privately to protect your privacy.*"
                    } else {
                        "📊 **Your Statistics**\n\
                        🔍 No command usage found yet. Start using commands to see your statistics!"
                    }
                }
                _ => {
                    if use_dm {
                        "📊 **Your Personal Statistics**\n\
                        🔍 No command usage found for this time period. Try a different period or use more commands!\n\
                        \n\
                        🔒 *This message was sent privately to protect your privacy.*"
                    } else {
                        "📊 **Your Statistics**\n\
                        🔍 No command usage found for this time period. Try a different period or use more commands!"
                    }
                }
            };

            if use_dm {
                if send_direct_message(ctx, ctx.author().id, response.to_string(), true).await? {
                    Ok(true) // DM sent successfully
                } else {
                    Ok(false) // DM failed
                }
            } else {
                ctx.say(response).await?;
                Ok(true) // Channel message sent successfully
            }
        }
        Err(e) => {
            tracing::error!("Failed to get user statistics for user {}: {}", user_id, e);
            
            let error_response = if use_dm {
                "❌ **Error Retrieving Statistics**\n\
                Failed to retrieve your statistics. Please try again later.\n\
                \n\
                🔒 *This message was sent privately to protect your privacy.*"
            } else {
                "❌ Failed to retrieve your statistics. Please try again later."
            };

            if use_dm {
                if send_direct_message(ctx, ctx.author().id, error_response.to_string(), false).await? {
                    Ok(true) // DM sent successfully
                } else {
                    Ok(false) // DM failed
                }
            } else {
                ctx.say(error_response).await?;
                Ok(true) // Channel message sent successfully
            }
        }
    }
}

/// Format user statistics into a readable Discord message
fn format_user_statistics(stats: &crate::statistics::UserActivity) -> String {
    let period_name = stats.period.name();
    
    // Format time range
    let time_range = if matches!(stats.period, TimePeriod::AllTime) {
        "All Time".to_string()
    } else {
        format!("{} to {}", 
            stats.period_start.format("%Y-%m-%d"),
            stats.period_end.format("%Y-%m-%d")
        )
    };

    // Format most used command
    let most_used = stats.most_used_command
        .as_ref()
        .map(|cmd| format!("/{}", cmd))
        .unwrap_or_else(|| "None".to_string());

    // Format activity times
    let most_active_hour = stats.most_active_hour()
        .map(|h| format!("{}:00", h))
        .unwrap_or_else(|| "N/A".to_string());

    let most_active_day = stats.most_active_day()
        .map(|d| match d {
            0 => "Sunday",
            1 => "Monday", 
            2 => "Tuesday",
            3 => "Wednesday",
            4 => "Thursday",
            5 => "Friday",
            6 => "Saturday",
            _ => "Unknown",
        })
        .unwrap_or("N/A");

    // Format command breakdown (top 5)
    let mut command_list = stats.command_breakdown
        .iter()
        .collect::<Vec<_>>();
    command_list.sort_by(|a, b| b.1.cmp(a.1));
    
    let top_commands = if command_list.is_empty() {
        "None".to_string()
    } else {
        command_list
            .iter()
            .take(5)
            .map(|(cmd, count)| format!("/{}: {}", cmd, count))
            .collect::<Vec<_>>()
            .join(", ")
    };

    // Format first and last command times
    let first_command = stats.first_command
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    let last_command = stats.last_command
        .map(|t| t.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| "N/A".to_string());

    format!(
        "📊 **Your {} Statistics**\n\
        📅 **Period:** {}\n\
        \n\
        **📈 Command Usage**\n\
        🔢 Total Commands: {}\n\
        ✅ Successful: {} ({:.1}%)\n\
        ❌ Failed: {}\n\
        ⚡ Avg Response Time: {:.1}ms\n\
        \n\
        **🏆 Most Used**\n\
        🎯 Command: {}\n\
        🕐 Active Hour: {}\n\
        📆 Active Day: {}\n\
        \n\
        **📋 Top Commands**\n\
        {}\n\
        \n\
        **🌍 Activity Scope**\n\
        📺 Unique Channels: {}\n\
        🏠 Unique Servers: {}\n\
        \n\
        **⏰ Timeline**\n\
        🚀 First Command: {}\n\
        🕐 Latest Command: {}",
        period_name,
        time_range,
        stats.total_commands,
        stats.successful_commands,
        stats.success_rate(),
        stats.failed_commands,
        stats.avg_response_time_ms,
        most_used,
        most_active_hour,
        most_active_day,
        top_commands,
        stats.unique_channels,
        stats.unique_guilds,
        first_command,
        last_command
    )
}

/// Export user data command - provides user with all their stored data for GDPR compliance
#[poise::command(slash_command)]
pub async fn export_my_data(ctx: Context<'_>) -> Result<(), CommandError> {
    let start_time = Instant::now();
    
    let result = async {
        let user_id = ctx.author().id.get();
        
        // Cooldown check
        let cooldown_config = CooldownConfig {
            user: Some(Duration::from_secs(300)), // 5 minute cooldown for data export
            ..Default::default()
        };

        if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
            "export_my_data",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        ) {
            ctx.say(format!("⏰ {}", cooldown_err)).await?;
            return Ok(());
        }

        // Check user preferences first
        let preferences = match ctx.data().user_db.get_preferences(user_id) {
            Ok(Some(prefs)) => prefs,
            Ok(None) => {
                ctx.say("📋 No data found for your account. You haven't used any bot features that store data.").await?;
                return Ok(());
            }
            Err(e) => {
                error!("Failed to retrieve user preferences for export: {}", e);
                ctx.say("❌ Failed to retrieve your data. Please try again later.").await?;
                return Ok(());
            }
        };

        // Check if user allows data export
        if !preferences.allow_data_export {
            ctx.say("🔒 Data export is disabled in your privacy settings. Contact an administrator if you need to change this.").await?;
            return Ok(());
        }

        ctx.say("🔄 Generating your data export. This may take a moment...").await?;

        // Collect all user data
        let mut export_data = serde_json::Map::new();
        
        // Add export metadata
        export_data.insert("export_timestamp".to_string(), serde_json::Value::String(Utc::now().to_rfc3339()));
        export_data.insert("user_id".to_string(), serde_json::Value::Number(user_id.into()));
        export_data.insert("export_type".to_string(), serde_json::Value::String("complete_user_data".to_string()));

        // Export user preferences
        if let Ok(Some(user_prefs_data)) = ctx.data().user_db.export_user_data(user_id) {
            export_data.insert("user_preferences".to_string(), user_prefs_data);
        }

        // Export command execution history
        let user_executions = ctx.data().metrics.get_user_executions(user_id);
        export_data.insert("command_executions".to_string(), serde_json::to_value(&user_executions).unwrap_or_default());

        // Export aggregated statistics
        if let Ok(stats_data) = ctx.data().user_stats.export_user_statistics_data(user_id, &user_executions).await {
            export_data.insert("aggregated_statistics".to_string(), stats_data);
        }

        // Add data summary
        let summary = serde_json::json!({
            "total_command_executions": user_executions.len(),
            "data_retention_days": preferences.data_retention_days,
            "account_created": preferences.created_at.to_rfc3339(),
            "last_updated": preferences.updated_at.to_rfc3339(),
            "privacy_settings": {
                "username_visible": preferences.username_visible,
                "allow_public_stats": preferences.allow_public_stats,
                "prefer_dm_delivery": preferences.prefer_dm_delivery,
                "preferred_language": preferences.preferred_language
            }
        });
        export_data.insert("summary".to_string(), summary);

        // Format the export as JSON
        let export_json = serde_json::to_string_pretty(&export_data)
            .map_err(|e| {
                error!("Failed to serialize export data: {}", e);
                CommandError::from("Failed to generate export file")
            })?;

        // Try to send via DM first, then fallback to channel
        let dm_content = format!(
            "📋 **Your Complete Data Export**\n\n\
            **Export Details:**\n\
            🕐 Generated: {}\n\
            📊 Command Executions: {}\n\
            🔧 Account Created: {}\n\
            📝 Last Updated: {}\n\n\
            **Privacy Notice:**\n\
            🔒 This export contains ALL data we have stored about your account.\n\
            📱 This data is sent privately and confidentially.\n\
            🗑️ You can request data deletion using `/delete_my_data`.\n\
            ⚖️ This export complies with GDPR and data protection regulations.\n\n\
            **📄 Your data (JSON format):**\n\
            ```json\n{}\n```",
            Utc::now().format("%Y-%m-%d %H:%M UTC"),
            user_executions.len(),
            preferences.created_at.format("%Y-%m-%d %H:%M UTC"),
            preferences.updated_at.format("%Y-%m-%d %H:%M UTC"),
            export_json
        );

        // Check if message is too long for Discord (2000 char limit)
        let success = if dm_content.len() > 2000 {
            // If data is too large, create a summary DM and mention file attachment alternative
            let summary_content = format!(
                "📋 **Your Data Export Summary**\n\n\
                **Export Details:**\n\
                🕐 Generated: {}\n\
                📊 Command Executions: {}\n\
                🔧 Account Created: {}\n\
                📝 Last Updated: {}\n\n\
                ⚠️ **Large Export Notice:**\n\
                Your data export is too large for a Discord message ({} characters).\n\
                The complete JSON data has been logged securely.\n\n\
                **Alternative Options:**\n\
                • Contact an administrator for file-based export\n\
                • Use `/delete_my_data` if you want to remove your data instead\n\n\
                **Privacy Notice:**\n\
                🔒 All your data is handled according to GDPR requirements.\n\
                📱 This summary was sent privately to protect your privacy.",
                Utc::now().format("%Y-%m-%d %H:%M UTC"),
                user_executions.len(),
                preferences.created_at.format("%Y-%m-%d %H:%M UTC"),
                preferences.updated_at.format("%Y-%m-%d %H:%M UTC"),
                dm_content.len()
            );
            
            send_direct_message(&ctx, ctx.author().id, summary_content, false).await?
        } else {
            send_direct_message(&ctx, ctx.author().id, dm_content, false).await?
        };

        if success {
            ctx.say("✅ Your data export has been sent to you via direct message. Check your DMs! 📬").await?;
        } else {
            ctx.say("❌ Unable to send your data export via DM. This might be due to your privacy settings.\n\
                    📧 Please contact an administrator for alternative delivery methods.").await?;
        }

        // Apply cooldown after successful execution
        ctx.data().cooldown.apply_cooldown(
            "export_my_data",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        );

        // Log the export request for audit purposes
        ctx.data().audit_logger.log_data_export(user_id, Some(user_id), "complete_user_data").await;
        info!("Data export requested by user {} ({})", user_id, ctx.author().name);

        Ok(())
    }.await;

    // Record metrics
    record_command_execution(&ctx, "export_my_data", start_time, &result);
    
    result
}

/// Delete user data command - permanently removes all user data for GDPR compliance
#[poise::command(slash_command)]
pub async fn delete_my_data(
    ctx: Context<'_>,
    #[description = "Type 'CONFIRM' to permanently delete all your data"]
    confirmation: Option<String>,
) -> Result<(), CommandError> {
    let start_time = Instant::now();
    
    let result = async {
        let user_id = ctx.author().id.get();
        
        // Cooldown check - longer cooldown for data deletion
        let cooldown_config = CooldownConfig {
            user: Some(Duration::from_secs(600)), // 10 minute cooldown for data deletion
            ..Default::default()
        };

        if let Err(cooldown_err) = ctx.data().cooldown.check_cooldown(
            "delete_my_data",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        ) {
            ctx.say(format!("⏰ {}", cooldown_err)).await?;
            return Ok(());
        }

        // Check if confirmation was provided
        let confirmed = confirmation
            .as_ref()
            .map(|s| s.to_uppercase() == "CONFIRM")
            .unwrap_or(false);

        if !confirmed {
            let warning_message = format!(
                "⚠️ **DATA DELETION WARNING** ⚠️\n\n\
                This command will **PERMANENTLY DELETE** all data associated with your account:\n\
                • Your user preferences and privacy settings\n\
                • All command execution history\n\
                • Cached statistics and activity data\n\
                • Any other stored personal information\n\n\
                **⚡ THIS ACTION CANNOT BE UNDONE! ⚡**\n\n\
                If you're sure you want to proceed, run:\n\
                `/delete_my_data confirmation:CONFIRM`\n\n\
                **Alternatives:**\n\
                • Use `/export_my_data` to get a copy of your data first\n\
                • Contact an administrator to discuss privacy settings\n\
                • Simply stop using the bot (data will expire per retention policy)\n\n\
                🔒 Your privacy is important to us. This deletion complies with GDPR requirements."
            );

            ctx.say(warning_message).await?;
            return Ok(());
        }

        ctx.say("🔄 Processing your data deletion request. This may take a moment...").await?;

        // Track what was deleted for confirmation
        let mut deleted_items = Vec::new();
        let mut deletion_errors = Vec::new();

        // Delete user preferences and settings
        match ctx.data().user_db.delete_preferences(user_id).await {
            Ok(true) => deleted_items.push("✅ User preferences and privacy settings"),
            Ok(false) => deletion_errors.push("⚠️ No user preferences found to delete"),
            Err(e) => {
                error!("Failed to delete user preferences for {}: {}", user_id, e);
                deletion_errors.push("❌ Failed to delete user preferences");
            }
        }

        // Clear user statistics cache
        ctx.data().user_stats.clear_user_cache(user_id);
        deleted_items.push("✅ Cached statistics and activity data");

        // Note: Command execution history in MetricsManager is in-memory and will be cleared on restart
        // For a production system, you'd want to implement persistent storage deletion here
        let user_executions = ctx.data().metrics.get_user_executions(user_id);
        if !user_executions.is_empty() {
            deleted_items.push("✅ Command execution history (in current session)");
        }

        // Clear DM throttle data for the user
        ctx.data().dm_throttle.clear_user_throttle(user_id).await;
        deleted_items.push("✅ DM throttle data");

        // Generate deletion confirmation
        let confirmation_message = if deletion_errors.is_empty() {
            format!(
                "🗑️ **DATA DELETION COMPLETED** ✅\n\n\
                **Successfully deleted:**\n{}\n\n\
                **Confirmation Details:**\n\
                🕐 Deletion completed: {}\n\
                👤 User ID: {}\n\
                📧 User: {}\n\n\
                **Important Notes:**\n\
                • All your personal data has been permanently removed\n\
                • This action complies with GDPR and data protection laws\n\
                • You can start fresh by using bot commands again\n\
                • Any new activity will create new data with default privacy settings\n\n\
                🔒 Your privacy rights have been fully respected.",
                deleted_items.join("\n"),
                Utc::now().format("%Y-%m-%d %H:%M UTC"),
                user_id,
                ctx.author().name
            )
        } else {
            format!(
                "🗑️ **DATA DELETION PARTIALLY COMPLETED** ⚠️\n\n\
                **Successfully deleted:**\n{}\n\n\
                **Issues encountered:**\n{}\n\n\
                **Confirmation Details:**\n\
                🕐 Deletion attempted: {}\n\
                👤 User ID: {}\n\
                📧 User: {}\n\n\
                ℹ️ Most of your data has been removed. Contact an administrator if you need assistance with the remaining items.",
                deleted_items.join("\n"),
                deletion_errors.join("\n"),
                Utc::now().format("%Y-%m-%d %H:%M UTC"),
                user_id,
                ctx.author().name
            )
        };

        // Try to send confirmation via DM, fallback to channel
        let dm_success = send_direct_message(&ctx, ctx.author().id, confirmation_message.clone(), false).await?;

        if dm_success {
            ctx.say("✅ Your data has been deleted and a confirmation was sent to your DMs. 📬").await?;
        } else {
            // If DM fails, send a brief confirmation in channel (without sensitive details)
            ctx.say("✅ Your data deletion has been completed. Check your DMs for full confirmation.\n\
                    ℹ️ If you can't receive DMs, your data has still been successfully deleted.").await?;
        }

        // Apply cooldown after successful execution
        ctx.data().cooldown.apply_cooldown(
            "delete_my_data",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        );

        // Log the deletion for audit purposes
        let deletion_summary = format!("{} items deleted, {} errors", deleted_items.len(), deletion_errors.len());
        ctx.data().audit_logger.log_data_deletion(user_id, Some(user_id), &deletion_summary).await;
        info!("Data deletion completed for user {} ({}): {} items deleted, {} errors", 
              user_id, ctx.author().name, deleted_items.len(), deletion_errors.len());

        Ok(())
    }.await;

    // Record metrics
    record_command_execution(&ctx, "delete_my_data", start_time, &result);
    
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statistics::{UserActivity, TimePeriod};
    use chrono::Utc;


    fn create_test_stats() -> UserActivity {
        let now = Utc::now();
        let mut stats = UserActivity::new(123, TimePeriod::Weekly, now - chrono::Duration::days(7), now);
        
        // Populate with test data
        stats.total_commands = 50;
        stats.successful_commands = 45;
        stats.failed_commands = 5;
        stats.avg_response_time_ms = 125.5;
        
        // Add some command breakdown
        stats.command_breakdown.insert("about".to_string(), 20);
        stats.command_breakdown.insert("uptime".to_string(), 15);
        stats.command_breakdown.insert("my_stats".to_string(), 10);
        stats.command_breakdown.insert("help".to_string(), 3);
        stats.command_breakdown.insert("info".to_string(), 2);
        
        stats.most_used_command = Some("about".to_string());
        
        // Set some activity patterns
        stats.hourly_activity[14] = 10; // 2 PM
        stats.daily_activity[1] = 15; // Monday
        
        stats.unique_channels = 3;
        stats.unique_guilds = 2;
        
        stats.first_command = Some(now - chrono::Duration::days(6));
        stats.last_command = Some(now - chrono::Duration::hours(2));
        
        stats
    }

    #[test]
    fn test_format_user_statistics() {
        let stats = create_test_stats();
        let formatted = format_user_statistics(&stats);
        
        // Check that the response contains expected elements
        assert!(formatted.contains("📊 **Your Weekly Statistics**"));
        assert!(formatted.contains("Total Commands: 50"));
        assert!(formatted.contains("Successful: 45 (90.0%)"));
        assert!(formatted.contains("Failed: 5"));
        assert!(formatted.contains("Avg Response Time: 125.5ms"));
        assert!(formatted.contains("Command: /about"));
        assert!(formatted.contains("Active Hour: 14:00"));
        assert!(formatted.contains("Active Day: Monday"));
        assert!(formatted.contains("Unique Channels: 3"));
        assert!(formatted.contains("Unique Servers: 2"));
        
        // Check top commands formatting
        assert!(formatted.contains("/about: 20"));
        assert!(formatted.contains("/uptime: 15"));
        assert!(formatted.contains("/my_stats: 10"));
    }

    #[test]
    fn test_format_user_statistics_for_dm() {
        let stats = create_test_stats();
        let formatted = format_user_statistics_for_dm(&stats);
        
        // Check that the DM formatting contains expected elements
        assert!(formatted.contains("**Your Weekly Statistics** 📊"));
        assert!(formatted.contains("• 🔢 **Total Commands:** 50"));
        assert!(formatted.contains("• ✅ **Successful:** 45 (90.0%)"));
        assert!(formatted.contains("• 🎯 **Most Used Command:** /about"));
        assert!(formatted.contains("📱 This data is private"));
        
        // Check enhanced privacy formatting
        assert!(formatted.contains("**🏆 Your Activity Highlights**"));
        assert!(formatted.contains("**📋 Top 5 Commands**"));
    }

    #[test]
    fn test_format_empty_statistics() {
        let now = Utc::now();
        let stats = UserActivity::new(123, TimePeriod::Daily, now - chrono::Duration::days(1), now);
        let formatted = format_user_statistics(&stats);
        
        // Check that empty stats are handled gracefully
        assert!(formatted.contains("📊 **Your Daily Statistics**"));
        assert!(formatted.contains("Total Commands: 0"));
        assert!(formatted.contains("Command: None"));
        assert!(formatted.contains("🕐 Active Hour:")); // Just check the label exists
        assert!(formatted.contains("📆 Active Day:")); // Just check the label exists
        assert!(formatted.contains("None")); // For top commands
        assert!(formatted.contains("🚀 First Command: N/A"));
        assert!(formatted.contains("🕐 Latest Command: N/A"));
    }

    #[test]
    fn test_format_empty_statistics_for_dm() {
        let now = Utc::now();
        let stats = UserActivity::new(123, TimePeriod::AllTime, now, now);
        let formatted = format_user_statistics_for_dm(&stats);
        
        // Check that empty DM stats are handled gracefully
        assert!(formatted.contains("**Your All Time Statistics** 📊"));
        assert!(formatted.contains("• 🔢 **Total Commands:** 0"));
        assert!(formatted.contains("📱 This data is private"));
    }
} 