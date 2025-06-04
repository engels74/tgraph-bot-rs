//! Utility functions used across the TGraph application

use crate::{Result, Timestamp};
use chrono::Utc;
use uuid::Uuid;

/// Generate a new unique entity ID
pub fn new_entity_id() -> Uuid {
    Uuid::new_v4()
}

/// Get the current timestamp
pub fn now() -> Timestamp {
    Utc::now()
}

/// Format a timestamp for display
pub fn format_timestamp(timestamp: &Timestamp) -> String {
    timestamp.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Validate that a string is not empty after trimming
pub fn validate_non_empty(value: &str, field_name: &str) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        Err(crate::TGraphError::new(format!("{} cannot be empty", field_name)))
    } else {
        Ok(trimmed.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entity_id() {
        let id1 = new_entity_id();
        let id2 = new_entity_id();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_validate_non_empty() {
        assert!(validate_non_empty("test", "field").is_ok());
        assert!(validate_non_empty("", "field").is_err());
        assert!(validate_non_empty("   ", "field").is_err());
    }
} 