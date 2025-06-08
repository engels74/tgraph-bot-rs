//! Discord API client with authentication and connection management

use anyhow::{Result, bail};
use poise::serenity_prelude::{self as serenity, GatewayIntents, ShardManager, ChannelId, GuildId, CreateAttachment, CreateEmbed, CreateMessage, Colour, Timestamp};
use std::sync::Arc;
use std::path::Path;
use std::fs;
use tracing::{info, warn, error, debug};
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tgraph_config::settings::DiscordConfig;
use chrono::{DateTime, Utc};

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

/// File attachment configuration for Discord messages
#[derive(Debug, Clone)]
pub struct GraphAttachment {
    /// File name to display in Discord
    pub filename: String,
    /// PNG image data
    pub data: Vec<u8>,
    /// Optional description for the attachment
    pub description: Option<String>,
}

impl GraphAttachment {
    /// Create a new GraphAttachment from a file path
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // Validate file exists
        if !path.exists() {
            bail!("File does not exist: {}", path.display());
        }

        // Validate file extension
        if !Self::is_png_file(path) {
            bail!("File must have .png extension: {}", path.display());
        }

        // Read file data
        let data = fs::read(path)
            .map_err(|e| anyhow::anyhow!("Failed to read file {}: {}", path.display(), e))?;

        // Validate file size (Discord limit is 25MB for bots)
        Self::validate_file_size(&data)?;

        // Validate PNG magic bytes
        Self::validate_png_format(&data)?;

        let filename = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("Invalid file name: {}", path.display()))?
            .to_string_lossy()
            .to_string();

        Ok(Self {
            filename,
            data,
            description: None,
        })
    }

    /// Create a new GraphAttachment from raw PNG data
    pub fn from_data(filename: String, data: Vec<u8>) -> Result<Self> {
        // Validate file size
        Self::validate_file_size(&data)?;

        // Validate PNG format
        Self::validate_png_format(&data)?;

        // Ensure filename has .png extension
        let filename = if filename.to_lowercase().ends_with(".png") {
            filename
        } else {
            format!("{}.png", filename)
        };

        Ok(Self {
            filename,
            data,
            description: None,
        })
    }

    /// Set a description for the attachment
    pub fn with_description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Convert to Discord CreateAttachment
    pub fn to_discord_attachment(&self) -> CreateAttachment {
        let mut attachment = CreateAttachment::bytes(self.data.clone(), &self.filename);
        
        if let Some(description) = &self.description {
            attachment = attachment.description(description);
        }

        attachment
    }

    /// Get file size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }

    /// Get human-readable file size
    pub fn size_human(&self) -> String {
        let size = self.data.len() as f64;
        
        if size < 1024.0 {
            format!("{} B", size)
        } else if size < 1024.0 * 1024.0 {
            format!("{:.1} KB", size / 1024.0)
        } else {
            format!("{:.1} MB", size / (1024.0 * 1024.0))
        }
    }

    /// Check if file has PNG extension
    fn is_png_file(path: &Path) -> bool {
        path.extension()
            .and_then(|ext| ext.to_str())
            .map(|ext| ext.to_lowercase() == "png")
            .unwrap_or(false)
    }

    /// Validate file size against Discord limits
    fn validate_file_size(data: &[u8]) -> Result<()> {
        const MAX_FILE_SIZE: usize = 25 * 1024 * 1024; // 25MB for bots
        
        if data.len() > MAX_FILE_SIZE {
            bail!(
                "File size ({:.1} MB) exceeds Discord's limit of 25 MB", 
                data.len() as f64 / (1024.0 * 1024.0)
            );
        }
        
        if data.is_empty() {
            bail!("File is empty");
        }
        
        Ok(())
    }

    /// Validate PNG file format by checking magic bytes
    fn validate_png_format(data: &[u8]) -> Result<()> {
        const PNG_MAGIC: &[u8] = &[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
        
        if data.len() < PNG_MAGIC.len() {
            bail!("File too small to be a valid PNG");
        }
        
        if !data.starts_with(PNG_MAGIC) {
            bail!("File does not contain valid PNG magic bytes");
        }
        
        Ok(())
    }
}

/// File attachment manager for Discord operations
pub struct AttachmentManager {
    /// Maximum allowed file size in bytes
    max_file_size: usize,
}

impl AttachmentManager {
    /// Create a new AttachmentManager with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 25 * 1024 * 1024, // 25MB default for Discord bots
        }
    }

    /// Create an AttachmentManager with custom size limit
    pub fn with_max_size(max_size: usize) -> Self {
        Self {
            max_file_size: max_size,
        }
    }

    /// Prepare a graph file for Discord attachment
    pub fn prepare_graph_attachment<P: AsRef<Path>>(&self, path: P) -> Result<GraphAttachment> {
        GraphAttachment::from_file(path)
    }

    /// Prepare multiple graph files for Discord attachment
    pub fn prepare_multiple_attachments<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<GraphAttachment>> {
        let mut attachments = Vec::new();
        
        for path in paths {
            let attachment = self.prepare_graph_attachment(path)?;
            attachments.push(attachment);
        }
        
        // Validate total size
        let total_size: usize = attachments.iter().map(|a| a.size()).sum();
        if total_size > self.max_file_size {
            bail!(
                "Total attachment size ({:.1} MB) exceeds limit ({:.1} MB)",
                total_size as f64 / (1024.0 * 1024.0),
                self.max_file_size as f64 / (1024.0 * 1024.0)
            );
        }
        
        Ok(attachments)
    }

    /// Create attachment from raw PNG data with validation
    pub fn create_from_data(&self, filename: String, data: Vec<u8>) -> Result<GraphAttachment> {
        if data.len() > self.max_file_size {
            bail!(
                "Data size ({:.1} MB) exceeds limit ({:.1} MB)",
                data.len() as f64 / (1024.0 * 1024.0),
                self.max_file_size as f64 / (1024.0 * 1024.0)
            );
        }
        
        GraphAttachment::from_data(filename, data)
    }
}

