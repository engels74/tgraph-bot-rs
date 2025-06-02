//! Test utilities and shared test helpers for TGraph Bot Rust Edition.
//!
//! This module provides common testing utilities, fixtures, and helper functions
//! that can be used across all crates in the workspace for unit and integration testing.

use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use std::sync::Once;
use tokio::runtime::Runtime;

#[cfg(feature = "tracing-subscriber")]
use tracing_subscriber::{fmt, EnvFilter};

/// Initialize test logging once per test run.
static INIT: Once = Once::new();

/// Initialize logging for tests with a sensible default configuration.
/// This function is safe to call multiple times and will only initialize once.
#[cfg(feature = "tracing-subscriber")]
pub fn init_test_logging() {
    INIT.call_once(|| {
        let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("debug"));

        fmt().with_test_writer().with_env_filter(filter).init();
    });
}

/// No-op version when tracing-subscriber is not available
#[cfg(not(feature = "tracing-subscriber"))]
pub fn init_test_logging() {
    // No-op when tracing-subscriber is not available
}

/// Create a tokio runtime for testing async functions.
/// This is useful for tests that need to run async code in a synchronous test context.
pub fn create_test_runtime() -> Runtime {
    Runtime::new().expect("Failed to create test runtime")
}

/// Test fixture for creating a mock timestamp.
pub fn mock_timestamp(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    min: u32,
    sec: u32,
) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, min, sec)
        .unwrap()
}

/// Create a temporary directory for tests that automatically cleans up.
#[cfg(feature = "tempfile")]
pub fn create_temp_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create temporary directory")
}

/// Create a temporary file for tests that automatically cleans up.
#[cfg(feature = "tempfile")]
pub fn create_temp_file() -> tempfile::NamedTempFile {
    tempfile::NamedTempFile::new().expect("Failed to create temporary file")
}

/// Assert that two floating point numbers are approximately equal within a tolerance.
pub fn assert_approx_eq(left: f64, right: f64, tolerance: f64) {
    let diff = (left - right).abs();
    assert!(
        diff <= tolerance,
        "assertion failed: `{left}` is not approximately equal to `{right}` (tolerance: {tolerance}, diff: {diff})"
    );
}

/// Generate test data for graph generation testing.
pub mod graph_fixtures {
    use super::*;
    use std::collections::HashMap;

    /// Sample data point for testing graph generation.
    #[derive(Debug, Clone)]
    pub struct TestDataPoint {
        pub timestamp: DateTime<Utc>,
        pub value: f64,
        pub category: String,
    }

    /// Generate sample time series data for testing.
    pub fn generate_time_series(count: usize, start_date: DateTime<Utc>) -> Vec<TestDataPoint> {
        (0..count)
            .map(|i| TestDataPoint {
                timestamp: start_date + chrono::Duration::days(i as i64),
                value: (i as f64 * 1.5) + (i as f64).sin() * 10.0,
                category: format!("category_{}", i % 3),
            })
            .collect()
    }

    /// Generate sample user activity data.
    pub fn generate_user_activity_data() -> HashMap<String, u64> {
        let mut data = HashMap::new();
        data.insert("user1".to_string(), 42);
        data.insert("user2".to_string(), 38);
        data.insert("user3".to_string(), 25);
        data.insert("user4".to_string(), 15);
        data.insert("user5".to_string(), 12);
        data
    }

    /// Generate sample platform usage data.
    pub fn generate_platform_data() -> HashMap<String, u64> {
        let mut data = HashMap::new();
        data.insert("Plex Web".to_string(), 35);
        data.insert("Plex for Android".to_string(), 28);
        data.insert("Plex for iOS".to_string(), 22);
        data.insert("Plex for Roku".to_string(), 18);
        data.insert("Plex for Samsung TV".to_string(), 12);
        data
    }
}

/// Discord-related test utilities.
pub mod discord_fixtures {
    use crate::{ChannelId, UserId};

    /// Create a test channel ID.
    pub fn test_channel_id() -> ChannelId {
        ChannelId(123456789012345678)
    }

    /// Create a test user ID.
    pub fn test_user_id() -> UserId {
        UserId(987654321098765432)
    }

    /// Create multiple test user IDs.
    pub fn test_user_ids(count: usize) -> Vec<UserId> {
        (0..count)
            .map(|i| UserId(100000000000000000 + i as u64))
            .collect()
    }
}

