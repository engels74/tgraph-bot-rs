//! Integration tests for tgraph-config crate.

use tgraph_config::{Config, ConfigCache};

#[test]
fn test_default_config_validation() {
    let mut config = Config::default();

    // Default config should fail validation due to empty tokens
    assert!(config.validate().is_err());

    // Set required fields
    config.discord.token = "test_token".to_string();
    config.tautulli.api_key = "test_api_key".to_string();

    // Now it should pass
    assert!(config.validate().is_ok());
}

#[test]
fn test_config_cache() {
    let config = Config::default();
    let cache = ConfigCache::new(config.clone());

    // Should be able to get the config
    let cached_config = cache.get();
    assert_eq!(cached_config.data.language, config.data.language);

    // Should be able to update the config
    let mut new_config = config;
    new_config.data.language = "fr-FR".to_string();
    cache.update(new_config.clone());

    let updated_config = cache.get();
    assert_eq!(updated_config.data.language, "fr-FR");
}
