//! Discord API client with authentication and connection management

use anyhow::{Result, bail};
use poise::serenity_prelude::{self as serenity, GatewayIntents, ShardManager, ChannelId, GuildId};
use std::sync::Arc;
use tracing::{info, warn, error, debug};
use tokio::sync::RwLock;
use tgraph_config::settings::DiscordConfig;

/// Discord client status
#[derive(Debug, Clone, PartialEq)]
pub enum ClientStatus {
    /// Client is not connected
    Disconnected,
    /// Client is connecting
    Connecting,
    /// Client is connected and ready
    Connected,
    /// Client is reconnecting after a failure
    Reconnecting,
    /// Client has failed and cannot reconnect
    Failed(String),
}

/// Permission check results for a Discord channel
#[derive(Debug, Clone, PartialEq)]
pub struct ChannelPermissions {
    /// Whether the bot can send messages in the channel
    pub can_send_messages: bool,
    /// Whether the bot can attach files in the channel
    pub can_attach_files: bool,
    /// Whether the bot can read message history in the channel
    pub can_read_message_history: bool,
    /// Whether the bot can embed links in messages
    pub can_embed_links: bool,
    /// Whether the bot can add reactions to messages
    pub can_add_reactions: bool,
    /// Guild ID if this is a guild channel
    pub guild_id: Option<GuildId>,
    /// Error message if permissions couldn't be checked
    pub error: Option<String>,
}

impl ChannelPermissions {
    /// Check if the bot has the minimum required permissions for posting graphs
    pub fn can_post_graphs(&self) -> bool {
        self.can_send_messages && self.can_attach_files
    }

    /// Check if the bot has all essential permissions
    pub fn has_essential_permissions(&self) -> bool {
        self.can_send_messages 
            && self.can_attach_files 
            && self.can_embed_links 
            && self.can_read_message_history
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        if let Some(error) = &self.error {
            return format!("❌ Error checking permissions: {}", error);
        }

        if self.has_essential_permissions() {
            "✅ All essential permissions available".to_string()
        } else {
            let mut missing = Vec::new();
            if !self.can_send_messages { missing.push("Send Messages"); }
            if !self.can_attach_files { missing.push("Attach Files"); }
            if !self.can_embed_links { missing.push("Embed Links"); }
            if !self.can_read_message_history { missing.push("Read Message History"); }
            
            format!("⚠️ Missing permissions: {}", missing.join(", "))
        }
    }
}

/// Discord client wrapper with enhanced connection management
pub struct DiscordClient {
    /// The underlying serenity client
    client: Option<serenity::Client>,
    /// Shard manager for connection control
    shard_manager: Option<Arc<ShardManager>>,
    /// Current connection status
    status: Arc<RwLock<ClientStatus>>,
    /// Discord configuration
    config: DiscordConfig,
}

impl DiscordClient {
    /// Create a new Discord client with the given configuration
    pub fn new(config: DiscordConfig) -> Self {
        Self {
            client: None,
            shard_manager: None,
            status: Arc::new(RwLock::new(ClientStatus::Disconnected)),
            config,
        }
    }

    /// Initialize the Discord client with authentication
    pub async fn initialize(&mut self) -> Result<()> {
        info!("Initializing Discord client");
        
        // Update status to connecting
        {
            let mut status = self.status.write().await;
            *status = ClientStatus::Connecting;
        }

        // Validate the token format before attempting connection
        if let Err(e) = self.validate_token() {
            let error_msg = format!("Token validation failed: {}", e);
            error!("{}", error_msg);
            
            let mut status = self.status.write().await;
            *status = ClientStatus::Failed(error_msg);
            return Err(e);
        }

        // Configure Discord intents
        let intents = self.get_required_intents();
        debug!("Configured Discord intents: {:?}", intents);

        // Create the serenity client
        match serenity::ClientBuilder::new(&self.config.token, intents)
            .await
        {
            Ok(client) => {
                info!("Discord client created successfully");
                
                // Store shard manager for connection control
                self.shard_manager = Some(client.shard_manager.clone());
                self.client = Some(client);
                
                // Update status to connected
                let mut status = self.status.write().await;
                *status = ClientStatus::Connected;
                
                Ok(())
            }
            Err(e) => {
                let error_msg = format!("Failed to create Discord client: {}", e);
                error!("{}", error_msg);
                
                let mut status = self.status.write().await;
                *status = ClientStatus::Failed(error_msg.clone());
                
                Err(anyhow::anyhow!(error_msg))
            }
        }
    }