/// Configuration-related test utilities.
pub mod config_fixtures {
    /// Create a minimal valid test configuration as YAML string.
    pub fn minimal_config_yaml() -> &'static str {
        r#"
tautulli:
  api_key: "test_api_key"
  url: "http://localhost:8181/api/v2"

discord:
  token: "test_token"
  channel_id: "123456789012345678"

scheduling:
  update_days: 7
  keep_days: 7

data:
  time_range_days: 30
  language: "en-US"

graphs:
  enabled:
    daily_play_count: true
    play_count_by_dayofweek: true
    play_count_by_hourofday: true
    top_10_platforms: true
    top_10_users: true
    play_count_by_month: true
"#
    }

    /// Create a full test configuration as YAML string.
    pub fn full_config_yaml() -> &'static str {
        concat!(
            "tautulli:\n",
            "  api_key: \"test_api_key_full\"\n",
            "  url: \"http://localhost:8181/api/v2\"\n",
            "\n",
            "discord:\n",
            "  token: \"test_token_full\"\n",
            "  channel_id: \"123456789012345678\"\n",
            "\n",
            "scheduling:\n",
            "  update_days: 7\n",
            "  fixed_update_time: \"14:30\"\n",
            "  keep_days: 7\n",
            "\n",
            "data:\n",
            "  time_range_days: 30\n",
            "  language: \"en-US\"\n",
            "\n",
            "graphs:\n",
            "  enabled:\n",
            "    daily_play_count: true\n",
            "    play_count_by_dayofweek: true\n",
            "    play_count_by_hourofday: true\n",
            "    top_10_platforms: true\n",
            "    top_10_users: true\n",
            "    play_count_by_month: true\n",
            "\n",
            "  privacy:\n",
            "    censor_usernames: true\n",
            "\n",
            "  styling:\n",
            "    enable_grid: false\n",
            "    colors:\n",
            "      tv: \"#1f77b4\"\n",
            "      movie: \"#ff7f0e\"\n",
            "      background: \"#ffffff\"\n",
            "      annotation: \"#ff0000\"\n",
            "      annotation_outline: \"#000000\"\n",
            "\n",
            "    annotations:\n",
            "      enable_outline: true\n",
            "      graphs:\n",
            "        daily_play_count: true\n",
            "        play_count_by_dayofweek: true\n",
            "        play_count_by_hourofday: true\n",
            "        top_10_platforms: true\n",
            "        top_10_users: true\n",
            "        play_count_by_month: true\n",
            "\n",
            "rate_limiting:\n",
            "  config_cooldown_minutes: 0\n",
            "  config_global_cooldown_seconds: 0\n",
            "  update_graphs_cooldown_minutes: 0\n",
            "  update_graphs_global_cooldown_seconds: 0\n",
            "  my_stats_cooldown_minutes: 5\n",
            "  my_stats_global_cooldown_seconds: 60\n"
        )
    }
}

/// Property-based testing utilities using proptest.
#[cfg(feature = "proptest")]
pub mod property_testing {
    use crate::{ChannelId, TautulliUserId, UserId};
    use proptest::prelude::*;

    /// Strategy for generating valid Discord channel IDs.
    pub fn channel_id_strategy() -> impl Strategy<Value = ChannelId> {
        (100000000000000000u64..=999999999999999999u64).prop_map(ChannelId)
    }

    /// Strategy for generating valid Discord user IDs.
    pub fn user_id_strategy() -> impl Strategy<Value = UserId> {
        (100000000000000000u64..=999999999999999999u64).prop_map(UserId)
    }

    /// Strategy for generating valid Tautulli user IDs.
    pub fn tautulli_user_id_strategy() -> impl Strategy<Value = TautulliUserId> {
        (1u64..=999999u64).prop_map(TautulliUserId)
    }

    /// Strategy for generating valid username strings.
    pub fn username_strategy() -> impl Strategy<Value = String> {
        r"[a-zA-Z0-9_]{3,32}".prop_map(|s| s.to_string())
    }

    /// Strategy for generating valid API keys.
    pub fn api_key_strategy() -> impl Strategy<Value = String> {
        r"[a-fA-F0-9]{32}".prop_map(|s| s.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_init_logging_multiple_calls() {
        // Should not panic when called multiple times
        init_test_logging();
        init_test_logging();
        init_test_logging();
    }

    #[test]
    fn test_mock_timestamp() {
        let timestamp = mock_timestamp(2024, 1, 1, 12, 0, 0);
        assert_eq!(timestamp.year(), 2024);
        assert_eq!(timestamp.month(), 1);
        assert_eq!(timestamp.day(), 1);
        assert_eq!(timestamp.hour(), 12);
    }

    #[test]
    fn test_assert_approx_eq() {
        assert_approx_eq(1.0, 1.0001, 0.001);
        assert_approx_eq(1.0, 0.9999, 0.001);
    }

    #[test]
    #[should_panic]
    fn test_assert_approx_eq_fails() {
        assert_approx_eq(1.0, 1.1, 0.05);
    }

    #[test]
    fn test_create_test_runtime() {
        let runtime = create_test_runtime();
        let result = runtime.block_on(async { 42 });
        assert_eq!(result, 42);
        // Runtime is dropped here outside of async context
    }

    #[cfg(feature = "proptest")]
    mod property_tests {
        use super::*;
        use proptest::prelude::*;

        proptest! {
            #[test]
            fn test_property_channel_id_display(id in property_testing::channel_id_strategy()) {
                let displayed = format!("{}", id);
                let parsed: u64 = displayed.parse().unwrap();
                assert_eq!(id.0, parsed);
            }

            #[test]
            fn test_property_username_valid(username in property_testing::username_strategy()) {
                assert!(username.len() >= 3);
                assert!(username.len() <= 32);
                assert!(username.chars().all(|c| c.is_alphanumeric() || c == '_'));
            }
        }
    }
}
