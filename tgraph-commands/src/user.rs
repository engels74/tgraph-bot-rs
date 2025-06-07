//! User commands for the TGraph bot

use crate::context::{Context, CommandError, record_command_execution};
use crate::cooldown::CooldownConfig;
use crate::statistics::TimePeriod;
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

        // Get user's command execution history
        let user_executions = ctx.data().metrics.get_user_executions(user_id);

        // Get user statistics for the specified period
        match ctx.data().user_stats.get_user_statistics(user_id, time_period, &user_executions).await {
            Ok(Some(stats)) => {
                // Format the statistics into a user-friendly response
                let response = format_user_statistics(&stats);
                ctx.say(response).await?;
            }
            Ok(None) => {
                // User has no statistics or privacy settings prevent display
                let response = match time_period {
                    TimePeriod::AllTime => {
                        "📊 **Your Statistics**\n\
                        🔍 No command usage found yet. Start using commands to see your statistics!"
                    }
                    _ => {
                        "📊 **Your Statistics**\n\
                        🔍 No command usage found for this time period. Try a different period or use more commands!"
                    }
                };
                ctx.say(response).await?;
            }
            Err(e) => {
                tracing::error!("Failed to get user statistics for user {}: {}", user_id, e);
                ctx.say("❌ Failed to retrieve your statistics. Please try again later.").await?;
                return Ok(());
            }
        }

        // Apply cooldown after successful execution
        ctx.data().cooldown.apply_cooldown(
            "my_stats",
            ctx.author().id,
            Some(ctx.channel_id()),
            &cooldown_config,
        );

        info!("My stats command executed by user {} for period {:?}", user_id, time_period);
        Ok(())
    }.await;

    // Record metrics
    record_command_execution(&ctx, "my_stats", start_time, &result);
    
    result
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::statistics::UserActivity;
    use chrono::Utc;

    fn create_test_stats() -> UserActivity {
        let now = Utc::now();
        let mut stats = UserActivity::new(123, TimePeriod::Weekly, now - chrono::Duration::days(7), now);
        
        // Add some test data
        stats.total_commands = 50;
        stats.successful_commands = 45;
        stats.failed_commands = 5;
        stats.avg_response_time_ms = 125.5;
        stats.most_used_command = Some("about".to_string());
        
        // Add command breakdown
        stats.command_breakdown.insert("about".to_string(), 20);
        stats.command_breakdown.insert("uptime".to_string(), 15);
        stats.command_breakdown.insert("my_stats".to_string(), 10);
        stats.command_breakdown.insert("help".to_string(), 3);
        stats.command_breakdown.insert("info".to_string(), 2);
        
        // Add activity patterns
        stats.hourly_activity[14] = 15; // Most active at 2 PM
        stats.daily_activity[1] = 25;   // Most active on Monday
        
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
    fn test_format_empty_statistics() {
        let now = Utc::now();
        let stats = UserActivity::new(123, TimePeriod::Daily, now - chrono::Duration::days(1), now);
        let formatted = format_user_statistics(&stats);
        
        // Check that empty stats are handled gracefully
        assert!(formatted.contains("📊 **Your Daily Statistics**"));
        assert!(formatted.contains("Total Commands: 0"));
        assert!(formatted.contains("Command: None"));
        assert!(formatted.contains("Active Hour: N/A"));
        assert!(formatted.contains("Active Day: N/A"));
        assert!(formatted.contains("None")); // For top commands
    }

    #[test]
    fn test_format_all_time_statistics() {
        let now = Utc::now();
        let mut stats = UserActivity::new(123, TimePeriod::AllTime, now - chrono::Duration::days(365), now);
        stats.total_commands = 1000;
        stats.successful_commands = 950;
        stats.failed_commands = 50;
        
        let formatted = format_user_statistics(&stats);
        
        assert!(formatted.contains("📊 **Your All Time Statistics**"));
        assert!(formatted.contains("**Period:** All Time"));
        assert!(formatted.contains("Total Commands: 1000"));
        assert!(formatted.contains("(95.0%)"));
    }
} 