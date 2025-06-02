//! Shared utility functions with zero-cost abstractions.

use chrono::{DateTime, Utc};

/// Formats a timestamp for display.
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Sanitizes a string for safe display.
pub fn sanitize_string(input: &str) -> String {
    input
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace() || "-_.,!?'".contains(*c))
        .collect()
}

/// Truncates a string to a maximum length with ellipsis.
pub fn truncate_string(input: &str, max_length: usize) -> String {
    if input.len() <= max_length {
        input.to_string()
    } else {
        format!("{}...", &input[..max_length.saturating_sub(3)])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

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
}
