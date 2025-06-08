//! Internationalization manager

use crate::bundle::BundleManager;
use crate::context::TranslationContext;
use crate::error::{I18nError, I18nResult};
use crate::resource::ResourceManager;
use crate::Locale;
use fluent::FluentArgs;
use std::path::Path;
use tracing::{debug, info, warn};

/// Manages internationalization for the application
#[derive(Debug)]
pub struct I18nManager {
    /// Default locale to fall back to
    default_locale: Locale,
    /// Resource manager for loading Fluent files
    resource_manager: ResourceManager,
    /// Bundle manager for handling FluentBundles
    bundle_manager: BundleManager,
}

impl I18nManager {
    /// Create a new I18n manager
    pub fn new<P: AsRef<Path>>(default_locale: Locale, locales_dir: P) -> I18nResult<Self> {
        let mut manager = Self {
            default_locale: default_locale.clone(),
            resource_manager: ResourceManager::new(locales_dir),
            bundle_manager: BundleManager::new(),
        };

        // Load the default locale immediately
        manager.load_locale(&default_locale)?;
        info!("I18nManager initialized with default locale: {:?}", default_locale);

        Ok(manager)
    }

    /// Load a locale's resources
    pub fn load_locale(&mut self, locale: &Locale) -> I18nResult<()> {
        debug!("Loading locale: {:?}", locale);

        let resource = self.resource_manager.load_resource(locale)?;
        self.bundle_manager.add_resource(locale, resource)?;

        info!("Successfully loaded locale: {:?}", locale);
        Ok(())
    }

    /// Get a localized message
    pub fn get_message(
        &self,
        key: &str,
        locale: &Locale,
        args: Option<&FluentArgs>,
    ) -> I18nResult<String> {
        // Try the requested locale first
        if self.bundle_manager.has_message(locale, key) {
            return self.bundle_manager.format_message(locale, key, args);
        }

        // Fall back to default locale if different
        if locale != &self.default_locale && self.bundle_manager.has_message(&self.default_locale, key) {
            warn!(
                "Message '{}' not found in locale {:?}, falling back to default locale {:?}",
                key, locale, self.default_locale
            );
            return self.bundle_manager.format_message(&self.default_locale, key, args);
        }

        // If still not found, return an error
        Err(I18nError::MessageNotFound {
            key: key.to_string(),
        })
    }

    /// Get a localized message with fallback to a default message
    pub fn get_message_or_default(
        &self,
        key: &str,
        locale: &Locale,
        args: Option<&FluentArgs>,
        default: &str,
    ) -> String {
        self.get_message(key, locale, args)
            .unwrap_or_else(|_| {
                warn!("Message '{}' not found, using default: '{}'", key, default);
                default.to_string()
            })
    }

    /// Check if a message exists for the given locale
    pub fn has_message(&self, key: &str, locale: &Locale) -> bool {
        self.bundle_manager.has_message(locale, key)
            || (locale != &self.default_locale && self.bundle_manager.has_message(&self.default_locale, key))
    }

    /// Get the default locale
    pub fn default_locale(&self) -> &Locale {
        &self.default_locale
    }

    /// Get all loaded locales
    pub fn loaded_locales(&self) -> Vec<&Locale> {
        self.bundle_manager.available_locales()
    }

    /// Reload a specific locale
    pub fn reload_locale(&mut self, locale: &Locale) -> I18nResult<()> {
        debug!("Reloading locale: {:?}", locale);

        self.bundle_manager.remove_bundle(locale);
        self.resource_manager.reload_resource(locale)?;
        self.load_locale(locale)?;

        info!("Successfully reloaded locale: {:?}", locale);
        Ok(())
    }

    /// Load all supported locales
    pub fn load_all_locales(&mut self) -> I18nResult<()> {
        for locale in Locale::all() {
            if let Err(e) = self.load_locale(&locale) {
                warn!("Failed to load locale {:?}: {}", locale, e);
            }
        }
        Ok(())
    }

    /// Get a localized message with context-aware translation
    pub fn get_message_with_context(
        &self,
        key: &str,
        locale: &Locale,
        context: &TranslationContext,
    ) -> I18nResult<String> {
        // Try gender-specific message first if applicable
        if let Some(gender_suffix) = context.get_gender_suffix(locale) {
            let gender_key = format!("{}-{}", key, gender_suffix);
            if self.has_message(&gender_key, locale) {
                let args = context.to_fluent_args();
                return self.get_message(&gender_key, locale, Some(&args));
            }
        }

        // Fall back to regular message with context
        let args = context.to_fluent_args();
        self.get_message(key, locale, Some(&args))
    }

    /// Get a localized message with context and fallback
    pub fn get_message_with_context_or_default(
        &self,
        key: &str,
        locale: &Locale,
        context: &TranslationContext,
        default: &str,
    ) -> String {
        self.get_message_with_context(key, locale, context)
            .unwrap_or_else(|_| {
                warn!("Message '{}' not found with context, using default: '{}'", key, default);
                default.to_string()
            })
    }

    /// Format a time duration message with proper pluralization
    pub fn format_time_duration(
        &self,
        key: &str,
        locale: &Locale,
        seconds: i64,
    ) -> I18nResult<String> {
        let context = TranslationContext::for_time_duration(seconds);
        self.get_message_with_context(key, locale, &context)
    }

    /// Format user statistics with context
    pub fn format_user_stats(
        &self,
        key: &str,
        locale: &Locale,
        count: i64,
        total: i64,
    ) -> I18nResult<String> {
        let context = TranslationContext::for_user_stats(count, total);
        self.get_message_with_context(key, locale, &context)
    }

    /// Format command statistics with context
    pub fn format_command_stats(
        &self,
        key: &str,
        locale: &Locale,
        successful: i64,
        failed: i64,
    ) -> I18nResult<String> {
        let context = TranslationContext::for_command_stats(successful, failed);
        self.get_message_with_context(key, locale, &context)
    }

    /// Format a pluralized message
    pub fn format_plural(
        &self,
        key: &str,
        locale: &Locale,
        count: i64,
    ) -> I18nResult<String> {
        let context = TranslationContext::with_count(count);
        self.get_message_with_context(key, locale, &context)
    }
}