impl Default for AttachmentManager {
    fn default() -> Self {
        Self::new()
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

    /// Create an attachment manager for this client
    pub fn attachment_manager(&self) -> AttachmentManager {
        AttachmentManager::new()
    }

    /// Create an attachment manager with custom size limit
    pub fn attachment_manager_with_limit(&self, max_size: usize) -> AttachmentManager {
        AttachmentManager::with_max_size(max_size)
    }

    /// Prepare a graph file for Discord attachment with validation
    pub fn prepare_graph_file<P: AsRef<Path>>(&self, path: P) -> Result<GraphAttachment> {
        let attachment_manager = self.attachment_manager();
        attachment_manager.prepare_graph_attachment(path)
    }

    /// Prepare multiple graph files for Discord attachment
    pub fn prepare_multiple_graphs<P: AsRef<Path>>(&self, paths: &[P]) -> Result<Vec<GraphAttachment>> {
        let attachment_manager = self.attachment_manager();
        attachment_manager.prepare_multiple_attachments(paths)
    }

    /// Create a graph attachment from raw PNG data
    pub fn create_graph_from_data(&self, filename: String, data: Vec<u8>) -> Result<GraphAttachment> {
        let attachment_manager = self.attachment_manager();
        attachment_manager.create_from_data(filename, data)
    }

    /// Validate and prepare an attachment for posting
    /// This combines permission checking with file preparation
    pub async fn validate_and_prepare_attachment<P: AsRef<Path>>(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
        path: P,
    ) -> Result<GraphAttachment> {
        // First check if we can post to the channel
        if !self.can_post_to_channel(http, channel_id).await? {
            bail!("Bot does not have permission to post attachments to channel {}", channel_id);
        }

        // Then prepare the attachment
        self.prepare_graph_file(path)
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

/// Configuration for retry logic when posting messages
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (not including the initial attempt)
    pub max_retries: u32,
    /// Base delay for exponential backoff in milliseconds
    pub base_delay_ms: u64,
    /// Maximum delay cap in milliseconds
    pub max_delay_ms: u64,
    /// Jitter factor for randomizing delays (0.0 to 1.0)
    pub jitter_factor: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay_ms: 1000,  // 1 second
            max_delay_ms: 30000,  // 30 seconds
            jitter_factor: 0.1,   // 10% jitter
        }
    }
}

impl RetryConfig {
    /// Create a new retry configuration
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay_ms,
            max_delay_ms,
            jitter_factor: 0.1,
        }
    }

    /// Set jitter factor for randomizing delays
    pub fn with_jitter(mut self, jitter_factor: f64) -> Self {
        self.jitter_factor = jitter_factor.clamp(0.0, 1.0);
        self
    }

    /// Calculate delay for retry attempt with exponential backoff and jitter
    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.base_delay_ms as f64;
        let exponential_delay = base_delay * (2.0_f64.powi(attempt as i32));
        
        // Apply max delay cap
        let capped_delay = exponential_delay.min(self.max_delay_ms as f64);
        
        // Add jitter to avoid thundering herd - simple deterministic approach
        let jitter = ((attempt as f64 * 17.0) % 1.0 - 0.5) * 2.0 * self.jitter_factor;
        let final_delay = capped_delay * (1.0 + jitter);
        
        Duration::from_millis(final_delay.max(0.0) as u64)
    }
}

/// Result of a message posting operation
#[derive(Debug, Clone)]
pub struct PostResult {
    /// Whether the message was posted successfully
    pub success: bool,
    /// The message ID if successful
    pub message_id: Option<serenity::MessageId>,
    /// Number of retry attempts made
    pub attempts: u32,
    /// Total time taken for the operation
    pub duration: Duration,
    /// Error message if unsuccessful
    pub error: Option<String>,
    /// Whether rate limit was encountered
    pub rate_limited: bool,
}

impl PostResult {
    /// Create a successful post result
    pub fn success(message_id: serenity::MessageId, attempts: u32, duration: Duration, rate_limited: bool) -> Self {
        Self {
            success: true,
            message_id: Some(message_id),
            attempts,
            duration,
            error: None,
            rate_limited,
        }
    }

    /// Create a failed post result
    pub fn failure(error: String, attempts: u32, duration: Duration, rate_limited: bool) -> Self {
        Self {
            success: false,
            message_id: None,
            attempts,
            duration,
            error: Some(error),
            rate_limited,
        }
    }

    /// Get a human-readable status message
    pub fn status_message(&self) -> String {
        if self.success {
            let rate_limit_msg = if self.rate_limited { " (overcame rate limits)" } else { "" };
            format!("✅ Message posted successfully after {} attempt(s) in {:.2}s{}", 
                   self.attempts, self.duration.as_secs_f64(), rate_limit_msg)
        } else {
            let error_msg = self.error.as_deref().unwrap_or("Unknown error");
            let rate_limit_msg = if self.rate_limited { " (rate limited)" } else { "" };
            format!("❌ Failed to post message after {} attempt(s) in {:.2}s: {}{}", 
                   self.attempts, self.duration.as_secs_f64(), error_msg, rate_limit_msg)
        }
    }
}

impl DiscordClient {
    /// Post a message to a Discord channel with retry logic for rate limits
    pub async fn post_message(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
        message: CreateMessage,
        retry_config: Option<RetryConfig>,
    ) -> Result<PostResult> {
        let retry_config = retry_config.unwrap_or_default();
        let start_time = std::time::Instant::now();
        let mut attempts = 0;
        let mut rate_limited = false;

        info!("Attempting to post message to channel {}", channel_id);

        for attempt in 0..=retry_config.max_retries {
            attempts += 1;
            
            debug!("Message post attempt {} of {} to channel {}", 
                   attempt + 1, retry_config.max_retries + 1, channel_id);

            match channel_id.send_message(http, message.clone()).await {
                Ok(sent_message) => {
                    let duration = start_time.elapsed();
                    info!("Message posted successfully to channel {} after {} attempt(s) in {:.2}s", 
                          channel_id, attempts, duration.as_secs_f64());
                    
                    return Ok(PostResult::success(sent_message.id, attempts, duration, rate_limited));
                }
                Err(serenity_error) => {
                    let is_rate_limit = Self::is_rate_limit_error(&serenity_error);
                    if is_rate_limit {
                        rate_limited = true;
                    }

                    // Log the error with appropriate level
                    if is_rate_limit {
                        warn!("Rate limit encountered on attempt {} for channel {}: {}", 
                              attempt + 1, channel_id, serenity_error);
                    } else {
                        error!("Error posting message on attempt {} to channel {}: {}", 
                               attempt + 1, channel_id, serenity_error);
                    }

                    // If this is not the last attempt and it's a retryable error, wait and retry
                    if attempt < retry_config.max_retries && Self::is_retryable_error(&serenity_error) {
                        let delay = retry_config.calculate_delay(attempt);
                        
                        info!("Retrying message post to channel {} in {:.2}s (attempt {} of {})", 
                              channel_id, delay.as_secs_f64(), attempt + 2, retry_config.max_retries + 1);
                        
                        sleep(delay).await;
                        continue;
                    } else {
                        // This was the last attempt or non-retryable error
                        let duration = start_time.elapsed();
                        let error_msg = format!("Failed to post message: {}", serenity_error);
                        
                        error!("Giving up posting message to channel {} after {} attempt(s): {}", 
                               channel_id, attempts, serenity_error);
                        
                        return Ok(PostResult::failure(error_msg, attempts, duration, rate_limited));
                    }
                }
            }
        }

        // This should never be reached due to the loop logic, but just in case
        let duration = start_time.elapsed();
        Ok(PostResult::failure(
            "Maximum retry attempts exceeded".to_string(),
            attempts,
            duration,
            rate_limited,
        ))
    }

