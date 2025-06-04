//! Command registry for managing bot commands

use tgraph_common::Result;

/// Registry for managing bot commands
pub struct CommandRegistry;

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self
    }

    /// Register all available commands
    pub fn register_all(&mut self) -> Result<()> {
        // TODO: Implement command registration
        Ok(())
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
} 