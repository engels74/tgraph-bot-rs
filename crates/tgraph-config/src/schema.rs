//! Configuration schema definitions using serde with validation attributes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tgraph_common::{ChannelId, TGraphError};

/// Main configuration structure for TGraph Bot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Tautulli configuration.
    pub tautulli: TautulliConfig,
    /// Discord configuration.
    pub discord: DiscordConfig,
    /// Scheduling configuration.
    pub scheduling: SchedulingConfig,
    /// Data configuration.
    pub data: DataConfig,
    /// Graph configuration.
    pub graphs: GraphsConfig,
    /// Rate limiting configuration.
    pub rate_limiting: RateLimitingConfig,
}

/// Tautulli API configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TautulliConfig {
    /// Tautulli API key.
    pub api_key: String,
    /// Tautulli API URL.
    pub url: String,
}

/// Discord bot configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscordConfig {
    /// Discord bot token.
    pub token: String,
    /// Discord channel ID for posting graphs.
    pub channel_id: ChannelId,
}

/// Scheduling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulingConfig {
    /// Number of days between updates.
    pub update_days: u32,
    /// Fixed update time in HH:MM format.
    pub fixed_update_time: Option<String>,
    /// Number of days to keep old graphs.
    pub keep_days: u32,
}

/// Data configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConfig {
    /// Time range in days for data collection.
    pub time_range_days: u32,
    /// Language code for localization.
    pub language: String,
}

/// Graph configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphsConfig {
    /// Enabled graphs configuration.
    pub enabled: EnabledGraphsConfig,
    /// Privacy configuration.
    pub privacy: PrivacyConfig,
    /// Styling configuration.
    pub styling: StylingConfig,
}

/// Enabled graphs configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnabledGraphsConfig {
    /// Daily play count graph.
    pub daily_play_count: bool,
    /// Play count by day of week graph.
    pub play_count_by_dayofweek: bool,
    /// Play count by hour of day graph.
    pub play_count_by_hourofday: bool,
    /// Top 10 platforms graph.
    pub top_10_platforms: bool,
    /// Top 10 users graph.
    pub top_10_users: bool,
    /// Play count by month graph.
    pub play_count_by_month: bool,
}

/// Privacy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyConfig {
    /// Whether to censor usernames in graphs.
    pub censor_usernames: bool,
}

/// Styling configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StylingConfig {
    /// Whether to enable grid in graphs.
    pub enable_grid: bool,
    /// Color configuration.
    pub colors: ColorsConfig,
    /// Annotations configuration.
    pub annotations: AnnotationsConfig,
}

/// Color configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorsConfig {
    /// TV show color.
    pub tv: String,
    /// Movie color.
    pub movie: String,
    /// Background color.
    pub background: String,
    /// Annotation color.
    pub annotation: String,
    /// Annotation outline color.
    pub annotation_outline: String,
}

/// Annotations configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationsConfig {
    /// Whether to enable annotation outlines.
    pub enable_outline: bool,
    /// Per-graph annotation settings.
    pub graphs: HashMap<String, bool>,
}

/// Rate limiting configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitingConfig {
    /// Config command cooldown in minutes.
    pub config_cooldown_minutes: u32,
    /// Config command global cooldown in seconds.
    pub config_global_cooldown_seconds: u32,
    /// Update graphs command cooldown in minutes.
    pub update_graphs_cooldown_minutes: u32,
    /// Update graphs command global cooldown in seconds.
    pub update_graphs_global_cooldown_seconds: u32,
    /// My stats command cooldown in minutes.
    pub my_stats_cooldown_minutes: u32,
    /// My stats command global cooldown in seconds.
    pub my_stats_global_cooldown_seconds: u32,
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), TGraphError> {
        // Basic validation - more comprehensive validation will be added later
        if self.tautulli.api_key.is_empty() {
            return Err(TGraphError::Config(
                "Tautulli API key cannot be empty".to_string(),
            ));
        }

        if self.discord.token.is_empty() {
            return Err(TGraphError::Config(
                "Discord token cannot be empty".to_string(),
            ));
        }

        Ok(())
    }
}
