//! Command registry for managing bot commands

use poise::Command;
use tgraph_common::Result;
use crate::context::{CommandContext, CommandError};

/// Registry for managing bot commands
pub struct CommandRegistry {
    /// List of registered commands
    commands: Vec<Command<CommandContext, CommandError>>,
}

impl CommandRegistry {
    /// Create a new command registry
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
        }
    }

    /// Register all available commands
    pub fn register_all(&mut self) -> Result<()> {
        // Register basic commands
        self.commands.push(crate::user::about());
        self.commands.push(crate::user::uptime());
        self.commands.push(crate::admin::update_graphs());

        Ok(())
    }

    /// Get all registered commands
    pub fn commands(&self) -> &[Command<CommandContext, CommandError>] {
        &self.commands
    }

    /// Get the number of registered commands
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
} 