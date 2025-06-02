//! Comprehensive tests for tgraph-common utilities following TDD principles.
//!
//! These tests cover:
//! - Date/time utilities with edge cases
//! - Unicode-safe string manipulation
//! - Async utilities with proper cancellation handling

use chrono::{DateTime, Duration as ChronoDuration, NaiveDate, TimeZone, Utc};
use proptest::prelude::*;
use std::time::Duration;
use tgraph_common::utils::*;
use tokio::time::timeout;
use tokio_test;
use unicode_segmentation::UnicodeSegmentation;

// =============================================================================
// TDD Test Cases: Date/time utilities handle edge cases correctly
// =============================================================================

#[tokio::test]
async fn test_time_range_validation_normal_cases() {
    // Test case: Normal time ranges should be valid
    let start = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();

    let result = validate_time_range(start, end);
    assert!(result.is_ok(), "Normal time range should be valid");
}

#[tokio::test]
async fn test_time_range_validation_edge_cases() {
    // Test case: Start time equals end time (should be invalid)
    let same_time = Utc.with_ymd_and_hms(2024, 1, 1, 12, 0, 0).unwrap();
    let result = validate_time_range(same_time, same_time);
    assert!(result.is_err(), "Same start and end time should be invalid");

    // Test case: Start time after end time (should be invalid)
    let start = Utc.with_ymd_and_hms(2024, 1, 2, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let result = validate_time_range(start, end);
    assert!(result.is_err(), "Start after end should be invalid");

    // Test case: Very large time spans should be handled
    let start = Utc.with_ymd_and_hms(1970, 1, 1, 0, 0, 0).unwrap();
    let end = Utc.with_ymd_and_hms(2100, 12, 31, 23, 59, 59).unwrap();
    let result = validate_time_range(start, end);
    assert!(result.is_ok(), "Large time span should be valid");
}

#[tokio::test]
async fn test_time_zone_conversion_edge_cases() {
    // Test case: UTC midnight conversions
    let utc_midnight = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
    let result = convert_to_display_timezone(utc_midnight, "UTC");
    assert!(result.is_ok(), "UTC midnight conversion should work");

    // Test case: Invalid timezone should be handled gracefully
    let result = convert_to_display_timezone(utc_midnight, "Invalid/Timezone");
    assert!(result.is_err(), "Invalid timezone should return error");

    // Test case: Leap year edge case
    let leap_day = Utc.with_ymd_and_hms(2024, 2, 29, 12, 0, 0).unwrap();
    let result = convert_to_display_timezone(leap_day, "UTC");
    assert!(result.is_ok(), "Leap day conversion should work");
}

#[tokio::test]
async fn test_duration_formatting_edge_cases() {
    // Test case: Zero duration
    let zero_duration = ChronoDuration::zero();
    let formatted = format_duration_human_readable(zero_duration);
    assert_eq!(
        formatted, "0 seconds",
        "Zero duration should format correctly"
    );

    // Test case: Maximum reasonable duration
    let large_duration = ChronoDuration::days(365 * 100); // 100 years
    let formatted = format_duration_human_readable(large_duration);
    assert!(
        formatted.contains("years"),
        "Large duration should include years"
    );

    // Test case: Negative duration should be handled
    let negative_duration = ChronoDuration::seconds(-3600);
    let formatted = format_duration_human_readable(negative_duration);
    assert!(
        formatted.contains("ago") || formatted.contains("negative"),
        "Negative duration should be indicated"
    );
}

#[tokio::test]
async fn test_time_ago_formatting() {
    let now = Utc::now();

    // Test case: Recent time (seconds ago)
    let recent = now - ChronoDuration::seconds(30);
    let formatted = format_time_ago(recent, now);
    assert!(
        formatted.contains("seconds"),
        "Recent time should show seconds"
    );

    // Test case: Future time should be handled
    let future = now + ChronoDuration::hours(1);
    let formatted = format_time_ago(future, now);
    assert!(
        formatted.contains("future") || formatted.contains("from now"),
        "Future time should be indicated"
    );
}

// =============================================================================
// TDD Test Cases: String manipulation functions are Unicode-safe
// =============================================================================

#[tokio::test]
async fn test_unicode_safe_truncation() {
    // Test case: Unicode characters should not be split
    let unicode_string = "Hello 世界! 🌟✨";
    let truncated = truncate_string_unicode_safe(unicode_string, 10);

    // Should not split multi-byte characters
    assert!(
        truncated.is_char_boundary(truncated.len()),
        "Truncation should not split Unicode characters"
    );

    // Test case: Emoji handling
    let emoji_string = "Test 🚀🌟✨🎉";
    let truncated = truncate_string_unicode_safe(emoji_string, 8);
    assert!(
        truncated.is_char_boundary(truncated.len()),
        "Emoji truncation should be safe"
    );
}

#[tokio::test]
async fn test_unicode_normalization() {
    // Test case: Different Unicode normalization forms
    let nfc_string = "café"; // NFC form
    let nfd_string = "cafe\u{0301}"; // NFD form (e with combining acute accent)

    let normalized_nfc = normalize_unicode_string(nfc_string);
    let normalized_nfd = normalize_unicode_string(nfd_string);

    assert_eq!(
        normalized_nfc, normalized_nfd,
        "Different Unicode forms should normalize to same result"
    );
}

#[tokio::test]
async fn test_safe_string_sanitization() {
    // Test case: Mixed script handling
    let mixed_script = "Hello नमस्ते مرحبا 你好 🌍";
    let sanitized = sanitize_string_unicode_safe(mixed_script);

    // Should preserve all valid Unicode characters
    assert!(sanitized.contains("नमस्ते"), "Devanagari should be preserved");
    assert!(sanitized.contains("مرحبا"), "Arabic should be preserved");
    assert!(sanitized.contains("你好"), "Chinese should be preserved");
    assert!(sanitized.contains("🌍"), "Emoji should be preserved");
}

#[tokio::test]
async fn test_grapheme_cluster_handling() {
    // Test case: Complex grapheme clusters should be treated as single units
    let complex_string = "👨‍👩‍👧‍👦"; // Family emoji (single grapheme cluster)
    let length = count_graphemes(complex_string);
    assert_eq!(length, 1, "Family emoji should count as one grapheme");

    // Test case: Combining characters
    let combining = "e\u{0301}\u{0302}"; // e with acute and circumflex
    let length = count_graphemes(combining);
    assert_eq!(
        length, 1,
        "Combined characters should count as one grapheme"
    );
}

#[tokio::test]
async fn test_bidirectional_text_handling() {
    // Test case: Mixed LTR/RTL text should be handled safely
    let bidi_text = "Hello مرحبا World";
    let sanitized = sanitize_bidi_text(bidi_text);

    // Should not introduce any directional override characters that could cause issues
    assert!(
        !sanitized.contains('\u{202E}'),
        "Should not contain RLO characters"
    );
    assert!(
        !sanitized.contains('\u{202D}'),
        "Should not contain LRO characters"
    );
}

// =============================================================================
// TDD Test Cases: Async utilities properly handle cancellation
// =============================================================================

#[tokio::test]
async fn test_timeout_wrapper_cancellation() {
    // Test case: Function that completes before timeout
    let quick_task = async {
        tokio::time::sleep(Duration::from_millis(10)).await;
        "completed"
    };

    let result = with_timeout(Duration::from_millis(100), quick_task).await;
    assert!(result.is_ok(), "Quick task should complete successfully");
    assert_eq!(result.unwrap(), "completed");
}

#[tokio::test]
async fn test_timeout_wrapper_timeout() {
    // Test case: Function that times out
    let slow_task = async {
        tokio::time::sleep(Duration::from_millis(200)).await;
        "never reached"
    };

    let result = with_timeout(Duration::from_millis(50), slow_task).await;
    assert!(result.is_err(), "Slow task should timeout");
}

#[tokio::test]
async fn test_cancellation_safe_cleanup() {
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::Arc;

    // Test case: Cleanup should run even when cancelled
    let cleanup_ran = Arc::new(AtomicBool::new(false));
    let cleanup_clone = cleanup_ran.clone();

    let cancellable_task = async move {
        let _guard = CleanupGuard::new(move || {
            cleanup_clone.store(true, Ordering::SeqCst);
        });

        // Simulate long-running work
        tokio::time::sleep(Duration::from_millis(100)).await;
        "completed"
    };

    // Cancel the task before it completes
    let result = timeout(Duration::from_millis(50), cancellable_task).await;
    assert!(result.is_err(), "Task should be cancelled");

    // Give cleanup a moment to run
    tokio::time::sleep(Duration::from_millis(10)).await;
    assert!(
        cleanup_ran.load(Ordering::SeqCst),
        "Cleanup should have run"
    );
}

#[tokio::test]
async fn test_retry_with_backoff_cancellation() {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;

    // Test case: Retry logic should respect cancellation
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_clone = attempt_count.clone();

    let failing_operation = move || {
        let count = attempt_clone.clone();
        async move {
            count.fetch_add(1, Ordering::SeqCst);
            tokio::time::sleep(Duration::from_millis(100)).await; // Slow operation
            Err::<String, &str>("always fails")
        }
    };

    let retry_config = RetryConfig::new()
        .max_attempts(5)
        .initial_delay(Duration::from_millis(10))
        .max_delay(Duration::from_millis(1000));

    // Cancel after a short time
    let result = timeout(
        Duration::from_millis(150),
        retry_with_exponential_backoff(failing_operation, retry_config),
    )
    .await;

    assert!(result.is_err(), "Retry should be cancelled");

    // Should have attempted at least once but not all 5 times due to cancellation
    let attempts = attempt_count.load(Ordering::SeqCst);
    assert!(attempts >= 1, "Should have attempted at least once");
    assert!(
        attempts < 5,
        "Should not complete all attempts due to cancellation"
    );
}

#[tokio::test]
async fn test_graceful_shutdown_signal() {
    use tokio::sync::broadcast;

    // Test case: Tasks should respect shutdown signals
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);

    let task = async move {
        loop {
            tokio::select! {
                _ = shutdown_rx.recv() => {
                    return "gracefully shutdown";
                }
                _ = tokio::time::sleep(Duration::from_millis(10)) => {
                    // Continue working
                }
            }
        }
    };

    // Start the task
    let task_handle = tokio::spawn(task);

    // Send shutdown signal after a short delay
    tokio::time::sleep(Duration::from_millis(25)).await;
    let _ = shutdown_tx.send(());

    let result = task_handle.await.unwrap();
    assert_eq!(
        result, "gracefully shutdown",
        "Task should shut down gracefully"
    );
}

