//! Locale management and utilities

use crate::error::{I18nError, I18nResult};
use serde::{Deserialize, Serialize};
use unic_langid::LanguageIdentifier;

/// Supported locales
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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
            Self::English => "en-US",
            Self::Spanish => "es-ES",
            Self::French => "fr-FR",
            Self::German => "de-DE",
        }
    }

    /// Get the short language code for this locale
    pub fn short_code(&self) -> &'static str {
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
            "en" | "en-US" => Some(Self::English),
            "es" | "es-ES" => Some(Self::Spanish),
            "fr" | "fr-FR" => Some(Self::French),
            "de" | "de-DE" => Some(Self::German),
            _ => None,
        }
    }

    /// Convert to Fluent LanguageIdentifier
    pub fn to_language_identifier(&self) -> I18nResult<LanguageIdentifier> {
        self.code()
            .parse()
            .map_err(|_| I18nError::InvalidLanguageId(self.code().to_string()))
    }

    /// Get all supported locales
    pub fn all() -> Vec<Self> {
        vec![Self::English, Self::Spanish, Self::French, Self::German]
    }

    /// Get the display name for this locale
    pub fn display_name(&self) -> &'static str {
        match self {
            Self::English => "English",
            Self::Spanish => "Español",
            Self::French => "Français",
            Self::German => "Deutsch",
        }
    }

    /// Get the resource file name for this locale
    pub fn resource_file(&self) -> String {
        format!("{}/main.ftl", self.short_code())
    }
}