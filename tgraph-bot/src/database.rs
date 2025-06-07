//! Database module for user preferences and privacy settings storage
//! 
//! This module provides a sled-based embedded database for storing user preferences,
//! privacy settings, and related data with GDPR-compliant handling.

use std::path::Path;
use std::sync::Arc;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use tracing::{info, warn, debug};

/// User privacy preferences and settings
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPreferences {
    /// Discord user ID
    pub user_id: u64,
    /// Whether to show username in statistics (for privacy)
    pub username_visible: bool,
    /// Data retention period in days (None means indefinite)
    pub data_retention_days: Option<u32>,
    /// Whether statistics can be shared publicly
    pub allow_public_stats: bool,
    /// Whether to allow data export requests
    pub allow_data_export: bool,
    /// User's preferred language code (e.g., "en", "es", "fr")
    pub preferred_language: Option<String>,
    /// Whether to send statistics via DM instead of channel
    pub prefer_dm_delivery: bool,
    /// Timestamp when preferences were created
    pub created_at: DateTime<Utc>,
    /// Timestamp when preferences were last updated
    pub updated_at: DateTime<Utc>,
}

impl UserPreferences {
    /// Create new user preferences with sensible defaults
    pub fn new(user_id: u64) -> Self {
        let now = Utc::now();
        Self {
            user_id,
            username_visible: true,      // Default to visible
            data_retention_days: None,   // Default to indefinite retention
            allow_public_stats: false,   // Default to private stats
            allow_data_export: true,     // Default to allowing export
            preferred_language: None,    // Default to None (use bot default)
            prefer_dm_delivery: true,    // Default to DM delivery for privacy
            created_at: now,
            updated_at: now,
        }
    }

    /// Update the timestamp to mark when preferences were modified
    pub fn mark_updated(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Check if data should be retained based on retention policy
    pub fn should_retain_data(&self) -> bool {
        match self.data_retention_days {
            None => true, // Indefinite retention
            Some(days) => {
                let retention_period = chrono::Duration::days(days as i64);
                let cutoff = Utc::now() - retention_period;
                self.updated_at > cutoff
            }
        }
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self::new(0) // Placeholder user ID
    }
}

/// Database manager for user preferences using sled embedded database
#[derive(Debug, Clone)]
pub struct UserDatabase {
    /// Sled database instance
    db: Arc<sled::Db>,
    /// Tree for storing user preferences
    preferences_tree: sled::Tree,
}

impl UserDatabase {
    /// Initialize the database with the given path
    ///
    /// # Arguments
    /// * `db_path` - Path where the database files should be stored
    ///
    /// # Returns
    /// * `Result<Self>` - Database instance or error
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        info!("Initializing user preferences database at: {:?}", db_path.as_ref());

        // Configure sled database
        let config = sled::Config::default()
            .path(db_path.as_ref())
            .cache_capacity(64 * 1024 * 1024) // 64MB cache
            .flush_every_ms(Some(1000))       // Flush every second
;

        let db = config.open()
            .with_context(|| format!("Failed to open database at {:?}", db_path.as_ref()))?;

        let preferences_tree = db.open_tree("user_preferences")
            .context("Failed to open user preferences tree")?;

        let database = Self {
            db: Arc::new(db),
            preferences_tree,
        };

        info!("User preferences database initialized successfully");
        Ok(database)
    }

    /// Store or update user preferences
    ///
    /// # Arguments
    /// * `preferences` - User preferences to store
    ///
    /// # Returns
    /// * `Result<()>` - Success or error
    pub async fn store_preferences(&self, mut preferences: UserPreferences) -> Result<()> {
        debug!("Storing preferences for user {}", preferences.user_id);

        // Update the timestamp
        preferences.mark_updated();

        // Serialize preferences to JSON
        let json_data = serde_json::to_vec(&preferences)
            .context("Failed to serialize user preferences")?;

        // Use user ID as key
        let key = preferences.user_id.to_be_bytes();

        // Store in database
        self.preferences_tree.insert(&key, json_data)
            .context("Failed to insert preferences into database")?;

        // Flush to ensure data is persisted
        self.preferences_tree.flush_async().await
            .context("Failed to flush preferences to disk")?;

        debug!("Successfully stored preferences for user {}", preferences.user_id);
        Ok(())
    }

