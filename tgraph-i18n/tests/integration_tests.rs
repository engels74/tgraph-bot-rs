//! Integration tests for the i18n system

use tgraph_i18n::{fluent_args, I18nManager, Locale};
use std::fs;
use tempfile::TempDir;

/// Create a temporary directory with test locale files
fn create_test_locales() -> TempDir {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    
    // Create locale directories
    fs::create_dir_all(temp_dir.path().join("en")).unwrap();
    fs::create_dir_all(temp_dir.path().join("es")).unwrap();
    
    // Create English locale file
    fs::write(
        temp_dir.path().join("en/main.ftl"),
        r#"
hello = Hello!
welcome = Welcome, {$name}!
item-count = You have {$count ->
    [one] {$count} item
   *[other] {$count} items
}
"#,
    ).unwrap();
    
    // Create Spanish locale file
    fs::write(
        temp_dir.path().join("es/main.ftl"),
        r#"
hello = ¡Hola!
welcome = ¡Bienvenido, {$name}!
item-count = Tienes {$count ->
    [one] {$count} elemento
   *[other] {$count} elementos
}
"#,
    ).unwrap();
    
    temp_dir
}

#[test]
fn test_i18n_manager_creation() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    assert_eq!(manager.default_locale(), &Locale::English);
    assert!(manager.loaded_locales().contains(&&Locale::English));
}

#[test]
fn test_basic_message_retrieval() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    let message = manager.get_message("hello", &Locale::English, None).unwrap();
    assert_eq!(message, "Hello!");
}

#[test]
fn test_message_with_arguments() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    let args = fluent_args!["name" => "Alice"];
    let message = manager.get_message("welcome", &Locale::English, args.as_ref()).unwrap();
    assert_eq!(message, "Welcome, Alice!");
}

#[test]
fn test_pluralization() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    // Test singular
    let args = fluent_args!["count" => 1];
    let message = manager.get_message("item-count", &Locale::English, args.as_ref()).unwrap();
    assert_eq!(message, "You have 1 item");
    
    // Test plural
    let args = fluent_args!["count" => 5];
    let message = manager.get_message("item-count", &Locale::English, args.as_ref()).unwrap();
    assert_eq!(message, "You have 5 items");
}

#[test]
fn test_locale_loading() {
    let temp_dir = create_test_locales();
    let mut manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    // Load Spanish locale
    manager.load_locale(&Locale::Spanish).unwrap();
    
    let message = manager.get_message("hello", &Locale::Spanish, None).unwrap();
    assert_eq!(message, "¡Hola!");
}

#[test]
fn test_fallback_to_default_locale() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();

    // Try to get a message in a locale that doesn't exist, should fall back to English
    let message = manager.get_message("hello", &Locale::French, None).unwrap();

    // This should succeed by falling back to English
    assert_eq!(message, "Hello!");
}

#[test]
fn test_message_not_found() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    let result = manager.get_message("nonexistent", &Locale::English, None);
    assert!(result.is_err());
}

#[test]
fn test_get_message_or_default() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    // Existing message
    let message = manager.get_message_or_default("hello", &Locale::English, None, "Default");
    assert_eq!(message, "Hello!");
    
    // Non-existing message
    let message = manager.get_message_or_default("nonexistent", &Locale::English, None, "Default");
    assert_eq!(message, "Default");
}

#[test]
fn test_has_message() {
    let temp_dir = create_test_locales();
    let manager = I18nManager::new(Locale::English, temp_dir.path()).unwrap();
    
    assert!(manager.has_message("hello", &Locale::English));
    assert!(!manager.has_message("nonexistent", &Locale::English));
}

#[test]
fn test_locale_enum_methods() {
    assert_eq!(Locale::English.code(), "en-US");
    assert_eq!(Locale::Spanish.code(), "es-ES");
    assert_eq!(Locale::French.code(), "fr-FR");
    assert_eq!(Locale::German.code(), "de-DE");
    
    assert_eq!(Locale::English.short_code(), "en");
    assert_eq!(Locale::Spanish.short_code(), "es");
    
    assert_eq!(Locale::from_code("en"), Some(Locale::English));
    assert_eq!(Locale::from_code("es-ES"), Some(Locale::Spanish));
    assert_eq!(Locale::from_code("invalid"), None);
    
    assert_eq!(Locale::English.display_name(), "English");
    assert_eq!(Locale::Spanish.display_name(), "Español");
    
    assert_eq!(Locale::all().len(), 4);
}

#[test]
fn test_language_identifier_conversion() {
    let locale = Locale::English;
    let lang_id = locale.to_language_identifier().unwrap();
    assert_eq!(lang_id.to_string(), "en-US");
}
