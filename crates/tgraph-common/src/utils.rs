//! Shared utility functions with zero-cost abstractions.

use chrono::{DateTime, Duration as ChronoDuration, Utc};
use chrono_tz::Tz;
use std::future::Future;
use std::time::Duration;
use tokio::time::timeout;
use unicode_bidi::{BidiInfo, Level};
use unicode_normalization::UnicodeNormalization;
use unicode_segmentation::UnicodeSegmentation;

use crate::types::UtilsError;

// =============================================================================
// Date/Time Utilities with Edge Case Handling
// =============================================================================

/// Validates that a time range is sensible (start < end).
///
/// Returns an error if the start time is after or equal to the end time.
pub fn validate_time_range(start: DateTime<Utc>, end: DateTime<Utc>) -> Result<(), UtilsError> {
    if start >= end {
        return Err(UtilsError::InvalidTimeRange {
            start: start.to_rfc3339(),
            end: end.to_rfc3339(),
        });
    }
    Ok(())
}

/// Converts a UTC timestamp to a display timezone with proper error handling.
///
/// Supports all IANA timezone names and handles edge cases like DST transitions.
pub fn convert_to_display_timezone(
    utc_time: DateTime<Utc>,
    timezone: &str,
) -> Result<DateTime<chrono_tz::Tz>, UtilsError> {
    let tz: Tz = timezone.parse().map_err(|_| UtilsError::InvalidTimezone {
        timezone: timezone.to_string(),
    })?;

    Ok(utc_time.with_timezone(&tz))
}

/// Formats a duration in a human-readable way, handling edge cases.
///
/// Handles zero durations, negative durations, and very large durations appropriately.
pub fn format_duration_human_readable(duration: ChronoDuration) -> String {
    if duration.is_zero() {
        return "0 seconds".to_string();
    }

    if duration < ChronoDuration::zero() {
        let positive_duration = -duration;
        return format!("{} ago", format_duration_human_readable(positive_duration));
    }

    let total_seconds = duration.num_seconds();
    let days = duration.num_days();
    let years = days / 365;
    let remaining_days = days % 365;
    let hours = (total_seconds % (24 * 3600)) / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;

    let mut parts = Vec::new();

    if years > 0 {
        parts.push(format!(
            "{} year{}",
            years,
            if years == 1 { "" } else { "s" }
        ));
    }
    if remaining_days > 0 {
        parts.push(format!(
            "{} day{}",
            remaining_days,
            if remaining_days == 1 { "" } else { "s" }
        ));
    }
    if hours > 0 {
        parts.push(format!(
            "{} hour{}",
            hours,
            if hours == 1 { "" } else { "s" }
        ));
    }
    if minutes > 0 {
        parts.push(format!(
            "{} minute{}",
            minutes,
            if minutes == 1 { "" } else { "s" }
        ));
    }
    if seconds > 0 || parts.is_empty() {
        parts.push(format!(
            "{} second{}",
            seconds,
            if seconds == 1 { "" } else { "s" }
        ));
    }

    match parts.len() {
        1 => parts[0].clone(),
        2 => format!("{} and {}", parts[0], parts[1]),
        _ => {
            let last = parts.pop().unwrap();
            format!("{}, and {}", parts.join(", "), last)
        }
    }
}

/// Formats how long ago a timestamp was, handling future times.
pub fn format_time_ago(timestamp: DateTime<Utc>, reference: DateTime<Utc>) -> String {
    let duration = reference.signed_duration_since(timestamp);

    if duration < ChronoDuration::zero() {
        let future_duration = -duration;
        return format!(
            "{} from now",
            format_duration_human_readable(future_duration)
        );
    }

    format!("{} ago", format_duration_human_readable(duration))
}

/// Formats a timestamp for display.
pub fn format_timestamp(timestamp: DateTime<Utc>) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

// =============================================================================
// Unicode-Safe String Manipulation
// =============================================================================

/// Truncates a string safely without breaking Unicode character boundaries.
///
/// Ensures that the result is always valid UTF-8 and respects grapheme cluster boundaries.
pub fn truncate_string_unicode_safe(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }

    let graphemes: Vec<&str> = input.graphemes(true).collect();

    if graphemes.len() <= max_chars {
        input.to_string()
    } else if max_chars <= 3 {
        // If max_chars is very small, just return dots
        "...".chars().take(max_chars).collect()
    } else {
        // Take max_chars - 3 graphemes, then add "..." (3 chars)
        let truncated: String = graphemes.iter().take(max_chars - 3).map(|&s| s).collect();
        truncated + "..."
    }
}

/// Normalizes Unicode string to NFC form for consistent comparisons.
///
/// Ensures that different Unicode representations of the same text are handled consistently.
pub fn normalize_unicode_string(input: &str) -> String {
    input.nfc().collect()
}

