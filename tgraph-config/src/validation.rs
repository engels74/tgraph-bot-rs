//! Validation utilities and regex patterns

use regex::Regex;
use std::str::FromStr;
use std::sync::LazyLock;
use validator::ValidationError;

/// Regex pattern for validating hex color codes (e.g., #FFFFFF, #FF0000)
pub static HEX_COLOR_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^#[0-9A-Fa-f]{6}$").expect("Invalid hex color regex pattern")
});

/// Validate a cron expression
pub fn validate_cron_expression(cron_expr: &str) -> Result<(), ValidationError> {
    if cron_expr.is_empty() {
        return Err(ValidationError::new("empty_cron_expression"));
    }

    match cron::Schedule::from_str(cron_expr) {
        Ok(_) => Ok(()),
        Err(_) => Err(ValidationError::new("invalid_cron_expression")),
    }
}

/// Validate timezone string (basic check for common IANA timezone format)
pub fn validate_timezone(timezone: &str) -> Result<(), ValidationError> {
    if timezone.is_empty() {
        return Err(ValidationError::new("empty_timezone"));
    }

    // Basic validation - should contain at least one slash for Area/Location format
    // or be "UTC" which is a special case
    if timezone == "UTC" || timezone.contains('/') {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_timezone_format"))
    }
}

/// Validate Discord token format (basic check)
pub fn validate_discord_token(token: &str) -> Result<(), ValidationError> {
    if token.is_empty() {
        return Err(ValidationError::new("empty_discord_token"));
    }

    // Discord bot tokens typically have a specific format: bot_id.timestamp.signature
    // Basic check for dot-separated structure
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() == 3 && parts.iter().all(|part| !part.is_empty()) {
        Ok(())
    } else {
        Err(ValidationError::new("invalid_discord_token_format"))
    }
}

/// Validate file path (basic check for valid path characters)
pub fn validate_file_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() {
        return Err(ValidationError::new("empty_file_path"));
    }

    // Check for invalid characters that would cause issues on most filesystems
    // Note: Colon is allowed for Windows drive letters (C:\)
    let invalid_chars = ['<', '>', '"', '|', '?', '*'];
    if path.chars().any(|c| invalid_chars.contains(&c)) {
        return Err(ValidationError::new("invalid_file_path_characters"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hex_color_regex() {
        // Valid hex colors
        assert!(HEX_COLOR_REGEX.is_match("#FFFFFF"));
        assert!(HEX_COLOR_REGEX.is_match("#000000"));
        assert!(HEX_COLOR_REGEX.is_match("#FF0000"));
        assert!(HEX_COLOR_REGEX.is_match("#00FF00"));
        assert!(HEX_COLOR_REGEX.is_match("#0000FF"));
        assert!(HEX_COLOR_REGEX.is_match("#abc123"));
        assert!(HEX_COLOR_REGEX.is_match("#ABC123"));

        // Invalid hex colors
        assert!(!HEX_COLOR_REGEX.is_match("FFFFFF"));  // Missing #
        assert!(!HEX_COLOR_REGEX.is_match("#FFF"));    // Too short
        assert!(!HEX_COLOR_REGEX.is_match("#FFFFFFF")); // Too long
        assert!(!HEX_COLOR_REGEX.is_match("#GGGGGG")); // Invalid characters
        assert!(!HEX_COLOR_REGEX.is_match("#FF FF FF")); // Spaces
        assert!(!HEX_COLOR_REGEX.is_match("")); // Empty
    }

    #[test]
    fn test_validate_cron_expression() {
        // Valid cron expressions (6-field format: sec min hour day month weekday)
        assert!(validate_cron_expression("0 0 0 * * *").is_ok());      // Daily at midnight
        assert!(validate_cron_expression("0 */5 * * * *").is_ok());    // Every 5 minutes
        assert!(validate_cron_expression("0 0 9-18 * * MON-FRI").is_ok()); // Weekdays 9-18
        assert!(validate_cron_expression("0 0 0 1 * *").is_ok());      // Monthly
        assert!(validate_cron_expression("0 0 2 * * 7").is_ok());      // Sunday at 2 AM

        // Invalid cron expressions
        assert!(validate_cron_expression("").is_err());              // Empty
        assert!(validate_cron_expression("invalid").is_err());       // Invalid format
        assert!(validate_cron_expression("0 0 0 32 * *").is_err());  // Invalid day
        assert!(validate_cron_expression("0 0 25 * * *").is_err());  // Invalid hour
        assert!(validate_cron_expression("0 0 2 * * 0").is_err());   // Invalid weekday (0 not supported)
    }

    #[test]
    fn test_validate_timezone() {
        // Valid timezones
        assert!(validate_timezone("UTC").is_ok());
        assert!(validate_timezone("America/New_York").is_ok());
        assert!(validate_timezone("Europe/London").is_ok());
        assert!(validate_timezone("Asia/Tokyo").is_ok());
        assert!(validate_timezone("Australia/Sydney").is_ok());

        // Invalid timezones
        assert!(validate_timezone("").is_err());                     // Empty
        assert!(validate_timezone("Invalid").is_err());              // No slash
        assert!(validate_timezone("America").is_err());              // No location
    }

    #[test]
    fn test_validate_discord_token() {
        // Valid token format (fake tokens for testing)
        assert!(validate_discord_token("792715454196088842.X-hvzA.Ovy4MCQywSkoMRRclStW4xAYK7I").is_ok());
        assert!(validate_discord_token("123456789.abcdef.ghijklmnop").is_ok());

        // Invalid token formats
        assert!(validate_discord_token("").is_err());                // Empty
        assert!(validate_discord_token("invalid_token").is_err());   // No dots
        assert!(validate_discord_token("123.456.").is_err());        // Empty third part
        assert!(validate_discord_token("123..456").is_err());        // Empty middle part
        assert!(validate_discord_token("123.456.789.abc").is_err()); // Too many parts
    }

    #[test]
    fn test_validate_file_path() {
        // Valid file paths
        assert!(validate_file_path("/var/log/app.log").is_ok());
        assert!(validate_file_path("./config.yaml").is_ok());
        assert!(validate_file_path("C:\\Program Files\\App\\config.txt").is_ok());
        assert!(validate_file_path("app.log").is_ok());

        // Invalid file paths
        assert!(validate_file_path("").is_err());                    // Empty
        assert!(validate_file_path("file<name.txt").is_err());       // Invalid character <
        assert!(validate_file_path("file>name.txt").is_err());       // Invalid character >
        assert!(validate_file_path("file\"name.txt").is_err());      // Invalid character "
        assert!(validate_file_path("file|name.txt").is_err());       // Invalid character |
        assert!(validate_file_path("file?name.txt").is_err());       // Invalid character ?
        assert!(validate_file_path("file*name.txt").is_err());       // Invalid character *
    }
} 