    /// Post a message with graph attachment using retry logic
    pub async fn post_graph_message(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
        message_builder: DiscordMessageBuilder,
        attachment: GraphAttachment,
        retry_config: Option<RetryConfig>,
    ) -> Result<PostResult> {
        // First check permissions
        let permissions = self.check_channel_permissions(http, channel_id).await;
        if !permissions.can_post_graphs() {
            let error_msg = format!("Insufficient permissions to post graphs: {}", permissions.status_message());
            warn!("{}", error_msg);
            return Ok(PostResult::failure(
                error_msg,
                1,
                Duration::from_millis(0),
                false,
            ));
        }

        info!("Posting graph message to channel {} with attachment: {} ({})", 
              channel_id, attachment.filename, attachment.size_human());

        // Build message with attachment
        let message = message_builder.build_with_attachments(vec![attachment]);

        // Post with retry logic
        self.post_message(http, channel_id, message, retry_config).await
    }

    /// Post a graph to Discord with complete validation and retry logic
    pub async fn post_graph<P: AsRef<Path>>(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
        graph_path: P,
        title: Option<&str>,
        description: Option<&str>,
        retry_config: Option<RetryConfig>,
    ) -> Result<PostResult> {
        let start_time = std::time::Instant::now();

        // Validate and prepare attachment
        debug!("Preparing graph attachment from path: {}", graph_path.as_ref().display());
        let attachment = match self.validate_and_prepare_attachment(http, channel_id, graph_path).await {
            Ok(attachment) => attachment,
            Err(e) => {
                let error_msg = format!("Failed to prepare graph attachment: {}", e);
                error!("{}", error_msg);
                return Ok(PostResult::failure(
                    error_msg,
                    1,
                    start_time.elapsed(),
                    false,
                ));
            }
        };

        // Build message
        let mut message_builder = DiscordMessageBuilder::graph();
        
        if let Some(title) = title {
            message_builder = message_builder.title(title);
        }
        
        if let Some(description) = description {
            message_builder = message_builder.description(description);
        }

        // Add metadata about the graph
        message_builder = message_builder
            .add_field("File Size", attachment.size_human(), true)
            .add_field("Format", "PNG", true);

        // Post the graph
        self.post_graph_message(http, channel_id, message_builder, attachment, retry_config).await
    }

    /// Post a simple text message with retry logic
    pub async fn post_simple_message(
        &self,
        http: &serenity::Http,
        channel_id: ChannelId,
        content: &str,
        message_type: Option<MessageType>,
        retry_config: Option<RetryConfig>,
    ) -> Result<PostResult> {
        // Check basic send message permission
        let permissions = self.check_channel_permissions(http, channel_id).await;
        if !permissions.can_send_messages {
            let error_msg = format!("No permission to send messages: {}", permissions.status_message());
            warn!("{}", error_msg);
            return Ok(PostResult::failure(
                error_msg,
                1,
                Duration::from_millis(0),
                false,
            ));
        }

        let message_type = message_type.unwrap_or(MessageType::Info);
        let message_builder = DiscordMessageBuilder::new(message_type)
            .content(content);

        let message = message_builder.build();
        self.post_message(http, channel_id, message, retry_config).await
    }

    /// Check if an error is due to rate limiting
    fn is_rate_limit_error(error: &serenity::Error) -> bool {
        // Simple string-based check for rate limit error
        let error_str = error.to_string().to_lowercase();
        error_str.contains("rate limit") || error_str.contains("429")
    }

    /// Check if an error is retryable
    fn is_retryable_error(error: &serenity::Error) -> bool {
        // For now, retry on any HTTP error that might be temporary
        match error {
            serenity::Error::Http(_) => {
                let error_str = error.to_string().to_lowercase();
                // Retry on rate limits, server errors, or network issues
                error_str.contains("rate limit") 
                    || error_str.contains("429")
                    || error_str.contains("500")
                    || error_str.contains("502")
                    || error_str.contains("503")
                    || error_str.contains("504")
                    || error_str.contains("timeout")
                    || error_str.contains("connection")
            }
            // Gateway connection issues might be retryable
            serenity::Error::Gateway(_) => true,
            _ => false,
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

/// Message type for color coding and formatting
#[derive(Debug, Clone, PartialEq)]
pub enum MessageType {
    /// Success message (green)
    Success,
    /// Information message (blue)
    Info,
    /// Warning message (yellow/orange)
    Warning,
    /// Error message (red)
    Error,
    /// Graph result message (purple)
    Graph,
    /// System message (gray)
    System,
}

impl MessageType {
    /// Get the Discord color for this message type
    pub fn color(&self) -> Colour {
        match self {
            MessageType::Success => Colour::from_rgb(34, 197, 94),   // Green
            MessageType::Info => Colour::from_rgb(59, 130, 246),     // Blue
            MessageType::Warning => Colour::from_rgb(245, 158, 11),  // Orange
            MessageType::Error => Colour::from_rgb(239, 68, 68),     // Red
            MessageType::Graph => Colour::from_rgb(147, 51, 234),    // Purple
            MessageType::System => Colour::from_rgb(107, 114, 128),  // Gray
        }
    }

    /// Get an emoji prefix for this message type
    pub fn emoji(&self) -> &'static str {
        match self {
            MessageType::Success => "✅",
            MessageType::Info => "ℹ️",
            MessageType::Warning => "⚠️",
            MessageType::Error => "❌",
            MessageType::Graph => "📊",
            MessageType::System => "🤖",
        }
    }
}

/// Metadata field for embeds
#[derive(Debug, Clone)]
pub struct MetadataField {
    /// Field name
    pub name: String,
    /// Field value
    pub value: String,
    /// Whether to display inline
    pub inline: bool,
}

impl MetadataField {
    /// Create a new metadata field
    pub fn new<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            inline: false,
        }
    }

