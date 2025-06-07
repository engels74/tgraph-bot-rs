//! Command context and framework integration

use std::sync::Arc;
use std::time::{Duration, Instant};
use tgraph_config::Config;
use tgraph_i18n::I18nManager;
use crate::{Permissions, CooldownManager, MetricsManager, UserDatabase, UserStatisticsManager};
use tracing::info;
use tokio::time::interval;

/// Shared application state accessible across commands and event handlers
#[derive(Debug)]
pub struct CommandContext {
    /// Application configuration
    pub config: Arc<Config>,
    /// HTTP client for external API calls
    pub http_client: reqwest::Client,
    /// Internationalization manager
    pub i18n: Arc<I18nManager>,
    /// Permission manager
    pub permissions: Arc<Permissions>,
    /// Cooldown manager
    pub cooldown: Arc<CooldownManager>,
    /// Metrics and usage tracking manager
    pub metrics: Arc<MetricsManager>,
    /// User database for preferences and privacy settings
    pub user_db: Arc<UserDatabase>,
    /// User statistics manager with caching and privacy controls
    pub user_stats: Arc<UserStatisticsManager>,
}

/// Error type for commands
pub type CommandError = Box<dyn std::error::Error + Send + Sync>;

/// Poise context type alias
pub type Context<'a> = poise::Context<'a, CommandContext, CommandError>;

/// Helper function to record command execution
pub fn record_command_execution(
    ctx: &Context<'_>,
    command_name: &str,
    start_time: Instant,
    result: &Result<(), CommandError>,
) {
    let duration = start_time.elapsed();
    let success = result.is_ok();
    let error = result.as_ref().err().map(|e| e.to_string());
    
    // Get guild ID from context
    let guild_id = ctx.guild_id().map(|g| g.get());
    
    // Create metadata with additional context
    let metadata = serde_json::json!({
        "guild_id": guild_id,
        "command_name": command_name,
        "is_slash_command": true,
        "discord_context": "slash_command"
    });

    ctx.data().metrics.record_execution(
        command_name,
        ctx.author().id,
        Some(ctx.channel_id()),
        guild_id,
        duration,
        success,
        error,
        metadata,
    );
}

/// Macro to wrap command execution with automatic metrics recording
#[macro_export]
macro_rules! with_metrics {
    ($ctx:expr, $command_name:expr, $body:block) => {{
        let start_time = std::time::Instant::now();
        let result = $body;
        $crate::context::record_command_execution(&$ctx, $command_name, start_time, &result);
        result
    }};
}

/// Start background task for periodic metrics cleanup
pub fn start_metrics_cleanup_task(metrics: Arc<MetricsManager>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(3600)); // Run every hour
        
        loop {
            interval.tick().await;
            
            // Clean up metrics older than 30 days
            metrics.cleanup_old_executions(30);
            
            // Log current metrics summary
            let (total, successes, _failures) = metrics.get_global_counts();
            if total > 0 {
                info!(
                    "Metrics cleanup complete. Total executions: {}, Success rate: {:.1}%", 
                    total, 
                    (successes as f64 / total as f64) * 100.0
                );
            }
        }
    })
}

/// Create a new command context with all required components
pub async fn create_command_context(config: Config) -> Result<CommandContext, CommandError> {
    // Initialize HTTP client
    let http_client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(config.discord.request_timeout_seconds))
        .build()?;

    // Initialize i18n
    let i18n = I18nManager::new(tgraph_i18n::Locale::default());

    // Initialize permissions from config
    let permissions = Permissions::new(&config);

    // Initialize cooldown manager
    let cooldown = CooldownManager::new();

    // Initialize metrics manager
    let metrics = MetricsManager::new();

    // Initialize user database
    let db_path = std::env::current_dir()?.join("data").join("user_preferences.db");
    std::fs::create_dir_all(db_path.parent().unwrap())?;
    let user_db = Arc::new(UserDatabase::new(db_path)?);

    // Initialize user statistics manager
    let user_stats = Arc::new(UserStatisticsManager::new(user_db.clone()));

    // Start background cleanup task
    let metrics_arc = Arc::new(metrics);
    let _cleanup_handle = start_metrics_cleanup_task(metrics_arc.clone());

    Ok(CommandContext {
        config: Arc::new(config),
        http_client,
        i18n: Arc::new(i18n),
        permissions: Arc::new(permissions),
        cooldown: Arc::new(cooldown),
        metrics: metrics_arc,
        user_db,
        user_stats,
    })
} 