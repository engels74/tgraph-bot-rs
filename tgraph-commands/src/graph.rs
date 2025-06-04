//! Graph-related commands for the bot

/// Graph command implementations
pub struct GraphCommands;

impl GraphCommands {
    /// Create new graph commands handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for GraphCommands {
    fn default() -> Self {
        Self::new()
    }
} 