    /// Check channel permissions for the bot
    /// This function handles modern Discord's Application/Integration permission system
    pub async fn check_channel_permissions(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
    ) -> ChannelPermissions {
        debug!("Checking permissions for channel {}", channel_id);

        // First, try to get the channel information
        let channel = match channel_id.to_channel(http).await {
            Ok(channel) => channel,
            Err(e) => {
                let error_msg = format!("Failed to fetch channel {}: {}", channel_id, e);
                warn!("{}", error_msg);
                return ChannelPermissions {
                    can_send_messages: false,
                    can_attach_files: false,
                    can_read_message_history: false,
                    can_embed_links: false,
                    can_add_reactions: false,
                    guild_id: None,
                    error: Some(error_msg),
                };
            }
        };

        match channel {
            serenity::Channel::Guild(guild_channel) => {
                self.check_guild_channel_permissions(http, &guild_channel).await
            }
            serenity::Channel::Private(_) => {
                // DM channels have full permissions for bots
                debug!("Channel {} is a DM channel - full permissions available", channel_id);
                ChannelPermissions {
                    can_send_messages: true,
                    can_attach_files: true,
                    can_read_message_history: true,
                    can_embed_links: true,
                    can_add_reactions: true,
                    guild_id: None,
                    error: None,
                }
            }
            _ => {
                let error_msg = "Unsupported channel type".to_string();
                warn!("{} for channel {}", error_msg, channel_id);
                ChannelPermissions {
                    can_send_messages: false,
                    can_attach_files: false,
                    can_read_message_history: false,
                    can_embed_links: false,
                    can_add_reactions: false,
                    guild_id: None,
                    error: Some(error_msg),
                }
            }
        }
    }

    /// Check permissions for a guild channel specifically
    async fn check_guild_channel_permissions(
        &self,
        http: &serenity::Http,
        guild_channel: &serenity::GuildChannel,
    ) -> ChannelPermissions {
        let guild_id = guild_channel.guild_id;
        let channel_id = guild_channel.id;

        debug!("Checking guild channel permissions for channel {} in guild {}", channel_id, guild_id);

        // Get the current user (bot) information
        let current_user = match http.get_current_user().await {
            Ok(user) => user,
            Err(e) => {
                let error_msg = format!("Failed to get current user: {}", e);
                error!("{}", error_msg);
                return ChannelPermissions {
                    can_send_messages: false,
                    can_attach_files: false,
                    can_read_message_history: false,
                    can_embed_links: false,
                    can_add_reactions: false,
                    guild_id: Some(guild_id),
                    error: Some(error_msg),
                };
            }
        };

        // First, get the guild to calculate permissions properly
        let guild = match http.get_guild(guild_id).await {
            Ok(guild) => guild,
            Err(e) => {
                let error_msg = format!("Failed to get guild {} for permission calculation: {}", guild_id, e);
                warn!("{}", error_msg);
                return ChannelPermissions {
                    can_send_messages: false,
                    can_attach_files: false,
                    can_read_message_history: false,
                    can_embed_links: false,
                    can_add_reactions: false,
                    guild_id: Some(guild_id),
                    error: Some(error_msg),
                };
            }
        };

        // Get the bot's member object from the guild to calculate permissions
        let bot_member = match http.get_member(guild_id, current_user.id).await {
            Ok(member) => member,
            Err(e) => {
                let error_msg = format!("Failed to get bot member from guild {}: {}", guild_id, e);
                warn!("{}", error_msg);
                return ChannelPermissions {
                    can_send_messages: false,
                    can_attach_files: false,
                    can_read_message_history: false,
                    can_embed_links: false,
                    can_add_reactions: false,
                    guild_id: Some(guild_id),
                    error: Some(error_msg),
                };
            }
        };

        // Calculate permissions for the bot in this specific channel
        // This properly handles role permissions, channel overwrites, and modern integrations
        let permissions = guild.user_permissions_in(guild_channel, &bot_member);

        debug!("Calculated permissions for channel {}: {:?}", channel_id, permissions);

        // Check specific permissions needed for posting graphs and messages
        let can_send_messages = permissions.send_messages();
        let can_attach_files = permissions.attach_files();
        let can_read_message_history = permissions.read_message_history();
        let can_embed_links = permissions.embed_links();
        let can_add_reactions = permissions.add_reactions();

        debug!(
            "Permission breakdown for channel {}: send_messages={}, attach_files={}, read_history={}, embed_links={}, add_reactions={}",
            channel_id, can_send_messages, can_attach_files, can_read_message_history, can_embed_links, can_add_reactions
        );

        ChannelPermissions {
            can_send_messages,
            can_attach_files,
            can_read_message_history,
            can_embed_links,
            can_add_reactions,
            guild_id: Some(guild_id),
            error: None,
        }
    }

