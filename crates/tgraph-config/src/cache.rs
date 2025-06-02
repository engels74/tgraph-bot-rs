//! Thread-safe configuration caching with arc-swap for lock-free reads.

use crate::schema::Config;
use arc_swap::ArcSwap;
use std::sync::Arc;

/// Thread-safe configuration cache using arc-swap for lock-free reads.
pub struct ConfigCache {
    config: ArcSwap<Config>,
}

impl ConfigCache {
    /// Creates a new configuration cache with the given initial configuration.
    pub fn new(config: Config) -> Self {
        Self {
            config: ArcSwap::from_pointee(config),
        }
    }

    /// Gets the current configuration.
    pub fn get(&self) -> Arc<Config> {
        self.config.load_full()
    }

    /// Updates the configuration atomically.
    pub fn update(&self, config: Config) {
        self.config.store(Arc::new(config));
    }
}

impl Default for ConfigCache {
    fn default() -> Self {
        Self::new(Config::default())
    }
}
