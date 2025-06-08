//! Resource management for Fluent files

use crate::error::{I18nError, I18nResult};
use crate::Locale;
use fluent::FluentResource;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use tracing::{debug, error, info, warn};

/// Manages loading of Fluent resources
#[derive(Debug)]
pub struct ResourceManager {
    /// Base directory for locale resources
    base_dir: PathBuf,
    /// Track which locales have been successfully loaded
    loaded_locales: HashMap<Locale, bool>,
}

impl ResourceManager {
    /// Create a new ResourceManager
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            loaded_locales: HashMap::new(),
        }
    }

    /// Load a resource for the given locale
    pub fn load_resource(&mut self, locale: &Locale) -> I18nResult<FluentResource> {
        let resource = self.load_resource_from_file(locale)?;
        self.loaded_locales.insert(locale.clone(), true);
        Ok(resource)
    }

    /// Load a resource from file
    fn load_resource_from_file(&self, locale: &Locale) -> I18nResult<FluentResource> {
        let resource_path = self.base_dir.join(locale.resource_file());
        
        debug!("Loading resource file: {:?}", resource_path);
        
        if !resource_path.exists() {
            warn!("Resource file does not exist: {:?}", resource_path);
            return Err(I18nError::ResourceLoadError {
                path: resource_path.to_string_lossy().to_string(),
            });
        }

        let content = fs::read_to_string(&resource_path)
            .map_err(|_| I18nError::ResourceLoadError {
                path: resource_path.to_string_lossy().to_string(),
            })?;

        let resource = FluentResource::try_new(content)
            .map_err(|(_, errors)| {
                let error_messages: Vec<String> = errors
                    .into_iter()
                    .map(|e| format!("{:?}", e))
                    .collect();
                
                error!("Failed to parse Fluent resource: {:?}", error_messages);
                
                I18nError::FluentParseError {
                    errors: error_messages,
                }
            })?;

        info!("Successfully loaded resource for locale: {:?}", locale);
        Ok(resource)
    }

    /// Get all loaded locales
    pub fn loaded_locales(&self) -> Vec<&Locale> {
        self.loaded_locales.keys().collect()
    }

    /// Clear all cached resources
    pub fn clear_cache(&mut self) {
        self.loaded_locales.clear();
        info!("Cleared resource cache");
    }

    /// Reload a specific locale's resource
    pub fn reload_resource(&mut self, locale: &Locale) -> I18nResult<()> {
        self.loaded_locales.remove(locale);
        self.load_resource(locale)?;
        info!("Reloaded resource for locale: {:?}", locale);
        Ok(())
    }

    /// Check if a resource is loaded for the given locale
    pub fn is_loaded(&self, locale: &Locale) -> bool {
        self.loaded_locales.contains_key(locale)
    }

    /// Get the base directory for resources
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new("locales")
    }
}