    /// Convenience method to check if the bot can post graphs to a specific channel
    pub async fn can_post_to_channel(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
    ) -> Result<bool> {
        let permissions = self.check_channel_permissions(http, channel_id).await;
        
        if let Some(error) = permissions.error {
            bail!("Permission check failed: {}", error);
        }

        Ok(permissions.can_post_graphs())
    }

    /// Get the current connection status
    pub async fn status(&self) -> ClientStatus {
        self.status.read().await.clone()
    }

    /// Check if the client is connected and ready
    pub async fn is_connected(&self) -> bool {
        matches!(*self.status.read().await, ClientStatus::Connected)
    }

    /// Get a reference to the underlying serenity client
    pub fn client(&self) -> Option<&serenity::Client> {
        self.client.as_ref()
    }

    /// Get the shard manager for connection control
    pub fn shard_manager(&self) -> Option<Arc<ShardManager>> {
        self.shard_manager.clone()
    }

    /// Validate the Discord bot token format
    fn validate_token(&self) -> Result<()> {
        let token = &self.config.token;
        
        if token.is_empty() {
            bail!("Discord token cannot be empty");
        }

        // Basic Discord bot token validation
        // Bot tokens should start with a specific pattern and have a minimum length
        if token.len() < 50 {
            bail!("Discord token appears to be too short (minimum 50 characters expected)");
        }

        // Discord bot tokens typically contain dots
        if !token.contains('.') {
            warn!("Discord token format may be invalid (missing dots)");
        }

        debug!("Discord token validation passed");
        Ok(())
    }

    /// Get the required Discord gateway intents
    fn get_required_intents(&self) -> GatewayIntents {
        // Configure intents based on what the bot needs to do
        GatewayIntents::GUILD_MESSAGES 
            | GatewayIntents::MESSAGE_CONTENT 
            | GatewayIntents::GUILDS
    }

    /// Attempt to reconnect the client
    pub async fn reconnect(&mut self) -> Result<()> {
        info!("Attempting to reconnect Discord client");
        
        // Update status to reconnecting
        {
            let mut status = self.status.write().await;
            *status = ClientStatus::Reconnecting;
        }

        // Shutdown existing client if present
        if let Some(shard_manager) = &self.shard_manager {
            warn!("Shutting down existing client for reconnection");
            shard_manager.shutdown_all().await;
        }

        // Clear existing client state
        self.client = None;
        self.shard_manager = None;

        // Re-initialize the client
        self.initialize().await
    }

    /// Gracefully shutdown the Discord client
    pub async fn shutdown(&mut self) -> Result<()> {
        info!("Shutting down Discord client");

        if let Some(shard_manager) = &self.shard_manager {
            shard_manager.shutdown_all().await;
            info!("Discord client shutdown completed");
        }

        // Update status
        let mut status = self.status.write().await;
        *status = ClientStatus::Disconnected;

        // Clear client state
        self.client = None;
        self.shard_manager = None;

        Ok(())
    }

    /// Get connection health information
    pub async fn health_info(&self) -> DiscordHealthInfo {
        let status = self.status.read().await.clone();
        
        DiscordHealthInfo {
            status,
            has_client: self.client.is_some(),
            has_shard_manager: self.shard_manager.is_some(),
            token_configured: !self.config.token.is_empty(),
        }
    }
}