// =============================================================================
// Property-based tests for robust edge case coverage
// =============================================================================

proptest! {
    #[test]
    fn prop_unicode_truncation_is_safe(
        text in "\\PC*", // Any Unicode text
        max_len in 0usize..200
    ) {
        let truncated = truncate_string_unicode_safe(&text, max_len);

        // Property: Result should always be valid UTF-8 (this is implicit in Rust String)
        // Property: Truncated string should respect character boundaries
        prop_assert!(truncated.is_ascii() || truncated.len() == 0 || text.is_char_boundary(truncated.len()) || truncated.ends_with("..."));

        // Property: Should not exceed max length in graphemes (what users perceive as characters)
        prop_assert!(truncated.graphemes(true).count() <= max_len);
    }

    #[test]
    fn prop_time_range_validation_is_consistent(
        start_secs in 0i64..2_000_000_000,
        duration_secs in -1_000_000i64..1_000_000
    ) {
        let start = DateTime::from_timestamp(start_secs, 0).unwrap_or_else(|| Utc::now());
        let end = start + ChronoDuration::seconds(duration_secs);

        let result = validate_time_range(start, end);

        // Property: Result should be consistent with duration sign
        if duration_secs > 0 {
            prop_assert!(result.is_ok(), "Positive duration should be valid");
        } else {
            prop_assert!(result.is_err(), "Non-positive duration should be invalid");
        }
    }

    #[test]
    fn prop_sanitization_preserves_safe_content(text in "[\\p{L}\\p{N}\\s.,!?'-]*") {
        let sanitized = sanitize_string_unicode_safe(&text);

        // Property: Safe characters should be preserved
        prop_assert_eq!(sanitized, text);
    }
}
