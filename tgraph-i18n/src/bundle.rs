//! FluentBundle management and message formatting

use crate::error::{I18nError, I18nResult};
use crate::Locale;
use fluent::{FluentArgs, FluentBundle, FluentResource, FluentValue};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use tracing::{debug, error, warn};

/// Thread-safe wrapper around FluentBundle
pub struct ThreadSafeBundle {
    bundle: FluentBundle<FluentResource>,
}

impl std::fmt::Debug for ThreadSafeBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ThreadSafeBundle")
            .field("bundle", &"FluentBundle<FluentResource>")
            .finish()
    }
}

impl ThreadSafeBundle {
    pub fn new(bundle: FluentBundle<FluentResource>) -> Self {
        Self { bundle }
    }

    pub fn get_message(&self, id: &str) -> Option<fluent::FluentMessage> {
        self.bundle.get_message(id)
    }

    pub fn has_message(&self, id: &str) -> bool {
        self.bundle.has_message(id)
    }



    pub fn add_resource(&mut self, resource: FluentResource) -> Result<(), Vec<fluent::FluentError>> {
        self.bundle.add_resource(resource)
    }
}

// SAFETY: FluentBundle is effectively read-only after initialization for our use case
unsafe impl Send for ThreadSafeBundle {}
unsafe impl Sync for ThreadSafeBundle {}

/// Manages FluentBundle instances for different locales
#[derive(Debug)]
pub struct BundleManager {
    /// Bundles by locale, wrapped in Arc<RwLock<>> for thread safety
    bundles: HashMap<Locale, Arc<RwLock<ThreadSafeBundle>>>,
}

impl BundleManager {
    /// Create a new BundleManager
    pub fn new() -> Self {
        Self {
            bundles: HashMap::new(),
        }
    }

    /// Add a resource to a locale's bundle
    pub fn add_resource(&mut self, locale: &Locale, resource: FluentResource) -> I18nResult<()> {
        let lang_id = locale.to_language_identifier()?;
        
        let bundle = self.bundles.entry(locale.clone()).or_insert_with(|| {
            let mut bundle = FluentBundle::new(vec![lang_id]);
            // Set use_isolating to false for simpler output
            bundle.set_use_isolating(false);
            Arc::new(RwLock::new(ThreadSafeBundle::new(bundle)))
        });

        bundle.write().unwrap().add_resource(resource)
            .map_err(|errors| {
                let error_messages: Vec<String> = errors
                    .into_iter()
                    .map(|e| format!("{:?}", e))
                    .collect();

                error!("Failed to add resource to bundle: {:?}", error_messages);

                I18nError::BundleCreationError {
                    locale: locale.code().to_string(),
                    source: Box::new(I18nError::FluentParseError {
                        errors: error_messages,
                    }),
                }
            })?;

        debug!("Added resource to bundle for locale: {:?}", locale);
        Ok(())
    }

    /// Format a message with the given arguments
    pub fn format_message(
        &self,
        locale: &Locale,
        message_id: &str,
        args: Option<&FluentArgs>,
    ) -> I18nResult<String> {
        let bundle = self.bundles.get(locale)
            .ok_or_else(|| I18nError::MessageNotFound {
                key: message_id.to_string(),
            })?;

        let bundle_guard = bundle.read().unwrap();

        let message = bundle_guard.get_message(message_id)
            .ok_or_else(|| I18nError::MessageNotFound {
                key: message_id.to_string(),
            })?;

        let pattern = message.value()
            .ok_or_else(|| I18nError::MessageNotFound {
                key: message_id.to_string(),
            })?;

        let mut errors = Vec::new();
        let formatted = bundle_guard.bundle.format_pattern(pattern, args, &mut errors);

        if !errors.is_empty() {
            let error_messages: Vec<String> = errors
                .into_iter()
                .map(|e| format!("{:?}", e))
                .collect();
            
            warn!("Formatting errors for message '{}': {:?}", message_id, error_messages);
            
            return Err(I18nError::MessageFormatError {
                key: message_id.to_string(),
                errors: error_messages,
            });
        }

        Ok(formatted.into_owned())
    }

    /// Check if a message exists in the bundle
    pub fn has_message(&self, locale: &Locale, message_id: &str) -> bool {
        self.bundles
            .get(locale)
            .map(|bundle| bundle.read().unwrap().has_message(message_id))
            .unwrap_or(false)
    }

    /// Get all available locales
    pub fn available_locales(&self) -> Vec<&Locale> {
        self.bundles.keys().collect()
    }

    /// Remove a bundle for a locale
    pub fn remove_bundle(&mut self, locale: &Locale) {
        self.bundles.remove(locale);
        debug!("Removed bundle for locale: {:?}", locale);
    }

    /// Clear all bundles
    pub fn clear(&mut self) {
        self.bundles.clear();
        debug!("Cleared all bundles");
    }
}

impl Default for BundleManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Helper function to create FluentArgs from key-value pairs
pub fn fluent_args<'a>(args: &[(&'a str, FluentValue<'a>)]) -> FluentArgs<'a> {
    let mut fluent_args = FluentArgs::new();
    for (key, value) in args {
        fluent_args.set(*key, value.clone());
    }
    fluent_args
}

/// Macro to create FluentArgs more easily
#[macro_export]
macro_rules! fluent_args {
    () => {
        None
    };
    ($($key:expr => $value:expr),+ $(,)?) => {{
        let mut args = fluent::FluentArgs::new();
        $(
            args.set($key, $value);
        )+
        Some(args)
    }};
}
