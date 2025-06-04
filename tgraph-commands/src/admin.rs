//! Admin commands for the bot

/// Admin command implementations
pub struct AdminCommands;

impl AdminCommands {
    /// Create new admin commands handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for AdminCommands {
    fn default() -> Self {
        Self::new()
    }
} 