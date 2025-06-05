//! Cooldown system for rate limiting command usage

use dashmap::DashMap;
use poise::serenity_prelude::{UserId, ChannelId};
use std::time::{Duration, Instant};
use thiserror::Error;
use tracing::debug;

/// Errors that can occur during cooldown operations
#[derive(Error, Debug)]
pub enum CooldownError {
    #[error("User {user_id} is on cooldown for command '{command}' (remaining: {remaining_seconds}s)")]
    UserOnCooldown {
        user_id: u64,
        command: String,
        remaining_seconds: u64,
    },
    #[error("Channel {channel_id} is on cooldown for command '{command}' (remaining: {remaining_seconds}s)")]
    ChannelOnCooldown {
        channel_id: u64,
        command: String,
        remaining_seconds: u64,
    },
    #[error("Global cooldown active for command '{command}' (remaining: {remaining_seconds}s)")]
    GlobalOnCooldown {
        command: String,
        remaining_seconds: u64,
    },
}

/// Cooldown key for tracking different types of cooldowns
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
enum CooldownKey {
    /// Per-user cooldown: (command_name, user_id)
    User(String, u64),
    /// Per-channel cooldown: (command_name, channel_id)
    Channel(String, u64),
    /// Global cooldown: command_name
    Global(String),
}

/// Cooldown configuration for a command
#[derive(Debug, Clone)]
pub struct CooldownConfig {
    /// Per-user cooldown duration
    pub user: Option<Duration>,
    /// Per-channel cooldown duration
    pub channel: Option<Duration>,
    /// Global cooldown duration
    pub global: Option<Duration>,
}

impl Default for CooldownConfig {
    fn default() -> Self {
        Self {
            user: Some(Duration::from_secs(3)), // Default 3 second user cooldown
            channel: None,
            global: None,
        }
    }
}

/// Manager for handling command cooldowns
#[derive(Debug)]
pub struct CooldownManager {
    /// Storage for cooldown timestamps
    cooldowns: DashMap<CooldownKey, Instant>,
}

impl CooldownManager {
    /// Create a new cooldown manager
    pub fn new() -> Self {
        Self {
            cooldowns: DashMap::new(),
        }
    }

    /// Check if a command is on cooldown and return an error if it is
    pub fn check_cooldown(
        &self,
        command: &str,
        user_id: UserId,
        channel_id: Option<ChannelId>,
        config: &CooldownConfig,
    ) -> Result<(), CooldownError> {
        let now = Instant::now();

        // Check global cooldown
        if let Some(global_duration) = config.global {
            let key = CooldownKey::Global(command.to_string());
            if let Some(last_used) = self.cooldowns.get(&key) {
                let elapsed = now.duration_since(*last_used);
                if elapsed < global_duration {
                    let remaining = global_duration - elapsed;
                    return Err(CooldownError::GlobalOnCooldown {
                        command: command.to_string(),
                        remaining_seconds: remaining.as_secs(),
                    });
                }
            }
        }

        // Check channel cooldown
        if let (Some(channel_duration), Some(channel_id)) = (config.channel, channel_id) {
            let key = CooldownKey::Channel(command.to_string(), channel_id.get());
            if let Some(last_used) = self.cooldowns.get(&key) {
                let elapsed = now.duration_since(*last_used);
                if elapsed < channel_duration {
                    let remaining = channel_duration - elapsed;
                    return Err(CooldownError::ChannelOnCooldown {
                        channel_id: channel_id.get(),
                        command: command.to_string(),
                        remaining_seconds: remaining.as_secs(),
                    });
                }
            }
        }

        // Check user cooldown
        if let Some(user_duration) = config.user {
            let key = CooldownKey::User(command.to_string(), user_id.get());
            if let Some(last_used) = self.cooldowns.get(&key) {
                let elapsed = now.duration_since(*last_used);
                if elapsed < user_duration {
                    let remaining = user_duration - elapsed;
                    return Err(CooldownError::UserOnCooldown {
                        user_id: user_id.get(),
                        command: command.to_string(),
                        remaining_seconds: remaining.as_secs(),
                    });
                }
            }
        }

        Ok(())
    }

    /// Apply cooldowns after successful command execution
    pub fn apply_cooldown(
        &self,
        command: &str,
        user_id: UserId,
        channel_id: Option<ChannelId>,
        config: &CooldownConfig,
    ) {
        let now = Instant::now();

        debug!("Applying cooldowns for command '{}' (user: {})", command, user_id);

        // Apply global cooldown
        if config.global.is_some() {
            let key = CooldownKey::Global(command.to_string());
            self.cooldowns.insert(key, now);
        }

        // Apply channel cooldown
        if let (Some(_), Some(channel_id)) = (config.channel, channel_id) {
            let key = CooldownKey::Channel(command.to_string(), channel_id.get());
            self.cooldowns.insert(key, now);
        }

        // Apply user cooldown
        if config.user.is_some() {
            let key = CooldownKey::User(command.to_string(), user_id.get());
            self.cooldowns.insert(key, now);
        }
    }

    /// Clear all cooldowns for a specific command
    pub fn clear_command_cooldowns(&self, command: &str) {
        let keys_to_remove: Vec<CooldownKey> = self.cooldowns
            .iter()
            .filter_map(|entry| {
                match entry.key() {
                    CooldownKey::User(cmd, _) |
                    CooldownKey::Channel(cmd, _) |
                    CooldownKey::Global(cmd) if cmd == command => {
                        Some(entry.key().clone())
                    }
                    _ => None,
                }
            })
            .collect();

        for key in keys_to_remove {
            self.cooldowns.remove(&key);
        }

        debug!("Cleared all cooldowns for command '{}'", command);
    }

    /// Clear all cooldowns for a specific user
    pub fn clear_user_cooldowns(&self, user_id: UserId) {
        let keys_to_remove: Vec<CooldownKey> = self.cooldowns
            .iter()
            .filter_map(|entry| {
                match entry.key() {
                    CooldownKey::User(_, uid) if *uid == user_id.get() => {
                        Some(entry.key().clone())
                    }
                    _ => None,
                }
            })
            .collect();

        for key in keys_to_remove {
            self.cooldowns.remove(&key);
        }

        debug!("Cleared all cooldowns for user {}", user_id);
    }

    /// Get the number of active cooldowns
    pub fn active_cooldowns(&self) -> usize {
        self.cooldowns.len()
    }

    /// Clean up expired cooldowns (should be called periodically)
    pub fn cleanup_expired(&self) {
        let now = Instant::now();
        let mut expired_keys = Vec::new();

        for entry in self.cooldowns.iter() {
            // Assume max reasonable cooldown is 1 hour for cleanup purposes
            if now.duration_since(*entry.value()) > Duration::from_secs(3600) {
                expired_keys.push(entry.key().clone());
            }
        }

        for key in expired_keys {
            self.cooldowns.remove(&key);
        }

        debug!("Cleaned up expired cooldowns");
    }
}

impl Default for CooldownManager {
    fn default() -> Self {
        Self::new()
    }
} 