//! Permission system for Discord bot commands

use poise::serenity_prelude::{self as serenity, UserId, RoleId, GuildId};
use std::collections::HashSet;
use tgraph_config::Config;
use tracing::{debug, warn};

/// Permission levels for bot commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Permission {
    /// Any user can execute this command
    User = 0,
    /// Moderators and above can execute this command
    Moderator = 1,
    /// Administrators and above can execute this command
    Administrator = 2,
    /// Only bot owners can execute this command
    Owner = 3,
}

impl Permission {
    /// Get the permission level name as a string
    pub fn as_str(&self) -> &'static str {
        match self {
            Permission::User => "User",
            Permission::Moderator => "Moderator", 
            Permission::Administrator => "Administrator",
            Permission::Owner => "Owner",
        }
    }
}

/// Permission manager for checking user permissions
#[derive(Debug)]
pub struct Permissions {
    /// Bot owner user IDs
    owners: HashSet<UserId>,
    /// Administrator user IDs
    administrators: HashSet<UserId>,
    /// Administrator role IDs (guild-specific)
    admin_roles: HashSet<RoleId>,
    /// Moderator role IDs (guild-specific)
    moderator_roles: HashSet<RoleId>,
}

impl Permissions {
    /// Create a new permissions manager from configuration
    pub fn new(config: &Config) -> Self {
        let owners = config.discord.owner_ids.iter()
            .map(|&id| UserId::new(id))
            .collect();

        let administrators = config.discord.admin_ids.iter()
            .map(|&id| UserId::new(id))
            .collect();

        let admin_roles = config.discord.admin_role_ids.iter()
            .map(|&id| RoleId::new(id))
            .collect();

        let moderator_roles = config.discord.moderator_role_ids.iter()
            .map(|&id| RoleId::new(id))
            .collect();

        Self {
            owners,
            administrators,
            admin_roles,
            moderator_roles,
        }
    }

    /// Check if a user has the required permission level
    pub async fn check_permission(
        &self,
        ctx: &serenity::Context,
        user_id: UserId,
        guild_id: Option<GuildId>,
        required: Permission,
    ) -> bool {
        debug!("Checking permission for user {} (required: {})", user_id, required.as_str());

        // Owner check - highest priority
        if self.owners.contains(&user_id) {
            debug!("User {} is bot owner", user_id);
            return true;
        }

        // For owner-only commands, no one else can proceed
        if required == Permission::Owner {
            warn!("User {} attempted to use owner-only command", user_id);
            return false;
        }

        // Administrator check
        if self.administrators.contains(&user_id) {
            debug!("User {} is configured as administrator", user_id);
            return required <= Permission::Administrator;
        }

        // Guild-based role checks
        if let Some(guild_id) = guild_id {
            if let Ok(member) = ctx.http.get_member(guild_id, user_id).await {
                // Check admin roles
                for role_id in &member.roles {
                    if self.admin_roles.contains(role_id) {
                        debug!("User {} has admin role {}", user_id, role_id);
                        return required <= Permission::Administrator;
                    }
                    if self.moderator_roles.contains(role_id) {
                        debug!("User {} has moderator role {}", user_id, role_id);
                        return required <= Permission::Moderator;
                    }
                }
            } else {
                warn!("Could not fetch member info for user {} in guild {}", user_id, guild_id);
            }
        }

        // Default to user permission level
        required <= Permission::User
    }

    /// Check if a user is a bot owner
    pub fn is_owner(&self, user_id: UserId) -> bool {
        self.owners.contains(&user_id)
    }

    /// Check if a user is an administrator
    pub fn is_administrator(&self, user_id: UserId) -> bool {
        self.administrators.contains(&user_id)
    }

    /// Add a new owner (runtime modification)
    pub fn add_owner(&mut self, user_id: UserId) {
        self.owners.insert(user_id);
    }

    /// Remove an owner (runtime modification)
    pub fn remove_owner(&mut self, user_id: UserId) {
        self.owners.remove(&user_id);
    }

    /// Add a new administrator (runtime modification)
    pub fn add_administrator(&mut self, user_id: UserId) {
        self.administrators.insert(user_id);
    }

    /// Remove an administrator (runtime modification)
    pub fn remove_administrator(&mut self, user_id: UserId) {
        self.administrators.remove(&user_id);
    }
} 