    /// Retrieve user preferences by user ID
    ///
    /// # Arguments
    /// * `user_id` - Discord user ID
    ///
    /// # Returns
    /// * `Result<Option<UserPreferences>>` - Preferences if found, None if not found, or error
    pub fn get_preferences(&self, user_id: u64) -> Result<Option<UserPreferences>> {
        debug!("Retrieving preferences for user {}", user_id);

        let key = user_id.to_be_bytes();

        match self.preferences_tree.get(&key)
            .context("Failed to query database for user preferences")? {
            Some(data) => {
                let preferences: UserPreferences = serde_json::from_slice(&data)
                    .context("Failed to deserialize user preferences")?;
                
                debug!("Found preferences for user {}", user_id);
                Ok(Some(preferences))
            }
            None => {
                debug!("No preferences found for user {}", user_id);
                Ok(None)
            }
        }
    }

    /// Get user preferences, creating default preferences if they don't exist
    ///
    /// # Arguments
    /// * `user_id` - Discord user ID
    ///
    /// # Returns
    /// * `Result<UserPreferences>` - User preferences (created if not found) or error
    pub async fn get_or_create_preferences(&self, user_id: u64) -> Result<UserPreferences> {
        match self.get_preferences(user_id)? {
            Some(preferences) => Ok(preferences),
            None => {
                debug!("Creating default preferences for new user {}", user_id);
                let preferences = UserPreferences::new(user_id);
                self.store_preferences(preferences.clone()).await?;
                Ok(preferences)
            }
        }
    }

    /// Update specific preference fields for a user
    ///
    /// # Arguments
    /// * `user_id` - Discord user ID
    /// * `update_fn` - Function that modifies the preferences
    ///
    /// # Returns
    /// * `Result<UserPreferences>` - Updated preferences or error
    pub async fn update_preferences<F>(&self, user_id: u64, update_fn: F) -> Result<UserPreferences>
    where
        F: FnOnce(&mut UserPreferences),
    {
        debug!("Updating preferences for user {}", user_id);

        let mut preferences = self.get_or_create_preferences(user_id).await?;
        update_fn(&mut preferences);
        self.store_preferences(preferences.clone()).await?;

        debug!("Successfully updated preferences for user {}", user_id);
        Ok(preferences)
    }

    /// Delete user preferences (for GDPR compliance)
    ///
    /// # Arguments
    /// * `user_id` - Discord user ID
    ///
    /// # Returns
    /// * `Result<bool>` - True if preferences were deleted, false if they didn't exist
    pub async fn delete_preferences(&self, user_id: u64) -> Result<bool> {
        info!("Deleting preferences for user {} (GDPR compliance)", user_id);

        let key = user_id.to_be_bytes();

        let existed = self.preferences_tree.remove(&key)
            .context("Failed to delete user preferences from database")?
            .is_some();

        if existed {
            // Flush to ensure deletion is persisted
            self.preferences_tree.flush_async().await
                .context("Failed to flush preferences deletion to disk")?;
            
            info!("Successfully deleted preferences for user {}", user_id);
        } else {
            warn!("Attempted to delete preferences for user {} but they didn't exist", user_id);
        }

        Ok(existed)
    }

    /// List all user IDs with stored preferences (for admin/maintenance purposes)
    ///
    /// # Returns
    /// * `Result<Vec<u64>>` - List of user IDs or error
    pub fn list_all_user_ids(&self) -> Result<Vec<u64>> {
        debug!("Listing all user IDs with stored preferences");

        let mut user_ids = Vec::new();

        for result in self.preferences_tree.iter() {
            let (key, _) = result.context("Failed to iterate over preferences tree")?;
            
            if key.len() == 8 {
                let user_id = u64::from_be_bytes(
                    key.as_ref().try_into()
                        .context("Invalid key format in preferences tree")?
                );
                user_ids.push(user_id);
            } else {
                warn!("Found invalid key length in preferences tree: {} bytes", key.len());
            }
        }

        debug!("Found {} users with stored preferences", user_ids.len());
        Ok(user_ids)
    }

    /// Export all data for a specific user (for GDPR compliance)
    ///
    /// # Arguments
    /// * `user_id` - Discord user ID
    ///
    /// # Returns
    /// * `Result<Option<serde_json::Value>>` - User data as JSON or None if not found
    pub fn export_user_data(&self, user_id: u64) -> Result<Option<serde_json::Value>> {
        info!("Exporting data for user {} (GDPR compliance)", user_id);

        match self.get_preferences(user_id)? {
            Some(preferences) => {
                let json_value = serde_json::to_value(&preferences)
                    .context("Failed to convert preferences to JSON value")?;
                Ok(Some(json_value))
            }
            None => Ok(None)
        }
    }