/// Sanitizes a string while preserving all valid Unicode characters.
///
/// More permissive than the basic sanitize_string, preserving international text.
pub fn sanitize_string_unicode_safe(input: &str) -> String {
    input
        .chars()
        .filter(|c| {
            // Keep letters, numbers, whitespace, basic punctuation, and emoji
            c.is_alphanumeric()
                || c.is_whitespace()
                || matches!(
                    *c,
                    '.' | ','
                        | '!'
                        | '?'
                        | '\''
                        | '-'
                        | '_'
                        | ':'
                        | ';'
                        | '('
                        | ')'
                        | '['
                        | ']'
                        | '{'
                        | '}'
                )
                || !c.is_control() // Allow all non-control characters
        })
        .collect()
}

/// Counts grapheme clusters in a string (what users perceive as characters).
///
/// Properly handles complex emoji and combining characters as single units.
pub fn count_graphemes(input: &str) -> usize {
    input.graphemes(true).count()
}

/// Sanitizes bidirectional text to prevent text direction attacks.
///
/// Removes potentially dangerous bidirectional override characters.
pub fn sanitize_bidi_text(input: &str) -> String {
    let _bidi_info = BidiInfo::new(input, Some(Level::ltr()));

    // Remove dangerous bidirectional control characters
    input
        .chars()
        .filter(|&c| {
            !matches!(
                c,
                '\u{202A}' | // LEFT-TO-RIGHT EMBEDDING
                '\u{202B}' | // RIGHT-TO-LEFT EMBEDDING
                '\u{202C}' | // POP DIRECTIONAL FORMATTING
                '\u{202D}' | // LEFT-TO-RIGHT OVERRIDE
                '\u{202E}' | // RIGHT-TO-LEFT OVERRIDE
                '\u{2066}' | // LEFT-TO-RIGHT ISOLATE
                '\u{2067}' | // RIGHT-TO-LEFT ISOLATE
                '\u{2068}' | // FIRST STRONG ISOLATE
                '\u{2069}' // POP DIRECTIONAL ISOLATE
            )
        })
        .collect()
}

/// Original sanitize function for backward compatibility.
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

// =============================================================================
// Async Utilities with Proper Cancellation Handling
// =============================================================================

/// Wraps any future with a timeout, providing consistent cancellation behavior.
pub async fn with_timeout<F, T>(duration: Duration, future: F) -> Result<T, UtilsError>
where
    F: Future<Output = T>,
{
    timeout(duration, future)
        .await
        .map_err(|_| UtilsError::Timeout {
            duration_ms: duration.as_millis() as u64,
        })
}

/// Configuration for retry operations with exponential backoff.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    max_attempts: u32,
    initial_delay: Duration,
    max_delay: Duration,
    backoff_multiplier: f64,
}

impl RetryConfig {
    /// Creates a new retry configuration with sensible defaults.
    pub fn new() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }

    /// Sets the maximum number of attempts.
    pub fn max_attempts(mut self, attempts: u32) -> Self {
        self.max_attempts = attempts;
        self
    }

    /// Sets the initial delay between attempts.
    pub fn initial_delay(mut self, delay: Duration) -> Self {
        self.initial_delay = delay;
        self
    }

    /// Sets the maximum delay between attempts.
    pub fn max_delay(mut self, delay: Duration) -> Self {
        self.max_delay = delay;
        self
    }

    /// Sets the backoff multiplier.
    pub fn backoff_multiplier(mut self, multiplier: f64) -> Self {
        self.backoff_multiplier = multiplier;
        self
    }
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Retries an operation with exponential backoff, respecting cancellation.
pub async fn retry_with_exponential_backoff<F, Fut, T, E>(
    mut operation: F,
    config: RetryConfig,
) -> Result<T, UtilsError>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut current_delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                last_error = Some(format!("{:?}", error));

                if attempt < config.max_attempts {
                    tokio::time::sleep(current_delay).await;

                    // Calculate next delay with exponential backoff
                    let next_delay_ms =
                        (current_delay.as_millis() as f64 * config.backoff_multiplier) as u64;
                    current_delay = Duration::from_millis(next_delay_ms).min(config.max_delay);
                }
            }
        }
    }

    Err(UtilsError::RetryFailed {
        attempts: config.max_attempts,
        last_error: last_error.unwrap_or_else(|| "Unknown error".to_string()),
    })
}

/// A guard that ensures cleanup runs even if a task is cancelled.
pub struct CleanupGuard<F>
where
    F: FnOnce(),
{
    cleanup: Option<F>,
}

impl<F> CleanupGuard<F>
where
    F: FnOnce(),
{
    /// Creates a new cleanup guard with the given cleanup function.
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }

    /// Disarms the guard, preventing cleanup from running.
    pub fn disarm(&mut self) {
        self.cleanup.take();
    }
}

impl<F> Drop for CleanupGuard<F>
where
    F: FnOnce(),
{
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
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
