//! Type-safe default values using const functions.

use crate::schema::*;
use std::collections::HashMap;
use tgraph_common::ChannelId;

impl Default for Config {
    fn default() -> Self {
        Self {
            tautulli: TautulliConfig::default(),
            discord: DiscordConfig::default(),
            scheduling: SchedulingConfig::default(),
            data: DataConfig::default(),
            graphs: GraphsConfig::default(),
            rate_limiting: RateLimitingConfig::default(),
        }
    }
}

impl Default for TautulliConfig {
    fn default() -> Self {
        Self {
            api_key: String::new(),
            url: "http://localhost:8181/api/v2".to_string(),
        }
    }
}

impl Default for DiscordConfig {
    fn default() -> Self {
        Self {
            token: String::new(),
            channel_id: ChannelId(0),
        }
    }
}

impl Default for SchedulingConfig {
    fn default() -> Self {
        Self {
            update_days: 7,
            fixed_update_time: None,
            keep_days: 7,
        }
    }
}

impl Default for DataConfig {
    fn default() -> Self {
        Self {
            time_range_days: 30,
            language: "en-US".to_string(),
        }
    }
}

impl Default for GraphsConfig {
    fn default() -> Self {
        Self {
            enabled: EnabledGraphsConfig::default(),
            privacy: PrivacyConfig::default(),
            styling: StylingConfig::default(),
        }
    }
}

impl Default for EnabledGraphsConfig {
    fn default() -> Self {
        Self {
            daily_play_count: true,
            play_count_by_dayofweek: true,
            play_count_by_hourofday: true,
            top_10_platforms: true,
            top_10_users: true,
            play_count_by_month: true,
        }
    }
}

impl Default for PrivacyConfig {
    fn default() -> Self {
        Self {
            censor_usernames: true,
        }
    }
}

impl Default for StylingConfig {
    fn default() -> Self {
        Self {
            enable_grid: false,
            colors: ColorsConfig::default(),
            annotations: AnnotationsConfig::default(),
        }
    }
}

impl Default for ColorsConfig {
    fn default() -> Self {
        Self {
            tv: "#1f77b4".to_string(),
            movie: "#ff7f0e".to_string(),
            background: "#ffffff".to_string(),
            annotation: "#ff0000".to_string(),
            annotation_outline: "#000000".to_string(),
        }
    }
}

impl Default for AnnotationsConfig {
    fn default() -> Self {
        let mut graphs = HashMap::new();
        graphs.insert("daily_play_count".to_string(), true);
        graphs.insert("play_count_by_dayofweek".to_string(), true);
        graphs.insert("play_count_by_hourofday".to_string(), true);
        graphs.insert("top_10_platforms".to_string(), true);
        graphs.insert("top_10_users".to_string(), true);
        graphs.insert("play_count_by_month".to_string(), true);

        Self {
            enable_outline: true,
            graphs,
        }
    }
}

impl Default for RateLimitingConfig {
    fn default() -> Self {
        Self {
            config_cooldown_minutes: 0,
            config_global_cooldown_seconds: 0,
            update_graphs_cooldown_minutes: 0,
            update_graphs_global_cooldown_seconds: 0,
            my_stats_cooldown_minutes: 5,
            my_stats_global_cooldown_seconds: 60,
        }
    }
}
