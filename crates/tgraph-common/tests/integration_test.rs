//! Integration tests for tgraph-common crate.

use chrono::{TimeZone, Utc};
use tgraph_common::{
    format_timestamp, sanitize_string, truncate_string, ChannelId, TautulliUserId, UserId,
};

#[test]
fn test_channel_id_display() {
    let channel_id = ChannelId(123456789);
    assert_eq!(format!("{}", channel_id), "123456789");
}

#[test]
fn test_user_id_display() {
    let user_id = UserId(987654321);
    assert_eq!(format!("{}", user_id), "987654321");
}

#[test]
fn test_tautulli_user_id_display() {
    let tautulli_id = TautulliUserId(555);
    assert_eq!(format!("{}", tautulli_id), "555");
}

#[test]
fn test_format_timestamp() {
    let timestamp = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    let formatted = format_timestamp(timestamp);
    assert_eq!(formatted, "2024-01-01 12:00:00 UTC");
}

#[test]
fn test_sanitize_string() {
    let input = "Hello, World! <script>alert('xss')</script>";
    let sanitized = sanitize_string(input);
    assert_eq!(sanitized, "Hello, World! scriptalert'xss'script");
}

#[test]
fn test_truncate_string() {
    let input = "This is a very long string that should be truncated";
    let truncated = truncate_string(input, 20);
    assert_eq!(truncated, "This is a very lo...");

    let short = "Short";
    let not_truncated = truncate_string(short, 20);
    assert_eq!(not_truncated, "Short");
}