/// Health information for the Discord client
#[derive(Debug)]
pub struct DiscordHealthInfo {
    pub status: ClientStatus,
    pub has_client: bool,
    pub has_shard_manager: bool,
    pub token_configured: bool,
}

impl DiscordHealthInfo {
    /// Check if the client is healthy
    pub fn is_healthy(&self) -> bool {
        self.status == ClientStatus::Connected 
            && self.has_client 
            && self.has_shard_manager 
            && self.token_configured
    }
}

/// Discord authentication error types
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token format: {0}")]
    InvalidToken(String),
    
    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),
    
    #[error("Connection timeout")]
    Timeout,
    
    #[error("Network error: {0}")]
    Network(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tgraph_config::settings::DiscordConfig;

    fn create_test_config(token: &str) -> DiscordConfig {
        DiscordConfig {
            token: token.to_string(),
            channels: vec![],
            max_concurrent_requests: 10,
            request_timeout_seconds: 30,
            owner_ids: vec![],
            admin_ids: vec![],
            admin_role_ids: vec![],
            moderator_role_ids: vec![],
        }
    }

    #[test]
    fn test_client_creation() {
        let config = create_test_config("test.token.here");
        let client = DiscordClient::new(config);
        
        assert!(client.client.is_none());
        assert!(client.shard_manager.is_none());
    }

    #[tokio::test]
    async fn test_initial_status() {
        let config = create_test_config("test.token.here");
        let client = DiscordClient::new(config);
        
        assert_eq!(client.status().await, ClientStatus::Disconnected);
        assert!(!client.is_connected().await);
    }

    #[test]
    fn test_token_validation_empty() {
        let config = create_test_config("");
        let client = DiscordClient::new(config);
        
        let result = client.validate_token();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("cannot be empty"));
    }

    #[test]
    fn test_token_validation_too_short() {
        let config = create_test_config("short");
        let client = DiscordClient::new(config);
        
        let result = client.validate_token();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("too short"));
    }

    #[test]
    fn test_token_validation_valid_format() {
        let config = create_test_config("MTExNzU4MzQ4NzEyNzY1NjQxNw.GZKbkF.1234567890123456789012345678901234567890");
        let client = DiscordClient::new(config);
        
        let result = client.validate_token();
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_health_info() {
        let config = create_test_config("test.token.here");
        let client = DiscordClient::new(config);
        
        let health = client.health_info().await;
        assert_eq!(health.status, ClientStatus::Disconnected);
        assert!(!health.has_client);
        assert!(!health.has_shard_manager);
        assert!(health.token_configured); // "test.token.here" is not empty
        assert!(!health.is_healthy());
    }

    #[test]
    fn test_channel_permissions_can_post_graphs() {
        let permissions = ChannelPermissions {
            can_send_messages: true,
            can_attach_files: true,
            can_read_message_history: true,
            can_embed_links: true,
            can_add_reactions: true,
            guild_id: None,
            error: None,
        };
        
        assert!(permissions.can_post_graphs());
        assert!(permissions.has_essential_permissions());
    }

    #[test]
    fn test_channel_permissions_missing_attach_files() {
        let permissions = ChannelPermissions {
            can_send_messages: true,
            can_attach_files: false, // Missing this permission
            can_read_message_history: true,
            can_embed_links: true,
            can_add_reactions: true,
            guild_id: None,
            error: None,
        };
        
        assert!(!permissions.can_post_graphs());
        assert!(!permissions.has_essential_permissions());
        assert!(permissions.status_message().contains("Attach Files"));
    }

    #[test]
    fn test_channel_permissions_error_status() {
        let permissions = ChannelPermissions {
            can_send_messages: false,
            can_attach_files: false,
            can_read_message_history: false,
            can_embed_links: false,
            can_add_reactions: false,
            guild_id: None,
            error: Some("Channel not found".to_string()),
        };
        
        assert!(!permissions.can_post_graphs());
        assert!(permissions.status_message().contains("Error checking permissions"));
    }
} 