    /// Clean up old data based on retention policies
    ///
    /// # Returns
    /// * `Result<usize>` - Number of records cleaned up
    pub async fn cleanup_expired_data(&self) -> Result<usize> {
        info!("Starting cleanup of expired user data");
        
        let mut cleanup_count = 0;
        let mut to_delete = Vec::new();

        // Collect users whose data should be deleted
        for result in self.preferences_tree.iter() {
            let (key, value) = result.context("Failed to iterate over preferences tree")?;
            
            if key.len() == 8 {
                let preferences: UserPreferences = serde_json::from_slice(&value)
                    .context("Failed to deserialize preferences during cleanup")?;
                
                if !preferences.should_retain_data() {
                    to_delete.push(preferences.user_id);
                }
            }
        }

        // Delete expired data
        for user_id in to_delete {
            if self.delete_preferences(user_id).await? {
                cleanup_count += 1;
                info!("Cleaned up expired data for user {}", user_id);
            }
        }

        info!("Cleanup complete: removed {} expired user records", cleanup_count);
        Ok(cleanup_count)
    }

    /// Get database statistics
    ///
    /// # Returns
    /// * `Result<DatabaseStats>` - Database statistics
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let user_count = self.list_all_user_ids()?.len();
        
        let size_on_disk = self.db.size_on_disk()
            .context("Failed to get database size")?;

        Ok(DatabaseStats {
            user_count,
            size_on_disk,
            database_path: String::from_utf8_lossy(&self.db.name()).to_string(),
        })
    }

    /// Flush all pending writes to disk
    pub async fn flush(&self) -> Result<()> {
        self.preferences_tree.flush_async().await
            .context("Failed to flush database to disk")?;
        Ok(())
    }

    /// Close the database connection gracefully
    pub async fn close(&self) -> Result<()> {
        info!("Closing user preferences database");
        
        // Flush any pending writes
        self.flush().await?;
        
        info!("User preferences database closed successfully");
        Ok(())
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    /// Number of users with stored preferences
    pub user_count: usize,
    /// Database size on disk in bytes
    pub size_on_disk: u64,
    /// Database file path
    pub database_path: String,
}

impl DatabaseStats {
    /// Format the database size in human-readable format
    pub fn format_size(&self) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = self.size_on_disk as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        format!("{:.2} {}", size, UNITS[unit_index])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;


    #[tokio::test]
    async fn test_database_initialization() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db_path = temp_dir.path().join("test_db");
        
