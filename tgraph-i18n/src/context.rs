//! Context-aware translation support
//!
//! This module provides structures and utilities for context-aware translations,
//! including pluralization, gender agreement, and other language-specific features.

use crate::Locale;
use fluent::{FluentArgs, FluentValue};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Gender for context-aware translations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Gender {
    /// Masculine gender
    Masculine,
    /// Feminine gender
    Feminine,
    /// Neuter gender
    Neuter,
    /// Unknown or not applicable
    Unknown,
}

impl Default for Gender {
    fn default() -> Self {
        Self::Unknown
    }
}

impl Gender {
    /// Convert to string representation for Fluent
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Masculine => "masculine",
            Self::Feminine => "feminine",
            Self::Neuter => "neuter",
            Self::Unknown => "unknown",
        }
    }

    /// Parse from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "masculine" | "m" => Some(Self::Masculine),
            "feminine" | "f" => Some(Self::Feminine),
            "neuter" | "n" => Some(Self::Neuter),
            "unknown" | "u" | "" => Some(Self::Unknown),
            _ => None,
        }
    }
}

/// Context information for translations
#[derive(Debug, Clone, Default)]
pub struct TranslationContext {
    /// Count for pluralization
    pub count: Option<i64>,
    /// Gender for gender-aware translations
    pub gender: Gender,
    /// Additional custom parameters
    pub params: HashMap<String, FluentValue<'static>>,
    /// Locale-specific context
    pub locale_context: HashMap<String, String>,
}

impl TranslationContext {
    /// Create a new empty context
    pub fn new() -> Self {
        Self::default()
    }

    /// Create context with a count for pluralization
    pub fn with_count(count: i64) -> Self {
        Self {
            count: Some(count),
            ..Default::default()
        }
    }

    /// Create context with gender
    pub fn with_gender(gender: Gender) -> Self {
        Self {
            gender,
            ..Default::default()
        }
    }

    /// Create context with both count and gender
    pub fn with_count_and_gender(count: i64, gender: Gender) -> Self {
        Self {
            count: Some(count),
            gender,
            ..Default::default()
        }
    }

    /// Set the count
    pub fn set_count(mut self, count: i64) -> Self {
        self.count = Some(count);
        self
    }

    /// Set the gender
    pub fn set_gender(mut self, gender: Gender) -> Self {
        self.gender = gender;
        self
    }

    /// Add a custom parameter
    pub fn add_param<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<FluentValue<'static>>,
    {
        self.params.insert(key.into(), value.into());
        self
    }

    /// Add locale-specific context
    pub fn add_locale_context<K, V>(mut self, key: K, value: V) -> Self
    where
        K: Into<String>,
        V: Into<String>,
    {
        self.locale_context.insert(key.into(), value.into());
        self
    }

    /// Convert to FluentArgs for message formatting
    pub fn to_fluent_args(&self) -> FluentArgs {
        let mut args = FluentArgs::new();

        // Add count if present
        if let Some(count) = self.count {
            args.set("count", count);
        }

        // Add gender
        args.set("gender", self.gender.as_str());

        // Add custom parameters
        for (key, value) in &self.params {
            args.set(key, value.clone());
        }

        args
    }

    /// Get the appropriate pluralization category for the given locale
    pub fn get_plural_category(&self, locale: &Locale) -> &'static str {
        let count = self.count.unwrap_or(0);
        
        match locale {
            Locale::English => {
                if count == 1 { "one" } else { "other" }
            }
            Locale::Spanish | Locale::French => {
                if count == 0 || count == 1 { "one" } else { "other" }
            }
            Locale::German => {
                if count == 1 { "one" } else { "other" }
            }
        }
    }

    /// Check if this context requires gender-aware translation
    pub fn needs_gender_agreement(&self) -> bool {
        self.gender != Gender::Unknown
    }

    /// Get gender-specific message key suffix
    pub fn get_gender_suffix(&self, locale: &Locale) -> Option<&'static str> {
        if !self.needs_gender_agreement() {
            return None;
        }

        match locale {
            // Languages with grammatical gender
            Locale::Spanish | Locale::French | Locale::German => {
                Some(self.gender.as_str())
            }
            // Languages without grammatical gender
            Locale::English => None,
        }
    }
}

/// Helper functions for common translation patterns
impl TranslationContext {
    /// Create context for time duration formatting
    pub fn for_time_duration(seconds: i64) -> Self {
        Self::with_count(seconds)
            .add_param("seconds", seconds)
            .add_param("minutes", seconds / 60)
            .add_param("hours", seconds / 3600)
            .add_param("days", seconds / 86400)
    }

    /// Create context for user statistics
    pub fn for_user_stats(count: i64, total: i64) -> Self {
        let percentage = if total > 0 {
            (count as f64 / total as f64 * 100.0).round() as i64
        } else {
            0
        };

        Self::with_count(count)
            .add_param("total", total)
            .add_param("percentage", percentage)
    }

    /// Create context for command usage statistics
    pub fn for_command_stats(successful: i64, failed: i64) -> Self {
        let total = successful + failed;
        let success_rate = if total > 0 {
            (successful as f64 / total as f64 * 100.0).round() as i64
        } else {
            0
        };

        Self::with_count(total)
            .add_param("successful", successful)
            .add_param("failed", failed)
            .add_param("success_rate", success_rate)
    }

    /// Create context for server/user counts
    pub fn for_counts(servers: i64, users: i64) -> Self {
        Self::new()
            .add_param("servers", servers)
            .add_param("users", users)
    }
}

/// Macro to create TranslationContext more easily
#[macro_export]
macro_rules! translation_context {
    () => {
        $crate::TranslationContext::new()
    };
    (count: $count:expr) => {
        $crate::TranslationContext::with_count($count)
    };
    (gender: $gender:expr) => {
        $crate::TranslationContext::with_gender($gender)
    };
    (count: $count:expr, gender: $gender:expr) => {
        $crate::TranslationContext::with_count_and_gender($count, $gender)
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut context = $crate::TranslationContext::new();
        $(
            context = context.add_param($key, $value);
        )+
        context
    }};
}
