//! Error types for internationalization operations

use thiserror::Error;

/// Errors that can occur during internationalization operations
#[derive(Error, Debug)]
pub enum I18nError {
    /// Failed to parse a language identifier
    #[error("Invalid language identifier: {0}")]
    InvalidLanguageId(String),

    /// Failed to load a resource file
    #[error("Failed to load resource file: {path}")]
    ResourceLoadError { path: String },

    /// Failed to parse a Fluent resource
    #[error("Failed to parse Fluent resource: {errors:?}")]
    FluentParseError { errors: Vec<String> },

    /// Message not found in any bundle
    #[error("Message not found: {key}")]
    MessageNotFound { key: String },

    /// Failed to format a message
    #[error("Failed to format message '{key}': {errors:?}")]
    MessageFormatError { key: String, errors: Vec<String> },

    /// Bundle creation failed
    #[error("Failed to create bundle for locale {locale}: {source}")]
    BundleCreationError {
        locale: String,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    /// IO error occurred
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for i18n operations
pub type I18nResult<T> = Result<T, I18nError>;