        let db = UserDatabase::new(&db_path).expect("Failed to initialize database");
        assert!(db_path.exists());
    }

    #[tokio::test]
    async fn test_store_and_retrieve_preferences() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id = 12345u64;
        let preferences = UserPreferences::new(user_id);

        // Store preferences
        db.store_preferences(preferences.clone()).await
            .expect("Failed to store preferences");

        // Retrieve preferences
        let retrieved = db.get_preferences(user_id)
            .expect("Failed to retrieve preferences")
            .expect("Preferences not found");

        assert_eq!(retrieved.user_id, user_id);
        assert_eq!(retrieved.username_visible, preferences.username_visible);
    }

    #[tokio::test]
    async fn test_get_or_create_preferences() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id = 67890u64;

        // Should create new preferences
        let preferences = db.get_or_create_preferences(user_id).await
            .expect("Failed to get or create preferences");
        
        assert_eq!(preferences.user_id, user_id);

        // Should retrieve existing preferences
        let preferences2 = db.get_or_create_preferences(user_id).await
            .expect("Failed to get existing preferences");
        
        assert_eq!(preferences.created_at, preferences2.created_at);
    }

    #[tokio::test]
    async fn test_update_preferences() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id = 11111u64;

        // Update preferences
        let updated = db.update_preferences(user_id, |prefs| {
            prefs.username_visible = false;
            prefs.allow_public_stats = true;
        }).await.expect("Failed to update preferences");

        assert_eq!(updated.username_visible, false);
        assert_eq!(updated.allow_public_stats, true);

        // Verify persistence
        let retrieved = db.get_preferences(user_id)
            .expect("Failed to retrieve updated preferences")
            .expect("Updated preferences not found");
        
        assert_eq!(retrieved.username_visible, false);
        assert_eq!(retrieved.allow_public_stats, true);
    }

    #[tokio::test]
    async fn test_delete_preferences() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id = 22222u64;
        let preferences = UserPreferences::new(user_id);

        // Store preferences
        db.store_preferences(preferences).await
            .expect("Failed to store preferences");

        // Verify they exist
        assert!(db.get_preferences(user_id)
            .expect("Failed to check preferences")
            .is_some());

        // Delete preferences
        let deleted = db.delete_preferences(user_id).await
            .expect("Failed to delete preferences");
        assert!(deleted);

        // Verify they're gone
        assert!(db.get_preferences(user_id)
            .expect("Failed to check deleted preferences")
            .is_none());

        // Try to delete again
        let deleted_again = db.delete_preferences(user_id).await
            .expect("Failed to delete non-existent preferences");
        assert!(!deleted_again);
    }

    #[tokio::test]
    async fn test_list_user_ids() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_ids = vec![33333u64, 44444u64, 55555u64];

        // Store preferences for multiple users
        for user_id in &user_ids {
            let preferences = UserPreferences::new(*user_id);
            db.store_preferences(preferences).await
                .expect("Failed to store preferences");
        }

        // List all user IDs
        let mut retrieved_ids = db.list_all_user_ids()
            .expect("Failed to list user IDs");
        retrieved_ids.sort();

        let mut expected_ids = user_ids.clone();
        expected_ids.sort();

        assert_eq!(retrieved_ids, expected_ids);
    }

    #[tokio::test]
    async fn test_export_user_data() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id = 66666u64;
        let preferences = UserPreferences::new(user_id);

        // Store preferences
        db.store_preferences(preferences.clone()).await
            .expect("Failed to store preferences");

        // Export data
        let exported = db.export_user_data(user_id)
            .expect("Failed to export user data")
            .expect("No data found for export");

        // Verify exported data contains expected fields
        assert!(exported.get("user_id").is_some());
        assert!(exported.get("username_visible").is_some());
        assert!(exported.get("created_at").is_some());
    }

    #[test]
    fn test_user_preferences_defaults() {
        let user_id = 77777u64;
        let preferences = UserPreferences::new(user_id);

        assert_eq!(preferences.user_id, user_id);
        assert_eq!(preferences.username_visible, true);
        assert_eq!(preferences.data_retention_days, None);
        assert_eq!(preferences.allow_public_stats, false);
        assert_eq!(preferences.allow_data_export, true);
        assert_eq!(preferences.preferred_language, None);
        assert_eq!(preferences.prefer_dm_delivery, true);
    }

    #[test]
    fn test_data_retention_logic() {
        let user_id = 88888u64;
        let mut preferences = UserPreferences::new(user_id);

        // Should retain with no retention policy
        assert!(preferences.should_retain_data());

        // Should retain within retention period
        preferences.data_retention_days = Some(30);
        assert!(preferences.should_retain_data());

        // Should not retain when data is old
        preferences.data_retention_days = Some(1);
        preferences.updated_at = Utc::now() - chrono::Duration::days(2);
        assert!(!preferences.should_retain_data());
    }

    #[tokio::test]
    async fn test_cleanup_expired_data() {
        let temp_dir = tempdir().expect("Failed to create temp directory");
        let db = UserDatabase::new(temp_dir.path().join("test_db"))
            .expect("Failed to initialize database");

        let user_id_retain = 99999u64;
        let user_id_expire = 99998u64;

        // Create preferences that should be retained
        let mut prefs_retain = UserPreferences::new(user_id_retain);
        prefs_retain.data_retention_days = Some(30); // 30 days retention
        db.store_preferences(prefs_retain).await
            .expect("Failed to store preferences to retain");

        // Create preferences that should be expired
        let mut prefs_expire = UserPreferences::new(user_id_expire);
        prefs_expire.data_retention_days = Some(1); // 1 day retention
        prefs_expire.updated_at = Utc::now() - chrono::Duration::days(2); // 2 days old
        
        // We need to manually insert this to avoid the automatic timestamp update
        let json_data = serde_json::to_vec(&prefs_expire)
            .expect("Failed to serialize preferences");
        let key = user_id_expire.to_be_bytes();
        db.preferences_tree.insert(&key, json_data)
            .expect("Failed to insert expired preferences");

        // Run cleanup
        let cleanup_count = db.cleanup_expired_data().await
            .expect("Failed to cleanup expired data");

        assert_eq!(cleanup_count, 1);

        // Verify retained data still exists
        assert!(db.get_preferences(user_id_retain)
            .expect("Failed to check retained preferences")
            .is_some());

        // Verify expired data is gone
        assert!(db.get_preferences(user_id_expire)
            .expect("Failed to check expired preferences")
            .is_none());
    }

    #[test]
    fn test_database_stats_format_size() {
        let stats = DatabaseStats {
            user_count: 100,
            size_on_disk: 1024,
            database_path: "/test/path".to_string(),
        };

        assert_eq!(stats.format_size(), "1.00 KB");

        let stats_large = DatabaseStats {
            user_count: 1000,
            size_on_disk: 1024 * 1024 * 1024,
            database_path: "/test/path".to_string(),
        };

        assert_eq!(stats_large.format_size(), "1.00 GB");
    }
} 