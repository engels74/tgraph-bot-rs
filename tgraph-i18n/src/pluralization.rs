//! Pluralization helpers for different languages
//!
//! This module provides specialized handlers for pluralization rules across
//! different languages, making it easier to handle complex pluralization scenarios.

use crate::{Locale, TranslationContext};

/// Helper for handling pluralization rules across different languages
#[derive(Debug)]
pub struct PluralizationHelper;

impl PluralizationHelper {
    /// Get the appropriate plural form for a count in the given locale
    pub fn get_plural_form(locale: &Locale, count: i64) -> &'static str {
        match locale {
            Locale::English => Self::english_plural_form(count),
            Locale::Spanish => Self::spanish_plural_form(count),
            Locale::French => Self::french_plural_form(count),
            Locale::German => Self::german_plural_form(count),
        }
    }

    /// English pluralization rules (simple: 1 = singular, everything else = plural)
    fn english_plural_form(count: i64) -> &'static str {
        if count == 1 { "one" } else { "other" }
    }

    /// Spanish pluralization rules (0 and 1 = singular, everything else = plural)
    fn spanish_plural_form(count: i64) -> &'static str {
        if count == 0 || count == 1 { "one" } else { "other" }
    }

    /// French pluralization rules (0 and 1 = singular, everything else = plural)
    fn french_plural_form(count: i64) -> &'static str {
        if count == 0 || count == 1 { "one" } else { "other" }
    }

    /// German pluralization rules (1 = singular, everything else = plural)
    fn german_plural_form(count: i64) -> &'static str {
        if count == 1 { "one" } else { "other" }
    }

    /// Create a context for time units with proper pluralization
    pub fn time_context(locale: &Locale, count: i64, unit: TimeUnit) -> TranslationContext {
        let plural_form = Self::get_plural_form(locale, count);
        
        TranslationContext::with_count(count)
            .add_param("unit", unit.as_str())
            .add_param("plural", plural_form)
    }

    /// Create a context for count-based messages
    pub fn count_context(locale: &Locale, count: i64) -> TranslationContext {
        let plural_form = Self::get_plural_form(locale, count);
        
        TranslationContext::with_count(count)
            .add_param("plural", plural_form)
    }

    /// Create a context for percentage-based messages
    pub fn percentage_context(locale: &Locale, count: i64, total: i64) -> TranslationContext {
        let percentage = if total > 0 {
            (count as f64 / total as f64 * 100.0).round() as i64
        } else {
            0
        };
        
        let plural_form = Self::get_plural_form(locale, count);
        
        TranslationContext::with_count(count)
            .add_param("total", total)
            .add_param("percentage", percentage)
            .add_param("plural", plural_form)
    }

    /// Check if a locale uses complex pluralization rules
    pub fn has_complex_pluralization(locale: &Locale) -> bool {
        match locale {
            Locale::English | Locale::German => false, // Simple 1/other rules
            Locale::Spanish | Locale::French => true,  // 0,1/other rules
        }
    }

    /// Get all plural categories for a locale
    pub fn get_plural_categories(locale: &Locale) -> Vec<&'static str> {
        match locale {
            Locale::English | Locale::Spanish | Locale::French | Locale::German => {
                vec!["one", "other"]
            }
        }
    }
}

/// Time units for pluralization
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Year,
}

impl TimeUnit {
    /// Get the string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Year => "year",
        }
    }

    /// Get the message key for this time unit
    pub fn message_key(&self) -> &'static str {
        match self {
            Self::Second => "time-seconds",
            Self::Minute => "time-minutes",
            Self::Hour => "time-hours",
            Self::Day => "time-days",
            Self::Week => "time-weeks",
            Self::Month => "time-months",
            Self::Year => "time-years",
        }
    }
}

/// Helper functions for common pluralization scenarios
impl PluralizationHelper {
    /// Format a time duration with automatic unit selection
    pub fn format_duration_context(seconds: i64) -> (TimeUnit, i64, TranslationContext) {
        let (unit, count) = if seconds < 60 {
            (TimeUnit::Second, seconds)
        } else if seconds < 3600 {
            (TimeUnit::Minute, seconds / 60)
        } else if seconds < 86400 {
            (TimeUnit::Hour, seconds / 3600)
        } else {
            (TimeUnit::Day, seconds / 86400)
        };

        let context = TranslationContext::with_count(count)
            .add_param("seconds", seconds)
            .add_param("minutes", seconds / 60)
            .add_param("hours", seconds / 3600)
            .add_param("days", seconds / 86400);

        (unit, count, context)
    }

    /// Create context for server/user counts
    pub fn server_user_context(servers: i64, users: i64) -> TranslationContext {
        TranslationContext::new()
            .add_param("servers", servers)
            .add_param("users", users)
            .add_param("server_plural", if servers == 1 { "one" } else { "other" })
            .add_param("user_plural", if users == 1 { "one" } else { "other" })
    }

    /// Create context for command statistics
    pub fn command_stats_context(total: i64, successful: i64, failed: i64) -> TranslationContext {
        let success_rate = if total > 0 {
            (successful as f64 / total as f64 * 100.0).round() as i64
        } else {
            0
        };

        TranslationContext::with_count(total)
            .add_param("successful", successful)
            .add_param("failed", failed)
            .add_param("success_rate", success_rate)
            .add_param("total_plural", if total == 1 { "one" } else { "other" })
            .add_param("success_plural", if successful == 1 { "one" } else { "other" })
            .add_param("failed_plural", if failed == 1 { "one" } else { "other" })
    }
}

/// Macro for creating pluralized contexts easily
#[macro_export]
macro_rules! plural_context {
    ($locale:expr, $count:expr) => {
        $crate::PluralizationHelper::count_context($locale, $count)
    };
    ($locale:expr, $count:expr, $total:expr) => {
        $crate::PluralizationHelper::percentage_context($locale, $count, $total)
    };
    (time: $locale:expr, $count:expr, $unit:expr) => {
        $crate::PluralizationHelper::time_context($locale, $count, $unit)
    };
    (duration: $seconds:expr) => {{
        let (unit, count, context) = $crate::PluralizationHelper::format_duration_context($seconds);
        (unit, count, context)
    }};
}
