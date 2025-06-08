//! Tests for context-aware translation functionality

use tgraph_i18n::{
    Gender, I18nManager, Locale, PluralizationHelper, TranslationContext,
    plural_context, translation_context
};

#[test]
fn test_translation_context_creation() {
    // Test basic context creation
    let context = TranslationContext::new();
    assert_eq!(context.count, None);
    assert_eq!(context.gender, Gender::Unknown);

    // Test context with count
    let context = TranslationContext::with_count(5);
    assert_eq!(context.count, Some(5));

    // Test context with gender
    let context = TranslationContext::with_gender(Gender::Feminine);
    assert_eq!(context.gender, Gender::Feminine);

    // Test context with both
    let context = TranslationContext::with_count_and_gender(3, Gender::Masculine);
    assert_eq!(context.count, Some(3));
    assert_eq!(context.gender, Gender::Masculine);
}

#[test]
fn test_translation_context_macro() {
    // Test empty context
    let context = translation_context!();
    assert_eq!(context.count, None);

    // Test count context
    let context = translation_context!(count: 5);
    assert_eq!(context.count, Some(5));

    // Test gender context
    let context = translation_context!(gender: Gender::Feminine);
    assert_eq!(context.gender, Gender::Feminine);

    // Test count and gender
    let context = translation_context!(count: 3, gender: Gender::Masculine);
    assert_eq!(context.count, Some(3));
    assert_eq!(context.gender, Gender::Masculine);

    // Test with parameters
    let context = translation_context!("name" => "Alice", "age" => 25);
    assert!(context.params.contains_key("name"));
    assert!(context.params.contains_key("age"));
}

#[test]
fn test_pluralization_helper() {
    // Test English pluralization
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::English, 1), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::English, 0), "other");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::English, 2), "other");

    // Test Spanish pluralization (0 and 1 are singular)
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::Spanish, 0), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::Spanish, 1), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::Spanish, 2), "other");

    // Test French pluralization (same as Spanish)
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::French, 0), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::French, 1), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::French, 2), "other");

    // Test German pluralization (same as English)
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::German, 1), "one");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::German, 0), "other");
    assert_eq!(PluralizationHelper::get_plural_form(&Locale::German, 2), "other");
}

#[test]
fn test_plural_context_macro() {
    // Test basic plural context
    let context = plural_context!(&Locale::English, 5);
    assert_eq!(context.count, Some(5));

    // Test percentage context
    let context = plural_context!(&Locale::English, 3, 10);
    assert_eq!(context.count, Some(3));
    assert!(context.params.contains_key("total"));
    assert!(context.params.contains_key("percentage"));
}

#[test]
fn test_gender_functionality() {
    // Test gender string conversion
    assert_eq!(Gender::Masculine.as_str(), "masculine");
    assert_eq!(Gender::Feminine.as_str(), "feminine");
    assert_eq!(Gender::Neuter.as_str(), "neuter");
    assert_eq!(Gender::Unknown.as_str(), "unknown");

    // Test gender parsing
    assert_eq!(Gender::from_str("masculine"), Some(Gender::Masculine));
    assert_eq!(Gender::from_str("m"), Some(Gender::Masculine));
    assert_eq!(Gender::from_str("feminine"), Some(Gender::Feminine));
    assert_eq!(Gender::from_str("f"), Some(Gender::Feminine));
    assert_eq!(Gender::from_str("invalid"), None);
}

#[test]
fn test_context_fluent_args_conversion() {
    let context = TranslationContext::with_count(5)
        .set_gender(Gender::Feminine)
        .add_param("name", "Alice")
        .add_param("age", 25);

    let args = context.to_fluent_args();
    
    // Check that all expected parameters are present
    assert!(args.get("count").is_some());
    assert!(args.get("gender").is_some());
    assert!(args.get("name").is_some());
    assert!(args.get("age").is_some());
}

#[test]
fn test_context_aware_translation_with_manager() {
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load additional locales for testing
    manager.load_locale(&Locale::Spanish).unwrap();

    // Test basic pluralization with time units
    let result = manager.format_plural("time-seconds", &Locale::English, 1);
    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(message.contains("1 second")); // Should be singular

    let result = manager.format_plural("time-seconds", &Locale::English, 5);
    assert!(result.is_ok());
    let message = result.unwrap();
    assert!(message.contains("5 seconds")); // Should be plural

    // Test with Spanish (different pluralization rules)
    let result = manager.format_plural("time-seconds", &Locale::Spanish, 0);
    assert!(result.is_ok());
    // Should use singular form for 0 in Spanish

    let result = manager.format_plural("time-seconds", &Locale::Spanish, 2);
    assert!(result.is_ok());
    // Should use plural form for 2 in Spanish
}

#[test]
fn test_helper_context_functions() {
    // Test time duration context
    let context = TranslationContext::for_time_duration(3661); // 1 hour, 1 minute, 1 second
    assert_eq!(context.count, Some(3661));
    assert!(context.params.contains_key("hours"));
    assert!(context.params.contains_key("minutes"));
    assert!(context.params.contains_key("days"));

    // Test user stats context
    let context = TranslationContext::for_user_stats(75, 100);
    assert_eq!(context.count, Some(75));
    assert!(context.params.contains_key("total"));
    assert!(context.params.contains_key("percentage"));

    // Test command stats context
    let context = TranslationContext::for_command_stats(80, 20);
    assert_eq!(context.count, Some(100)); // total
    assert!(context.params.contains_key("successful"));
    assert!(context.params.contains_key("failed"));
    assert!(context.params.contains_key("success_rate"));
}

#[test]
fn test_pluralization_helper_contexts() {
    // Test server/user context
    let context = PluralizationHelper::server_user_context(1, 5);
    assert!(context.params.contains_key("servers"));
    assert!(context.params.contains_key("users"));
    assert!(context.params.contains_key("server_plural"));
    assert!(context.params.contains_key("user_plural"));

    // Test command stats context
    let context = PluralizationHelper::command_stats_context(100, 80, 20);
    assert_eq!(context.count, Some(100));
    assert!(context.params.contains_key("successful"));
    assert!(context.params.contains_key("failed"));
    assert!(context.params.contains_key("success_rate"));

    // Test duration formatting
    let (_unit, count, context) = PluralizationHelper::format_duration_context(3661);
    assert_eq!(count, 1); // Should be 1 hour
    assert!(context.params.contains_key("seconds"));
    assert!(context.params.contains_key("minutes"));
    assert!(context.params.contains_key("hours"));
}

#[test]
fn test_gender_suffix_functionality() {
    let context = TranslationContext::with_gender(Gender::Feminine);
    
    // Languages with grammatical gender should return suffix
    assert_eq!(context.get_gender_suffix(&Locale::Spanish), Some("feminine"));
    assert_eq!(context.get_gender_suffix(&Locale::French), Some("feminine"));
    assert_eq!(context.get_gender_suffix(&Locale::German), Some("feminine"));
    
    // English doesn't use grammatical gender
    assert_eq!(context.get_gender_suffix(&Locale::English), None);
    
    // Unknown gender should not return suffix
    let context = TranslationContext::with_gender(Gender::Unknown);
    assert_eq!(context.get_gender_suffix(&Locale::Spanish), None);
}

#[test]
fn test_complex_pluralization_detection() {
    assert!(!PluralizationHelper::has_complex_pluralization(&Locale::English));
    assert!(!PluralizationHelper::has_complex_pluralization(&Locale::German));
    assert!(PluralizationHelper::has_complex_pluralization(&Locale::Spanish));
    assert!(PluralizationHelper::has_complex_pluralization(&Locale::French));
}
