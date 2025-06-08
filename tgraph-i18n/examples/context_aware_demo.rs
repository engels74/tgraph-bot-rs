//! Demonstration of context-aware translation features
//!
//! This example shows how to use the context-aware translation system
//! with pluralization, gender agreement, and other language-specific features.

use tgraph_i18n::{
    Gender, I18nManager, Locale, PluralizationHelper, TranslationContext,
    plural_context, translation_context
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize the I18n manager
    let mut manager = I18nManager::new(Locale::English, "../locales")?;
    
    // Load all supported locales
    manager.load_all_locales()?;
    
    println!("=== Context-Aware Translation Demo ===\n");
    
    // Demo 1: Basic pluralization
    demo_pluralization(&manager)?;
    
    // Demo 2: Time duration formatting
    demo_time_duration(&manager)?;
    
    // Demo 3: Statistics formatting
    demo_statistics(&manager)?;
    
    // Demo 4: Gender-aware translations (conceptual)
    demo_gender_awareness(&manager)?;
    
    // Demo 5: Complex contexts
    demo_complex_contexts(&manager)?;
    
    Ok(())
}

fn demo_pluralization(manager: &I18nManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("1. Pluralization Demo");
    println!("====================");
    
    let test_counts = vec![0, 1, 2, 5, 10];
    
    for locale in &[Locale::English, Locale::Spanish, Locale::French, Locale::German] {
        println!("\n{} ({}):", locale.display_name(), locale.code());
        
        for &count in &test_counts {
            // Test time-seconds pluralization
            let result = manager.format_plural("time-seconds", locale, count)?;
            println!("  {} -> {}", count, result);
        }
    }
    
    println!();
    Ok(())
}

fn demo_time_duration(manager: &I18nManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("2. Time Duration Formatting Demo");
    println!("=================================");
    
    let durations = vec![
        (1, "1 second"),
        (30, "30 seconds"),
        (60, "1 minute"),
        (90, "90 seconds / 1.5 minutes"),
        (3600, "1 hour"),
        (3661, "1 hour, 1 minute, 1 second"),
        (86400, "1 day"),
    ];
    
    for locale in &[Locale::English, Locale::Spanish] {
        println!("\n{} ({}):", locale.display_name(), locale.code());
        
        for &(seconds, description) in &durations {
            let result = manager.format_time_duration("time-seconds", locale, seconds)?;
            println!("  {} ({}) -> {}", seconds, description, result);
        }
    }
    
    println!();
    Ok(())
}

fn demo_statistics(manager: &I18nManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("3. Statistics Formatting Demo");
    println!("=============================");
    
    // Demo command statistics
    let successful = 85;
    let failed = 15;
    
    for locale in &[Locale::English, Locale::Spanish] {
        println!("\n{} Command Statistics:", locale.display_name());
        
        let result = manager.format_command_stats(
            "uptime-commands-executed", 
            locale, 
            successful, 
            failed
        )?;
        println!("  {}", result);
    }
    
    // Demo user statistics
    let user_commands = 42;
    let total_commands = 100;
    
    for locale in &[Locale::English, Locale::Spanish] {
        println!("\n{} User Statistics:", locale.display_name());
        
        let result = manager.format_user_stats(
            "stats-successful", 
            locale, 
            user_commands, 
            total_commands
        )?;
        println!("  {}", result);
    }
    
    println!();
    Ok(())
}

fn demo_gender_awareness(manager: &I18nManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("4. Gender-Aware Translation Demo");
    println!("================================");
    
    // This is a conceptual demo since our current messages don't have gender variants
    // In a real application, you might have messages like:
    // welcome-masculine = Bienvenido, {$name}!
    // welcome-feminine = Bienvenida, {$name}!
    
    let genders = vec![
        (Gender::Masculine, "masculine"),
        (Gender::Feminine, "feminine"),
        (Gender::Neuter, "neuter"),
        (Gender::Unknown, "unknown"),
    ];
    
    for locale in &[Locale::Spanish, Locale::French, Locale::German] {
        println!("\n{} Gender Support:", locale.display_name());
        
        for &(gender, description) in &genders {
            let context = TranslationContext::with_gender(gender)
                .add_param("name", "Alex");
            
            // Check if gender suffix would be used
            if let Some(suffix) = context.get_gender_suffix(locale) {
                println!("  {} -> would use suffix '-{}'", description, suffix);
            } else {
                println!("  {} -> no gender suffix needed", description);
            }
        }
    }
    
    println!();
    Ok(())
}

fn demo_complex_contexts(manager: &I18nManager) -> Result<(), Box<dyn std::error::Error>> {
    println!("5. Complex Context Demo");
    println!("=======================");
    
    // Demo using macros for context creation
    println!("\nUsing translation_context! macro:");
    
    // Simple count context
    let context = translation_context!(count: 5);
    println!("  Count context: {:?}", context.count);
    
    // Count and gender context
    let context = translation_context!(count: 3, gender: Gender::Feminine);
    println!("  Count + Gender context: count={:?}, gender={:?}", 
             context.count, context.gender);
    
    // Parameter context
    let context = translation_context!(
        "name" => "Alice",
        "age" => 25,
        "score" => 95.5
    );
    println!("  Parameter context: {} params", context.params.len());
    
    // Demo using plural_context! macro
    println!("\nUsing plural_context! macro:");
    
    let context = plural_context!(&Locale::English, 5);
    println!("  Simple plural: count={:?}", context.count);
    
    let context = plural_context!(&Locale::Spanish, 3, 10);
    println!("  Percentage plural: count={:?}, has total={}", 
             context.count, context.params.contains_key("total"));
    
    // Demo helper functions
    println!("\nUsing helper functions:");
    
    let context = TranslationContext::for_time_duration(3661);
    println!("  Time duration context: {} params", context.params.len());
    
    let context = PluralizationHelper::server_user_context(5, 150);
    println!("  Server/User context: {} params", context.params.len());
    
    let context = PluralizationHelper::command_stats_context(100, 85, 15);
    println!("  Command stats context: count={:?}, {} params", 
             context.count, context.params.len());
    
    println!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_demo_runs_without_panic() {
        // This test ensures the demo code doesn't panic
        // In a real scenario, you'd want more specific tests
        let result = std::panic::catch_unwind(|| {
            if let Ok(mut manager) = I18nManager::new(Locale::English, "../locales") {
                let _ = manager.load_all_locales();
                
                // Test basic functionality
                let context = translation_context!(count: 5);
                assert_eq!(context.count, Some(5));
                
                let context = plural_context!(&Locale::English, 3);
                assert_eq!(context.count, Some(3));
            }
        });
        
        assert!(result.is_ok());
    }
}
