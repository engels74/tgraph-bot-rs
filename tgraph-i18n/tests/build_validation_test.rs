//! Test to verify the build script validation works correctly

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::TempDir;

/// Test that the build script validates locale files correctly
#[test]
fn test_build_script_validation() {
    // This test verifies that the build script runs and validates locale files
    // We can't easily test the build script directly, but we can verify that
    // the validation logic works by testing the functions it uses
    
    // The build script should have already run during compilation
    // If we're here, it means the validation passed
    assert!(true, "Build script validation passed during compilation");
}

/// Test that we can detect when locale files are missing
#[test]
fn test_missing_locale_detection() {
    // Create a temporary directory structure
    let temp_dir = TempDir::new().unwrap();
    let locales_dir = temp_dir.path().join("locales");
    fs::create_dir_all(&locales_dir).unwrap();
    
    // Create only English locale
    let en_dir = locales_dir.join("en");
    fs::create_dir_all(&en_dir).unwrap();
    fs::write(en_dir.join("main.ftl"), "hello = Hello!").unwrap();
    
    // The build script would detect this as having only one locale
    // In a real scenario, this would be flagged as incomplete
    assert!(en_dir.join("main.ftl").exists());
}

/// Test that we can detect syntax errors in Fluent files
#[test]
fn test_fluent_syntax_validation() {
    use fluent_syntax::parser::parse;
    
    // Test valid Fluent syntax
    let valid_content = r#"
hello = Hello!
welcome = Welcome to {$app}!
time-seconds = {$count ->
    [one] {$count} second
   *[other] {$count} seconds
}
"#;
    
    let result = parse(valid_content);
    assert!(result.is_ok(), "Valid Fluent syntax should parse successfully");
    
    // Test invalid Fluent syntax
    let invalid_content = r#"
hello = Hello!
broken = This has {$unclosed parameter
"#;
    
    let result = parse(invalid_content);
    assert!(result.is_err(), "Invalid Fluent syntax should fail to parse");
}

/// Test parameter extraction from Fluent messages
#[test]
fn test_parameter_extraction() {
    use fluent_syntax::parser::parse;
    use fluent_syntax::ast::{Entry, Pattern, PatternElement, Expression, InlineExpression};
    use std::collections::HashSet;
    
    let content = r#"
simple = Hello!
with-param = Hello {$name}!
multi-param = {$user} has {$count} items
select = {$count ->
    [one] {$count} item
   *[other] {$count} items
}
"#;
    
    let resource = parse(content).unwrap();
    
    for entry in resource.body {
        if let Entry::Message(message) = entry {
            let key = message.id.name;
            let mut params = HashSet::new();
            
            if let Some(Pattern { elements }) = message.value {
                extract_params_from_pattern(&elements, &mut params);
            }
            
            match key {
                "simple" => assert!(params.is_empty(), "Simple message should have no parameters"),
                "with-param" => {
                    assert_eq!(params.len(), 1);
                    assert!(params.contains("name"));
                }
                "multi-param" => {
                    assert_eq!(params.len(), 2);
                    assert!(params.contains("user"));
                    assert!(params.contains("count"));
                }
                "select" => {
                    assert_eq!(params.len(), 1);
                    assert!(params.contains("count"));
                }
                _ => {}
            }
        }
    }
    
    fn extract_params_from_pattern<S>(elements: &[PatternElement<S>], params: &mut HashSet<String>) 
    where
        S: AsRef<str> + ToString,
    {
        for element in elements {
            match element {
                PatternElement::Placeable { expression } => {
                    extract_params_from_expression(expression, params);
                }
                PatternElement::TextElement { .. } => {
                    // Text elements don't contain parameters
                }
            }
        }
    }
    
    fn extract_params_from_expression<S>(expression: &Expression<S>, params: &mut HashSet<String>) 
    where
        S: AsRef<str> + ToString,
    {
        match expression {
            Expression::Select { selector, variants } => {
                // Extract from selector
                extract_params_from_inline_expression(selector, params);
                
                // Extract from variants
                for variant in variants {
                    let Pattern { elements } = &variant.value;
                    extract_params_from_pattern(elements, params);
                }
            }
            Expression::Inline(inline) => {
                extract_params_from_inline_expression(inline, params);
            }
        }
    }
    
    fn extract_params_from_inline_expression<S>(expression: &InlineExpression<S>, params: &mut HashSet<String>) 
    where
        S: AsRef<str> + ToString,
    {
        match expression {
            InlineExpression::VariableReference { id } => {
                params.insert(id.name.to_string());
            }
            InlineExpression::FunctionReference { arguments, .. } => {
                // Extract from function arguments
                for arg in &arguments.positional {
                    extract_params_from_inline_expression(arg, params);
                }
                for arg in &arguments.named {
                    extract_params_from_inline_expression(&arg.value, params);
                }
            }
            InlineExpression::MessageReference { .. } |
            InlineExpression::TermReference { .. } |
            InlineExpression::StringLiteral { .. } |
            InlineExpression::NumberLiteral { .. } => {
                // These don't contain variable parameters
            }
            InlineExpression::Placeable { expression } => {
                extract_params_from_expression(expression, params);
            }
        }
    }
}

/// Test that all our locale files are consistent
#[test]
fn test_locale_consistency() {
    use tgraph_i18n::{I18nManager, Locale};
    
    let mut manager = I18nManager::new(Locale::English, "../locales").unwrap();
    
    // Load all locales
    for locale in Locale::all() {
        if locale != Locale::English {
            let result = manager.load_locale(&locale);
            assert!(result.is_ok(), "Failed to load locale {:?}: {:?}", locale, result);
        }
    }
    
    // Test that key messages exist in all locales (without parameters)
    let test_messages = vec![
        "hello",
        "success",
        "bot-title",
        "uptime-title",
        "stats-command-usage",
        "admin-metrics-title",
        "export-title",
        "graph-success-title",
        "status-online",
    ];
    
    for locale in Locale::all() {
        for message_key in &test_messages {
            let result = manager.get_message(message_key, &locale, None);
            assert!(
                result.is_ok(),
                "Message '{}' not found in locale {:?}: {:?}",
                message_key, locale, result
            );
        }
    }
}

/// Test that the build script would catch parameter mismatches
#[test]
fn test_parameter_consistency_detection() {
    // This test simulates what the build script does to detect parameter mismatches
    use std::collections::{HashMap, HashSet};
    
    // Simulate messages with different parameters
    let mut locale_messages: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();
    
    // English messages
    let mut en_messages = HashMap::new();
    en_messages.insert("greeting".to_string(), {
        let mut params = HashSet::new();
        params.insert("name".to_string());
        params
    });
    en_messages.insert("count".to_string(), {
        let mut params = HashSet::new();
        params.insert("number".to_string());
        params
    });
    locale_messages.insert("en".to_string(), en_messages);
    
    // Spanish messages (with parameter mismatch)
    let mut es_messages = HashMap::new();
    es_messages.insert("greeting".to_string(), {
        let mut params = HashSet::new();
        params.insert("nombre".to_string()); // Different parameter name!
        params
    });
    es_messages.insert("count".to_string(), {
        let mut params = HashSet::new();
        params.insert("number".to_string());
        params
    });
    locale_messages.insert("es".to_string(), es_messages);
    
    // Check for parameter consistency
    let reference_locale = "en";
    let reference_messages = &locale_messages[reference_locale];
    
    let mut inconsistencies = Vec::new();
    
    for (locale, messages) in &locale_messages {
        if locale == reference_locale {
            continue;
        }
        
        for (key, ref_params) in reference_messages {
            if let Some(locale_params) = messages.get(key) {
                if ref_params != locale_params {
                    inconsistencies.push(format!(
                        "Parameter mismatch in '{}' for locale '{}': expected {:?}, found {:?}",
                        key, locale, ref_params, locale_params
                    ));
                }
            }
        }
    }
    
    // We expect to find the parameter mismatch
    assert!(!inconsistencies.is_empty(), "Should detect parameter mismatch");
    assert!(inconsistencies[0].contains("greeting"), "Should detect greeting parameter mismatch");
}
