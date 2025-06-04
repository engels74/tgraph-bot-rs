//! Locale management and utilities

use serde::{Deserialize, Serialize};

/// Supported locales
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum Locale {
    English,
    Spanish,
    French,
    German,
}

impl Default for Locale {
    fn default() -> Self {
        Self::English
    }
}

impl Locale {
    /// Get the language code for this locale
    pub fn code(&self) -> &'static str {
        match self {
            Self::English => "en",
            Self::Spanish => "es",
            Self::French => "fr",
            Self::German => "de",
        }
    }

    /// Parse a locale from a language code
    pub fn from_code(code: &str) -> Option<Self> {
        match code {
            "en" => Some(Self::English),
            "es" => Some(Self::Spanish),
            "fr" => Some(Self::French),
            "de" => Some(Self::German),
            _ => None,
        }
    }
} 