    /// Create a new inline metadata field
    pub fn inline<N: Into<String>, V: Into<String>>(name: N, value: V) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
            inline: true,
        }
    }

    /// Set the inline property
    pub fn with_inline(mut self, inline: bool) -> Self {
        self.inline = inline;
        self
    }
}

/// Discord message builder with embed support
#[derive(Debug, Clone)]
pub struct DiscordMessageBuilder {
    /// Message content (outside embed)
    content: Option<String>,
    /// Embed title
    title: Option<String>,
    /// Embed description
    description: Option<String>,
    /// Message type for color coding
    message_type: MessageType,
    /// Metadata fields
    fields: Vec<MetadataField>,
    /// Author name for embed
    author: Option<String>,
    /// Author icon URL
    author_icon: Option<String>,
    /// Footer text
    footer: Option<String>,
    /// Footer icon URL
    footer_icon: Option<String>,
    /// Thumbnail URL
    thumbnail: Option<String>,
    /// Image URL (for main image)
    image: Option<String>,
    /// Custom timestamp
    timestamp: Option<DateTime<Utc>>,
    /// Whether to include generation timestamp in footer
    include_generation_time: bool,
}

impl DiscordMessageBuilder {
    /// Create a new message builder
    pub fn new(message_type: MessageType) -> Self {
        Self {
            content: None,
            title: None,
            description: None,
            message_type,
            fields: Vec::new(),
            author: None,
            author_icon: None,
            footer: None,
            footer_icon: None,
            thumbnail: None,
            image: None,
            timestamp: None,
            include_generation_time: true,
        }
    }

    /// Create a graph result message builder
    pub fn graph() -> Self {
        Self::new(MessageType::Graph)
    }

    /// Create a success message builder
    pub fn success() -> Self {
        Self::new(MessageType::Success)
    }

    /// Create an info message builder
    pub fn info() -> Self {
        Self::new(MessageType::Info)
    }

    /// Create a warning message builder
    pub fn warning() -> Self {
        Self::new(MessageType::Warning)
    }

    /// Create an error message builder
    pub fn error() -> Self {
        Self::new(MessageType::Error)
    }

    /// Create a system message builder
    pub fn system() -> Self {
        Self::new(MessageType::System)
    }

    /// Set the main message content (outside embed)
    pub fn content<S: Into<String>>(mut self, content: S) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set the embed title
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }

    /// Set the embed description
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a metadata field
    pub fn field(mut self, field: MetadataField) -> Self {
        self.fields.push(field);
        self
    }

    /// Add a simple metadata field
    pub fn add_field<N: Into<String>, V: Into<String>>(mut self, name: N, value: V, inline: bool) -> Self {
        self.fields.push(MetadataField {
            name: name.into(),
            value: value.into(),
            inline,
        });
        self
    }

    /// Set the author
    pub fn author<S: Into<String>>(mut self, name: S) -> Self {
        self.author = Some(name.into());
        self
    }

    /// Set the author with icon
    pub fn author_with_icon<S: Into<String>, U: Into<String>>(mut self, name: S, icon_url: U) -> Self {
        self.author = Some(name.into());
        self.author_icon = Some(icon_url.into());
        self
    }

    /// Set the footer
    pub fn footer<S: Into<String>>(mut self, text: S) -> Self {
        self.footer = Some(text.into());
        self
    }

    /// Set the footer with icon
    pub fn footer_with_icon<S: Into<String>, U: Into<String>>(mut self, text: S, icon_url: U) -> Self {
        self.footer = Some(text.into());
        self.footer_icon = Some(icon_url.into());
        self
    }

    /// Set the thumbnail URL
    pub fn thumbnail<S: Into<String>>(mut self, url: S) -> Self {
        self.thumbnail = Some(url.into());
        self
    }

    /// Set the image URL
    pub fn image<S: Into<String>>(mut self, url: S) -> Self {
        self.image = Some(url.into());
        self
    }

    /// Set a custom timestamp
    pub fn timestamp(mut self, timestamp: DateTime<Utc>) -> Self {
        self.timestamp = Some(timestamp);
        self
    }

    /// Control whether to include generation timestamp in footer
    pub fn include_generation_time(mut self, include: bool) -> Self {
        self.include_generation_time = include;
        self
    }

    /// Build the Discord CreateMessage
    pub fn build(self) -> CreateMessage {
        let mut message = CreateMessage::new();

        // Set content if provided
        if let Some(content) = self.content {
            message = message.content(content);
        }

        // Create embed
        let mut embed = CreateEmbed::new().color(self.message_type.color());

        // Set title with emoji prefix
        if let Some(title) = self.title {
            let title_with_emoji = format!("{} {}", self.message_type.emoji(), title);
            embed = embed.title(title_with_emoji);
        }

        // Set description
        if let Some(description) = self.description {
            embed = embed.description(description);
        }

        // Add fields
        for field in self.fields {
            embed = embed.field(field.name, field.value, field.inline);
        }

        // Set author
        if let Some(author) = self.author {
            if let Some(icon) = self.author_icon {
                embed = embed.author(serenity::CreateEmbedAuthor::new(author).icon_url(icon));
            } else {
                embed = embed.author(serenity::CreateEmbedAuthor::new(author));
            }
        }

        // Set footer with generation time
        let footer_text = if self.include_generation_time {
            let generation_time = Utc::now().format("%Y-%m-%d %H:%M:%S UTC");
            match self.footer {
                Some(footer) => format!("{} • Generated at {}", footer, generation_time),
                None => format!("Generated at {}", generation_time),
            }
        } else {
            self.footer.unwrap_or_default()
        };

        if !footer_text.is_empty() {
            if let Some(footer_icon) = self.footer_icon {
                embed = embed.footer(serenity::CreateEmbedFooter::new(footer_text).icon_url(footer_icon));
            } else {
                embed = embed.footer(serenity::CreateEmbedFooter::new(footer_text));
            }
        }

        // Set thumbnail
        if let Some(thumbnail) = self.thumbnail {
            embed = embed.thumbnail(thumbnail);
        }

        // Set image
        if let Some(image) = self.image {
            embed = embed.image(image);
        }

        // Set timestamp
        let timestamp = self.timestamp.unwrap_or_else(Utc::now);
        embed = embed.timestamp(Timestamp::from(timestamp));

        message.embed(embed)
    }

    /// Build a message with attachments
    pub fn build_with_attachments(self, attachments: Vec<GraphAttachment>) -> CreateMessage {
        let mut message = self.build();

        // Add attachments
        for attachment in attachments {
            message = message.add_file(attachment.to_discord_attachment());
        }

        message
    }
}

