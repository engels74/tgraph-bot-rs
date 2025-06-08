//! Build script for tgraph-i18n crate
//! 
//! This script validates all Fluent locale files at compile time to ensure:
//! - All locale files have the same message keys
//! - All Fluent syntax is valid
//! - Parameter consistency across translations
//! - No missing translations

use std::collections::{HashMap, HashSet};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use fluent_syntax::parser::parse;
use fluent_syntax::ast::{Entry, Pattern, PatternElement, Expression, InlineExpression};

/// Extract message keys and their parameters from a Fluent file
fn extract_messages_and_params(content: &str) -> Result<HashMap<String, HashSet<String>>, String> {
    let resource = parse(content)
        .map_err(|errors| format!("Parse errors: {:?}", errors))?;
    
    let mut messages = HashMap::new();
    
    for entry in resource.body {
        if let Entry::Message(message) = entry {
            let key = message.id.name.to_string();
            let mut params = HashSet::new();
            
            if let Some(Pattern { elements }) = message.value {
                extract_params_from_pattern(&elements, &mut params);
            }

            // Also check attributes
            for attribute in message.attributes {
                let Pattern { elements } = attribute.value;
                extract_params_from_pattern(&elements, &mut params);
            }
            
            messages.insert(key, params);
        }
    }
    
    Ok(messages)
}

/// Recursively extract parameter names from pattern elements
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

/// Extract parameter names from expressions
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

/// Extract parameter names from inline expressions
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

/// Validate a single locale file
fn validate_locale_file(path: &Path) -> Result<HashMap<String, HashSet<String>>, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
    
    extract_messages_and_params(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
}

/// Find all locale files
fn find_locale_files() -> Result<HashMap<String, PathBuf>, String> {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")
        .map_err(|_| "CARGO_MANIFEST_DIR not set")?;
    
    let locales_dir = Path::new(&manifest_dir).parent()
        .ok_or("Failed to get parent directory")?
        .join("locales");
    
    if !locales_dir.exists() {
        return Err(format!("Locales directory not found: {}", locales_dir.display()));
    }
    
    let mut locale_files = HashMap::new();
    
    for entry in fs::read_dir(&locales_dir)
        .map_err(|e| format!("Failed to read locales directory: {}", e))? {
        let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
        let path = entry.path();
        
        if path.is_dir() {
            let locale_name = path.file_name()
                .and_then(|n| n.to_str())
                .ok_or_else(|| format!("Invalid locale directory name: {}", path.display()))?;
            
            let main_ftl = path.join("main.ftl");
            if main_ftl.exists() {
                locale_files.insert(locale_name.to_string(), main_ftl);
            }
        }
    }
    
    if locale_files.is_empty() {
        return Err("No locale files found".to_string());
    }
    
    Ok(locale_files)
}

/// Main validation function
fn validate_locales() -> Result<(), String> {
    println!("cargo:rerun-if-changed=../locales");
    
    let locale_files = find_locale_files()?;
    
    println!("Found {} locale files", locale_files.len());
    
    let mut all_messages: HashMap<String, HashMap<String, HashSet<String>>> = HashMap::new();
    let mut validation_errors = Vec::new();
    
    // Validate each locale file
    for (locale, path) in &locale_files {
        match validate_locale_file(path) {
            Ok(messages) => {
                println!("✅ {}: {} messages", locale, messages.len());
                all_messages.insert(locale.clone(), messages);
            }
            Err(e) => {
                validation_errors.push(format!("❌ {}: {}", locale, e));
            }
        }
    }
    
    if !validation_errors.is_empty() {
        return Err(format!("Validation errors:\n{}", validation_errors.join("\n")));
    }
    
    // Check consistency across locales
    if all_messages.len() > 1 {
        let reference_locale = all_messages.keys().next().unwrap();
        let reference_messages = &all_messages[reference_locale];
        
        for (locale, messages) in &all_messages {
            if locale == reference_locale {
                continue;
            }
            
            // Check for missing keys
            for key in reference_messages.keys() {
                if !messages.contains_key(key) {
                    validation_errors.push(format!("❌ {}: Missing message key '{}'", locale, key));
                }
            }
            
            // Check for extra keys
            for key in messages.keys() {
                if !reference_messages.contains_key(key) {
                    validation_errors.push(format!("❌ {}: Extra message key '{}'", locale, key));
                }
            }
            
            // Check parameter consistency
            for (key, ref_params) in reference_messages {
                if let Some(locale_params) = messages.get(key) {
                    if ref_params != locale_params {
                        validation_errors.push(format!(
                            "❌ {}: Parameter mismatch for '{}'. Expected: {:?}, Found: {:?}",
                            locale, key, ref_params, locale_params
                        ));
                    }
                }
            }
        }
    }
    
    if !validation_errors.is_empty() {
        return Err(format!("Consistency errors:\n{}", validation_errors.join("\n")));
    }
    
    println!("🎉 All locale files validated successfully!");
    Ok(())
}

fn main() {
    if let Err(e) = validate_locales() {
        eprintln!("Locale validation failed:\n{}", e);
        process::exit(1);
    }
}
