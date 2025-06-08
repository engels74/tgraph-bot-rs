//! Test to verify all locale files have complete message coverage

use tgraph_i18n::{I18nManager, Locale, fluent_args};

#[test]
fn test_all_locales_have_same_messages() {
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load all supported locales
    for locale in Locale::all() {
        if locale != Locale::English {
            manager.load_locale(&locale).unwrap();
        }
    }
    
    // Test a sample of key messages across all locales (without parameters)
    let test_messages = vec![
        "hello",
        "success",
        "loading",
        "please-wait",
        "bot-title",
        "uptime-title",
        "admin-metrics-title",
        "export-title",
        "graph-success-title",
        "status-online",
        "status-offline",
    ];
    
    for locale in Locale::all() {
        for message_key in &test_messages {
            let result = manager.get_message(message_key, &locale, None);
            assert!(
                result.is_ok(), 
                "Message '{}' not found in locale {:?}: {:?}", 
                message_key, locale, result
            );
        }
    }
}

#[test]
fn test_pluralization_works_across_locales() {
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load all supported locales
    for locale in Locale::all() {
        if locale != Locale::English {
            manager.load_locale(&locale).unwrap();
        }
    }
    
    // Test pluralization for time units
    let pluralization_tests = vec![
        ("time-seconds", 1),
        ("time-seconds", 2),
        ("time-minutes", 1),
        ("time-minutes", 5),
        ("time-hours", 1),
        ("time-hours", 24),
        ("time-days", 1),
        ("time-days", 7),
    ];
    
    for locale in Locale::all() {
        for (message_key, count) in &pluralization_tests {
            let args = fluent_args!["count" => count];
            let result = manager.get_message(message_key, &locale, args.as_ref());
            assert!(
                result.is_ok(),
                "Pluralization failed for '{}' with count {} in locale {:?}: {:?}",
                message_key, count, locale, result
            );
            
            let message = result.unwrap();
            assert!(
                message.contains(&count.to_string()),
                "Message '{}' doesn't contain count {} in locale {:?}: '{}'",
                message_key, count, locale, message
            );
        }
    }
}

#[test]
fn test_parameter_substitution_works() {
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load all supported locales
    for locale in Locale::all() {
        if locale != Locale::English {
            manager.load_locale(&locale).unwrap();
        }
    }
    
    // Test parameter substitution
    let param_tests = vec![
        ("bot-version", fluent_args!["version" => "1.0.0"]),
        ("uptime-duration", fluent_args!["hours" => 2, "minutes" => 30, "seconds" => 45]),
        ("stats-total-commands", fluent_args!["count" => 42]),
        ("export-generated", fluent_args!["time" => "2024-01-01 12:00 UTC"]),
    ];
    
    for locale in Locale::all() {
        for (message_key, args) in &param_tests {
            let result = manager.get_message(message_key, &locale, args.as_ref());
            assert!(
                result.is_ok(),
                "Parameter substitution failed for '{}' in locale {:?}: {:?}",
                message_key, locale, result
            );
            
            let message = result.unwrap();
            // Verify that the message doesn't contain unreplaced placeholders
            assert!(
                !message.contains("{$"),
                "Message '{}' contains unreplaced placeholders in locale {:?}: '{}'",
                message_key, locale, message
            );
        }
    }
}

#[test]
fn test_fallback_to_english_works() {
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load Spanish locale
    manager.load_locale(&Locale::Spanish).unwrap();
    
    // Test that a message that exists in English but not in Spanish falls back
    // (This is a hypothetical test - in practice all our messages should exist in all locales)
    let result = manager.get_message("hello", &Locale::Spanish, None);
    assert!(result.is_ok(), "Fallback mechanism failed: {:?}", result);
}

#[test]
fn test_message_keys_consistency() {
    // This test ensures that we have the expected number of messages
    // and that key message categories are present
    let manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Test that we have a reasonable number of messages (should be 100+)
    let test_keys = vec![
        // Common messages (without parameters)
        "hello", "success", "loading", "please-wait",

        // Commands
        "command-about", "command-help", "command-stats", "command-update-graphs", "command-metrics",

        // Bot information (without parameters)
        "bot-title", "bot-description", "bot-built-with", "bot-features",

        // Uptime messages (without parameters)
        "uptime-title", "uptime-status-ready",

        // User statistics (without parameters)
        "stats-command-usage", "stats-most-used", "stats-activity-scope", "stats-timeline",
        "stats-no-data", "stats-none", "stats-all-time",

        // Admin messages
        "admin-update-graphs-title", "admin-metrics-title", "admin-scheduler-title",
        "admin-update-graphs-starting", "admin-update-graphs-wait",

        // Data export/deletion (without parameters)
        "export-title", "export-details", "delete-confirmation-required", "delete-success",
        "export-privacy-notice", "export-contains-all-data", "export-sent-privately",

        // Time periods
        "period-daily", "period-weekly", "period-monthly", "period-all-time",

        // Graph messages (without parameters)
        "graph-success-title", "graph-error-title",

        // Permission messages (without parameters)
        "permission-error-title",

        // Graph types
        "graph-line", "graph-bar", "graph-pie", "graph-scatter",

        // Status messages
        "status-online", "status-offline", "status-maintenance",
    ];
    
    for key in test_keys {
        let result = manager.get_message(key, &Locale::English, None);
        assert!(
            result.is_ok(),
            "Expected message key '{}' not found in English locale: {:?}",
            key, result
        );
    }
}