/// Message template for common message patterns
#[derive(Debug, Clone)]
pub struct MessageTemplate {
    /// Template name
    pub name: String,
    /// Default title pattern
    pub title_template: String,
    /// Default description pattern
    pub description_template: String,
    /// Default message type
    pub message_type: MessageType,
    /// Default fields
    pub default_fields: Vec<MetadataField>,
}

impl MessageTemplate {
    /// Create a new message template
    pub fn new<N: Into<String>, T: Into<String>, D: Into<String>>(
        name: N,
        title_template: T,
        description_template: D,
        message_type: MessageType,
    ) -> Self {
        Self {
            name: name.into(),
            title_template: title_template.into(),
            description_template: description_template.into(),
            message_type,
            default_fields: Vec::new(),
        }
    }

    /// Add a default field to the template
    pub fn with_field(mut self, field: MetadataField) -> Self {
        self.default_fields.push(field);
        self
    }

    /// Create a message builder from this template
    pub fn builder(&self) -> DiscordMessageBuilder {
        let mut builder = DiscordMessageBuilder::new(self.message_type.clone())
            .title(&self.title_template)
            .description(&self.description_template);

        // Add default fields
        for field in &self.default_fields {
            builder = builder.field(field.clone());
        }

        builder
    }

    /// Apply template with replacements
    pub fn apply_with_replacements(&self, replacements: &[(&str, &str)]) -> DiscordMessageBuilder {
        let mut title = self.title_template.clone();
        let mut description = self.description_template.clone();

        // Apply replacements
        for (placeholder, value) in replacements {
            title = title.replace(placeholder, value);
            description = description.replace(placeholder, value);
        }

        let mut builder = DiscordMessageBuilder::new(self.message_type.clone())
            .title(title)
            .description(description);

        // Add default fields with replacements applied
        for field in &self.default_fields {
            let mut field_name = field.name.clone();
            let mut field_value = field.value.clone();

            for (placeholder, value) in replacements {
                field_name = field_name.replace(placeholder, value);
                field_value = field_value.replace(placeholder, value);
            }

            builder = builder.add_field(field_name, field_value, field.inline);
        }

        builder
    }
}

/// Common message templates for graph operations
pub struct MessageTemplates;

impl MessageTemplates {
    /// Template for successful graph generation
    pub fn graph_success() -> MessageTemplate {
        MessageTemplate::new(
            "graph_success",
            "Graph Generated Successfully",
            "Your {graph_type} graph has been generated and is ready for viewing.",
            MessageType::Graph,
        )
        .with_field(MetadataField::inline("Status", "✅ Complete"))
        .with_field(MetadataField::inline("Type", "{graph_type}"))
    }

    /// Template for graph generation error
    pub fn graph_error() -> MessageTemplate {
        MessageTemplate::new(
            "graph_error",
            "Graph Generation Failed",
            "There was an error generating your {graph_type} graph: {error}",
            MessageType::Error,
        )
        .with_field(MetadataField::inline("Status", "❌ Failed"))
        .with_field(MetadataField::inline("Type", "{graph_type}"))
    }

    /// Template for command acknowledgment
    pub fn command_received() -> MessageTemplate {
        MessageTemplate::new(
            "command_received",
            "Command Received",
            "Processing your {command} command...",
            MessageType::Info,
        )
        .with_field(MetadataField::inline("Command", "{command}"))
        .with_field(MetadataField::inline("Status", "🔄 Processing"))
    }

    /// Template for permission errors
    pub fn permission_error() -> MessageTemplate {
        MessageTemplate::new(
            "permission_error",
            "Permission Denied",
            "I don't have the required permissions to {action} in this channel.",
            MessageType::Error,
        )
        .with_field(MetadataField::inline("Required", "{permissions}"))
        .with_field(MetadataField::inline("Action", "{action}"))
    }
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

