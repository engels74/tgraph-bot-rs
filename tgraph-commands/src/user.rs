//! User commands for the bot

/// User command implementations
pub struct UserCommands;

impl UserCommands {
    /// Create new user commands handler
    pub fn new() -> Self {
        Self
    }
}

impl Default for UserCommands {
    fn default() -> Self {
        Self::new()
    }
} 