//! Compile-time and runtime validation using custom derive macros.

use crate::schema::Config;
use tgraph_common::{Result, TGraphError};

/// Configuration validator.
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validates a configuration.
    pub fn validate(config: &Config) -> Result<()> {
        config.validate().map_err(Into::into)
    }
}