    #[test]
    fn test_graph_attachment_from_data_valid_png() {
        // Valid PNG magic bytes
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, // PNG magic bytes
            0x00, 0x00, 0x00, 0x0D, // Additional minimal PNG data
            0x49, 0x48, 0x44, 0x52, // IHDR chunk
        ];

        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data.clone());
        assert!(attachment.is_ok());

        let attachment = attachment.unwrap();
        assert_eq!(attachment.filename, "test.png");
        assert_eq!(attachment.data, png_data);
        assert_eq!(attachment.description, None);
    }

    #[test]
    fn test_graph_attachment_from_data_auto_add_extension() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52,
        ];

        let attachment = GraphAttachment::from_data("test".to_string(), png_data);
        assert!(attachment.is_ok());

        let attachment = attachment.unwrap();
        assert_eq!(attachment.filename, "test.png");
    }

    #[test]
    fn test_graph_attachment_from_data_invalid_magic_bytes() {
        let invalid_data = vec![0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08]; // Invalid PNG magic bytes

        let attachment = GraphAttachment::from_data("test.png".to_string(), invalid_data);
        assert!(attachment.is_err());
        assert!(attachment.unwrap_err().to_string().contains("PNG magic bytes"));
    }

    #[test]
    fn test_graph_attachment_from_data_empty_file() {
        let empty_data = vec![];

        let attachment = GraphAttachment::from_data("test.png".to_string(), empty_data);
        assert!(attachment.is_err());
        assert!(attachment.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_graph_attachment_with_description() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52,
        ];

        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data)
            .unwrap()
            .with_description("Test graph");

        assert_eq!(attachment.description, Some("Test graph".to_string()));
    }

    #[test]
    fn test_graph_attachment_size_human() {
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52,
        ];

        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data.clone()).unwrap();
        
        // Should be in bytes since it's small
        assert_eq!(attachment.size_human(), format!("{} B", png_data.len()));
        assert_eq!(attachment.size(), png_data.len());
    }

    #[test]
    fn test_graph_attachment_size_kb() {
        let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic bytes
        png_data.resize(2048, 0); // Make it 2KB

        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data).unwrap();
        assert_eq!(attachment.size_human(), "2.0 KB");
    }

    #[test] 
    fn test_attachment_manager_new() {
        let manager = AttachmentManager::new();
        assert_eq!(manager.max_file_size, 25 * 1024 * 1024); // 25MB
    }

    #[test]
    fn test_attachment_manager_with_max_size() {
        let manager = AttachmentManager::with_max_size(1024 * 1024); // 1MB
        assert_eq!(manager.max_file_size, 1024 * 1024);
    }

    #[test]
    fn test_attachment_manager_create_from_data_exceeds_limit() {
        let manager = AttachmentManager::with_max_size(10); // Very small limit
        let mut png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]; // PNG magic bytes
        png_data.extend(vec![0x00; 20]); // Add more bytes to exceed the limit

        let result = manager.create_from_data("test.png".to_string(), png_data);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("exceeds limit"));
    }

    #[test] 
    fn test_attachment_manager_default() {
        let manager: AttachmentManager = Default::default();
        assert_eq!(manager.max_file_size, 25 * 1024 * 1024); // 25MB
    }

    #[test]
    fn test_discord_client_attachment_manager() {
        let config = create_test_config("test_token_1234567890123456789012345678901234567890");
        let client = DiscordClient::new(config);
        
        let manager = client.attachment_manager();
        assert_eq!(manager.max_file_size, 25 * 1024 * 1024);
        
        let manager_custom = client.attachment_manager_with_limit(1024 * 1024);
        assert_eq!(manager_custom.max_file_size, 1024 * 1024);
    }

    #[test]
    fn test_message_type_colors() {
        assert_eq!(MessageType::Success.color(), Colour::from_rgb(34, 197, 94));
        assert_eq!(MessageType::Info.color(), Colour::from_rgb(59, 130, 246));
        assert_eq!(MessageType::Warning.color(), Colour::from_rgb(245, 158, 11));
        assert_eq!(MessageType::Error.color(), Colour::from_rgb(239, 68, 68));
        assert_eq!(MessageType::Graph.color(), Colour::from_rgb(147, 51, 234));
        assert_eq!(MessageType::System.color(), Colour::from_rgb(107, 114, 128));
    }

    #[test]
    fn test_message_type_emojis() {
        assert_eq!(MessageType::Success.emoji(), "✅");
        assert_eq!(MessageType::Info.emoji(), "ℹ️");
        assert_eq!(MessageType::Warning.emoji(), "⚠️");
        assert_eq!(MessageType::Error.emoji(), "❌");
        assert_eq!(MessageType::Graph.emoji(), "📊");
        assert_eq!(MessageType::System.emoji(), "🤖");
    }

    #[test]
    fn test_metadata_field_creation() {
        let field = MetadataField::new("Test", "Value");
        assert_eq!(field.name, "Test");
        assert_eq!(field.value, "Value");
        assert!(!field.inline);

        let inline_field = MetadataField::inline("Test", "Value");
        assert_eq!(inline_field.name, "Test");
        assert_eq!(inline_field.value, "Value");
        assert!(inline_field.inline);
    }

    #[test]
    fn test_metadata_field_with_inline() {
        let field = MetadataField::new("Test", "Value").with_inline(true);
        assert!(field.inline);

        let field = MetadataField::inline("Test", "Value").with_inline(false);
        assert!(!field.inline);
    }

    #[test]
    fn test_discord_message_builder_creation() {
        let builder = DiscordMessageBuilder::new(MessageType::Info);
        
        // Check that builder was created with correct defaults
        assert_eq!(builder.message_type, MessageType::Info);
        assert!(builder.content.is_none());
        assert!(builder.title.is_none());
        assert!(builder.description.is_none());
        assert!(builder.include_generation_time);
    }

    #[test]
    fn test_discord_message_builder_factory_methods() {
        let graph_builder = DiscordMessageBuilder::graph();
        assert_eq!(graph_builder.message_type, MessageType::Graph);

        let success_builder = DiscordMessageBuilder::success();
        assert_eq!(success_builder.message_type, MessageType::Success);

        let info_builder = DiscordMessageBuilder::info();
        assert_eq!(info_builder.message_type, MessageType::Info);

        let warning_builder = DiscordMessageBuilder::warning();
        assert_eq!(warning_builder.message_type, MessageType::Warning);

        let error_builder = DiscordMessageBuilder::error();
        assert_eq!(error_builder.message_type, MessageType::Error);

        let system_builder = DiscordMessageBuilder::system();
        assert_eq!(system_builder.message_type, MessageType::System);
    }

    #[test]
    fn test_discord_message_builder_fluent_interface() {
        let builder = DiscordMessageBuilder::graph()
            .title("Test Title")
            .description("Test Description")
            .content("Test Content")
            .author("Test Author")
            .footer("Test Footer")
            .thumbnail("https://example.com/thumb.png")
            .image("https://example.com/image.png")
            .include_generation_time(false)
            .add_field("Field1", "Value1", true)
            .add_field("Field2", "Value2", false);

        // Verify all properties were set correctly
        assert_eq!(builder.title.as_ref().unwrap(), "Test Title");
        assert_eq!(builder.description.as_ref().unwrap(), "Test Description");
        assert_eq!(builder.content.as_ref().unwrap(), "Test Content");
        assert_eq!(builder.author.as_ref().unwrap(), "Test Author");
        assert_eq!(builder.footer.as_ref().unwrap(), "Test Footer");
        assert_eq!(builder.thumbnail.as_ref().unwrap(), "https://example.com/thumb.png");
        assert_eq!(builder.image.as_ref().unwrap(), "https://example.com/image.png");
        assert!(!builder.include_generation_time);
        assert_eq!(builder.fields.len(), 2);
        assert_eq!(builder.fields[0].name, "Field1");
        assert_eq!(builder.fields[0].value, "Value1");
        assert!(builder.fields[0].inline);
        assert_eq!(builder.fields[1].name, "Field2");
        assert_eq!(builder.fields[1].value, "Value2");
        assert!(!builder.fields[1].inline);
    }

    #[test]
    fn test_discord_message_builder_with_timestamp() {
        let timestamp = Utc::now();
        let builder = DiscordMessageBuilder::info()
            .title("Test")
            .timestamp(timestamp);

        assert!(builder.timestamp.is_some());
        assert_eq!(builder.timestamp.unwrap(), timestamp);
    }

    #[test]
    fn test_discord_message_builder_author_with_icon() {
        let builder = DiscordMessageBuilder::info()
            .author_with_icon("Author Name", "https://example.com/icon.png");

        assert_eq!(builder.author.as_ref().unwrap(), "Author Name");
        assert_eq!(builder.author_icon.as_ref().unwrap(), "https://example.com/icon.png");
    }

    #[test]
    fn test_discord_message_builder_footer_with_icon() {
        let builder = DiscordMessageBuilder::info()
            .footer_with_icon("Footer Text", "https://example.com/footer.png");

        assert_eq!(builder.footer.as_ref().unwrap(), "Footer Text");
        assert_eq!(builder.footer_icon.as_ref().unwrap(), "https://example.com/footer.png");
    }

    #[test]
    fn test_message_template_creation() {
        let template = MessageTemplate::new(
            "test_template",
            "Test {placeholder}",
            "Description {placeholder}",
            MessageType::Info,
        );

        assert_eq!(template.name, "test_template");
        assert_eq!(template.title_template, "Test {placeholder}");
        assert_eq!(template.description_template, "Description {placeholder}");
        assert_eq!(template.message_type, MessageType::Info);
        assert!(template.default_fields.is_empty());
    }

    #[test]
    fn test_message_template_with_fields() {
        let template = MessageTemplate::new(
            "test_template",
            "Test Title",
            "Test Description",
            MessageType::Success,
        )
        .with_field(MetadataField::inline("Status", "Success"))
        .with_field(MetadataField::new("Details", "Additional info"));

        assert_eq!(template.default_fields.len(), 2);
        assert_eq!(template.default_fields[0].name, "Status");
        assert_eq!(template.default_fields[0].value, "Success");
        assert!(template.default_fields[0].inline);
        assert_eq!(template.default_fields[1].name, "Details");
        assert_eq!(template.default_fields[1].value, "Additional info");
        assert!(!template.default_fields[1].inline);
    }

    #[test]
    fn test_message_template_builder() {
        let template = MessageTemplate::new(
            "test_template",
            "Test Title",
            "Test Description",
            MessageType::Warning,
        )
        .with_field(MetadataField::inline("Field", "Value"));

        let builder = template.builder();
        assert_eq!(builder.message_type, MessageType::Warning);
        assert_eq!(builder.title.as_ref().unwrap(), "Test Title");
        assert_eq!(builder.description.as_ref().unwrap(), "Test Description");
        assert_eq!(builder.fields.len(), 1);
    }

    #[test]
    fn test_message_template_apply_with_replacements() {
        let template = MessageTemplate::new(
            "test_template",
            "Hello {name}",
            "Welcome to {place}",
            MessageType::Info,
        )
        .with_field(MetadataField::inline("User", "{name}"))
        .with_field(MetadataField::new("Location", "{place}"));

        let replacements = [
            ("{name}", "Alice"),
            ("{place}", "Wonderland"),
        ];

        let builder = template.apply_with_replacements(&replacements);
        assert_eq!(builder.title.as_ref().unwrap(), "Hello Alice");
        assert_eq!(builder.description.as_ref().unwrap(), "Welcome to Wonderland");
        assert_eq!(builder.fields.len(), 2);
        assert_eq!(builder.fields[0].name, "User");
        assert_eq!(builder.fields[0].value, "Alice");
        assert_eq!(builder.fields[1].name, "Location");
        assert_eq!(builder.fields[1].value, "Wonderland");
    }

    #[test]
    fn test_message_templates_graph_success() {
        let template = MessageTemplates::graph_success();
        assert_eq!(template.name, "graph_success");
        assert_eq!(template.message_type, MessageType::Graph);
        assert!(template.title_template.contains("Graph Generated Successfully"));
        assert!(template.description_template.contains("{graph_type}"));
        assert_eq!(template.default_fields.len(), 2);
    }

    #[test]
    fn test_message_templates_graph_error() {
        let template = MessageTemplates::graph_error();
        assert_eq!(template.name, "graph_error");
        assert_eq!(template.message_type, MessageType::Error);
        assert!(template.title_template.contains("Graph Generation Failed"));
        assert!(template.description_template.contains("{graph_type}"));
        assert!(template.description_template.contains("{error}"));
    }

    #[test]
    fn test_message_templates_command_received() {
        let template = MessageTemplates::command_received();
        assert_eq!(template.name, "command_received");
        assert_eq!(template.message_type, MessageType::Info);
        assert!(template.description_template.contains("{command}"));
    }

    #[test]
    fn test_message_templates_permission_error() {
        let template = MessageTemplates::permission_error();
        assert_eq!(template.name, "permission_error");
        assert_eq!(template.message_type, MessageType::Error);
        assert!(template.title_template.contains("Permission Denied"));
        assert!(template.description_template.contains("{action}"));
    }

    #[test]
    fn test_discord_message_builder_build_with_attachments() {
        // Create a test PNG attachment
        let png_data = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A,
            0x00, 0x00, 0x00, 0x0D,
            0x49, 0x48, 0x44, 0x52,
        ];
        
        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data)
            .expect("Failed to create attachment");

        let builder = DiscordMessageBuilder::graph()
            .title("Test Graph")
            .description("Test graph with attachment");

        let message = builder.build_with_attachments(vec![attachment]);
        
        // The message should be created successfully
        // Note: We can't easily test the internal structure of CreateMessage 
        // without actually sending it, but we can verify it builds without error
        // We'll just verify the build completes without panicking
        let _ = message;
    }

    #[test]
    fn test_graph_attachment_to_discord_attachment() {
        let png_data = vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48, 0x44, 0x52];
        let attachment = GraphAttachment::from_data("test.png".to_string(), png_data)
            .unwrap()
            .with_description("Test graph attachment");

        let _discord_attachment = attachment.to_discord_attachment();
        
        // We can't directly test the CreateAttachment structure since it's opaque,
        // but we can verify our attachment was created successfully
        assert_eq!(attachment.filename, "test.png");
        assert_eq!(attachment.description, Some("Test graph attachment".to_string()));
    }

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.base_delay_ms, 1000);
        assert_eq!(config.max_delay_ms, 30000);
        assert_eq!(config.jitter_factor, 0.1);
    }

    #[test]
    fn test_retry_config_new() {
        let config = RetryConfig::new(5, 2000, 60000);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.base_delay_ms, 2000);
        assert_eq!(config.max_delay_ms, 60000);
        assert_eq!(config.jitter_factor, 0.1);
    }

    #[test]
    fn test_retry_config_with_jitter() {
        let config = RetryConfig::new(3, 1000, 30000).with_jitter(0.2);
        assert_eq!(config.jitter_factor, 0.2);
        
        // Test clamp behavior
        let config_high = RetryConfig::new(3, 1000, 30000).with_jitter(1.5);
        assert_eq!(config_high.jitter_factor, 1.0);
        
        let config_low = RetryConfig::new(3, 1000, 30000).with_jitter(-0.5);
        assert_eq!(config_low.jitter_factor, 0.0);
    }

    #[test]
    fn test_retry_config_calculate_delay() {
        let config = RetryConfig::new(3, 1000, 10000).with_jitter(0.0); // No jitter for predictable testing
        
        // First attempt (attempt 0)
        let delay0 = config.calculate_delay(0);
        assert_eq!(delay0.as_millis(), 1000); // base_delay_ms * 2^0 = 1000
        
        // Second attempt (attempt 1)
        let delay1 = config.calculate_delay(1);
        assert_eq!(delay1.as_millis(), 2000); // base_delay_ms * 2^1 = 2000
        
        // Third attempt (attempt 2)
        let delay2 = config.calculate_delay(2);
        assert_eq!(delay2.as_millis(), 4000); // base_delay_ms * 2^2 = 4000
        
        // Fourth attempt (attempt 3)
        let delay3 = config.calculate_delay(3);
        assert_eq!(delay3.as_millis(), 8000); // base_delay_ms * 2^3 = 8000
        
        // Fifth attempt (attempt 4) - should hit the max delay cap
        let delay4 = config.calculate_delay(4);
        assert_eq!(delay4.as_millis(), 10000); // capped at max_delay_ms
    }

    #[test]
    fn test_retry_config_calculate_delay_with_jitter() {
        let config = RetryConfig::new(3, 1000, 30000).with_jitter(0.1);
        
        // With jitter, delays should vary but be within expected range
        let delay0 = config.calculate_delay(0);
        let base_delay = 1000u64;
        let min_delay = (base_delay as f64 * 0.9) as u64; // 10% jitter down
        let max_delay = (base_delay as f64 * 1.1) as u64; // 10% jitter up
        
        assert!(delay0.as_millis() >= min_delay as u128);
        assert!(delay0.as_millis() <= max_delay as u128);
    }

    #[test]
    fn test_post_result_success() {
        use std::time::Duration;
        use poise::serenity_prelude::MessageId;
        
        let message_id = MessageId::new(123456789);
        let result = PostResult::success(message_id, 2, Duration::from_secs(3), true);
        
        assert!(result.success);
        assert_eq!(result.message_id, Some(message_id));
        assert_eq!(result.attempts, 2);
        assert_eq!(result.duration, Duration::from_secs(3));
        assert!(result.rate_limited);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_post_result_failure() {
        use std::time::Duration;
        
        let result = PostResult::failure("Connection failed".to_string(), 3, Duration::from_secs(10), false);
        
        assert!(!result.success);
        assert!(result.message_id.is_none());
        assert_eq!(result.attempts, 3);
        assert_eq!(result.duration, Duration::from_secs(10));
        assert!(!result.rate_limited);
        assert_eq!(result.error, Some("Connection failed".to_string()));
    }

    #[test]
    fn test_post_result_status_message() {
        use std::time::Duration;
        use poise::serenity_prelude::MessageId;
        
        // Success message
        let success_result = PostResult::success(MessageId::new(123), 1, Duration::from_millis(1500), false);
        let success_msg = success_result.status_message();
        assert!(success_msg.contains("✅"));
        assert!(success_msg.contains("1 attempt"));
        assert!(success_msg.contains("1.50s"));
        
        // Success with rate limiting
        let success_rate_limited = PostResult::success(MessageId::new(456), 3, Duration::from_secs(5), true);
        let success_rate_msg = success_rate_limited.status_message();
        assert!(success_rate_msg.contains("✅"));
        assert!(success_rate_msg.contains("3 attempt"));
        assert!(success_rate_msg.contains("overcame rate limits"));
        
        // Failure message
        let failure_result = PostResult::failure("Network error".to_string(), 2, Duration::from_secs(2), false);
        let failure_msg = failure_result.status_message();
        assert!(failure_msg.contains("❌"));
        assert!(failure_msg.contains("2 attempt"));
        assert!(failure_msg.contains("Network error"));
        
        // Failure with rate limiting
        let failure_rate_limited = PostResult::failure("Rate limited".to_string(), 4, Duration::from_secs(8), true);
        let failure_rate_msg = failure_rate_limited.status_message();
        assert!(failure_rate_msg.contains("❌"));
        assert!(failure_rate_msg.contains("4 attempt"));
        assert!(failure_rate_msg.contains("rate limited"));
    }

    #[test]
    fn test_is_rate_limit_error() {
        use poise::serenity_prelude as serenity;
        
        // Test with a mock error containing rate limit keywords
        let rate_limit_error = serenity::Error::Other("Rate limit exceeded");
        assert!(DiscordClient::is_rate_limit_error(&rate_limit_error));
        
        let status_429_error = serenity::Error::Other("HTTP 429 Too Many Requests");
        assert!(DiscordClient::is_rate_limit_error(&status_429_error));
        
        let normal_error = serenity::Error::Other("Invalid channel");
        assert!(!DiscordClient::is_rate_limit_error(&normal_error));
    }

    #[test]
    fn test_is_retryable_error() {
        use poise::serenity_prelude as serenity;
        
        // Gateway errors should be retryable
        let gateway_error = serenity::Error::Gateway(serenity::GatewayError::InvalidShardData);
        assert!(DiscordClient::is_retryable_error(&gateway_error));
        
        // Non-HTTP/Gateway errors should generally not be retryable
        let io_error = std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid JSON");
        let json_error = serenity::Error::Json(serde_json::Error::io(io_error));
        assert!(!DiscordClient::is_retryable_error(&json_error));
        
        let other_error = serenity::Error::Other("Some other error");
        assert!(!DiscordClient::is_retryable_error(&other_error));
        
        // Test the string-based retry logic with mock HTTP errors
        // Since we're using string matching, we can test with HTTP errors that contain the keywords
        let timeout_http_error = serenity::Error::Other("Connection timeout occurred");
        // For Other errors, it won't match HTTP pattern, but we can test the string logic separately
        
        // Test that our string-based checking works for common error messages
        assert!(!DiscordClient::is_retryable_error(&timeout_http_error)); // Other errors are not retryable in our implementation
    }
} 