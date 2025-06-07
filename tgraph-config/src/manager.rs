//! Thread-safe configuration manager

use crate::{Config, ConfigError, ConfigLoader};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    mpsc, Arc, Mutex, RwLock, RwLockReadGuard, RwLockWriteGuard,
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};
use thiserror::Error;
use tgraph_common::Result as TGraphResult;

/// Enhanced configuration error for manager operations
#[derive(Debug, Error)]
pub enum ConfigManagerError {
    /// Configuration error from loader
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
    
    /// Lock poisoning error
    #[error("Configuration lock is poisoned")]
    LockPoisoned,
    
    /// Timeout acquiring lock
    #[error("Timeout acquiring configuration lock")]
    LockTimeout,
    
    /// File watching error
    #[error("File watching error: {0}")]
    WatcherError(#[from] notify::Error),
    
    /// Watcher already running
    #[error("Configuration file watcher is already running")]
    WatcherAlreadyRunning,
    
    /// Watcher not running
    #[error("Configuration file watcher is not running")]
    WatcherNotRunning,
    
    /// Thread join error
    #[error("Failed to join watcher thread")]
    ThreadJoinError,
}

impl From<ConfigManagerError> for tgraph_common::TGraphError {
    fn from(err: ConfigManagerError) -> Self {
        tgraph_common::TGraphError::config(err.to_string())
    }
}

/// Configuration watcher state
#[derive(Debug)]
struct WatcherState {
    /// Path to the configuration file being watched
    config_path: PathBuf,
    /// Handle to the watcher thread
    thread_handle: JoinHandle<()>,
    /// Signal to stop the watcher thread
    stop_signal: Arc<AtomicBool>,
}

/// Thread-safe configuration manager
/// 
/// Provides safe access to configuration across multiple threads using Arc<RwLock<Config>>.
/// Supports both read-only and mutable access patterns while preventing deadlocks.
/// Can optionally watch configuration files for changes and automatically reload.
#[derive(Debug)]
pub struct ConfigManager {
    /// The configuration wrapped in a thread-safe container
    config: Arc<RwLock<Config>>,
    /// Optional file watcher state
    watcher_state: Arc<Mutex<Option<WatcherState>>>,
}

impl Clone for ConfigManager {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            watcher_state: Arc::clone(&self.watcher_state),
        }
    }
}

impl ConfigManager {
    /// Create a new ConfigManager with the provided configuration
    pub fn new(config: Config) -> Self {
        Self {
            config: Arc::new(RwLock::new(config)),
            watcher_state: Arc::new(Mutex::new(None)),
        }
    }
    
    /// Load configuration from default sources and create a ConfigManager
    pub fn load() -> TGraphResult<Self> {
        let config = ConfigLoader::load()?;
        Ok(Self::new(config))
    }
    
    /// Load configuration from a specific file and create a ConfigManager
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> TGraphResult<Self> {
        let config = ConfigLoader::load_from_file(path)?;
        Ok(Self::new(config))
    }
    
    /// Get a cloned copy of the entire configuration
    /// 
    /// This method returns a complete clone of the configuration, which is safe
    /// to use without holding any locks. Use this when you need to access
    /// multiple configuration values or when you need to pass configuration
    /// to functions that might hold it for extended periods.
    pub fn get_config(&self) -> Result<Config, ConfigManagerError> {
        let guard = self.config.read()
            .map_err(|_| ConfigManagerError::LockPoisoned)?;
        Ok(guard.clone())
    }
    
    /// Execute a closure with read-only access to the configuration
    /// 
    /// This method provides efficient read-only access to the configuration
    /// without cloning. The closure receives a reference to the configuration
    /// and can return any value. Use this for read-only operations that don't
    /// need to hold the configuration beyond the closure execution.
    /// 
    /// # Example
    /// ```
    /// let discord_token = manager.with_config(|config| {
    ///     config.discord.token.clone()
    /// })?;
    /// ```
    pub fn with_config<F, R>(&self, f: F) -> Result<R, ConfigManagerError>
    where
        F: FnOnce(&Config) -> R,
    {
        let guard = self.config.read()
            .map_err(|_| ConfigManagerError::LockPoisoned)?;
        Ok(f(&guard))
    }
    
    /// Execute a closure with mutable access to the configuration
    /// 
    /// This method provides safe mutable access to the configuration.
    /// The closure receives a mutable reference to the configuration and
    /// can modify it. The configuration is automatically validated after
    /// the closure completes. If validation fails, the original configuration
    /// is preserved unchanged.
    /// 
    /// # Example
    /// ```
    /// manager.update_config(|config| {
    ///     config.discord.token = "new_token".to_string();
    /// })?;
    /// ```
    pub fn update_config<F>(&self, f: F) -> Result<(), ConfigManagerError>
    where
        F: FnOnce(&mut Config),
    {
        // First, get a copy of the current configuration
        let mut new_config = {
            let guard = self.config.read()
                .map_err(|_| ConfigManagerError::LockPoisoned)?;
            guard.clone()
        };
        
        // Apply the modification to the copy
        f(&mut new_config);
        
        // Validate the modified configuration
        new_config.validate_all()
            .map_err(|e| ConfigManagerError::Config(ConfigError::ValidationError(e)))?;
        
        // If validation succeeds, replace the original configuration
        let mut guard = self.config.write()
            .map_err(|_| ConfigManagerError::LockPoisoned)?;
        *guard = new_config;
        
        Ok(())
    }
    
    /// Replace the entire configuration with a new one
    /// 
    /// This method validates the new configuration before replacing the current one.
    /// If validation fails, the current configuration remains unchanged.
    pub fn replace_config(&self, new_config: Config) -> Result<(), ConfigManagerError> {
        // Validate the new configuration first
        new_config.validate_all()
            .map_err(|e| ConfigManagerError::Config(ConfigError::ValidationError(e)))?;
        
        let mut guard = self.config.write()
            .map_err(|_| ConfigManagerError::LockPoisoned)?;
        
        *guard = new_config;
        Ok(())
    }
    
    /// Reload configuration from the same source
    /// 
    /// This method attempts to reload the configuration using the default
    /// loading mechanism. If the reload fails, the current configuration
    /// remains unchanged.
    pub fn reload(&self) -> Result<(), ConfigManagerError> {
        let new_config = ConfigLoader::load()
            .map_err(|e| match e {
                tgraph_common::TGraphError::Config { message, source } => {
                    if let Some(config_err) = source.and_then(|s| s.downcast::<ConfigError>().ok()) {
                        ConfigManagerError::Config(*config_err)
                    } else {
                        ConfigManagerError::Config(ConfigError::MissingConfig(message))
                    }
                }
                _ => ConfigManagerError::Config(ConfigError::MissingConfig(e.to_string()))
            })?;
        
        self.replace_config(new_config)
    }
    
    /// Reload configuration from a specific file
    /// 
    /// This method attempts to reload the configuration from the specified file.
    /// If the reload fails, the current configuration remains unchanged.
    pub fn reload_from_file<P: AsRef<Path>>(&self, path: P) -> Result<(), ConfigManagerError> {
        let new_config = ConfigLoader::load_from_file(path)
            .map_err(|e| match e {
                tgraph_common::TGraphError::Config { message, source } => {
                    if let Some(config_err) = source.and_then(|s| s.downcast::<ConfigError>().ok()) {
                        ConfigManagerError::Config(*config_err)
                    } else {
                        ConfigManagerError::Config(ConfigError::MissingConfig(message))
                    }
                }
                _ => ConfigManagerError::Config(ConfigError::MissingConfig(e.to_string()))
            })?;
        
        self.replace_config(new_config)
    }
    
    /// Start watching a configuration file for changes and automatically reload
    /// 
    /// This method starts a background thread that monitors the specified file
    /// for changes. When changes are detected, the configuration is automatically
    /// reloaded and validated. If the new configuration is invalid, the old
    /// configuration is preserved and an error is logged.
    /// 
    /// # Arguments
    /// * `config_path` - Path to the configuration file to watch
    /// 
    /// # Errors
    /// Returns an error if the watcher is already running or if the file
    /// system watcher cannot be created.
    pub fn start_watching<P: AsRef<Path>>(&self, config_path: P) -> Result<(), ConfigManagerError> {
        let config_path = config_path.as_ref().to_path_buf();
        
        // Check if watcher is already running
        {
            let watcher_guard = self.watcher_state.lock()
                .map_err(|_| ConfigManagerError::LockPoisoned)?;
            
            if watcher_guard.is_some() {
                return Err(ConfigManagerError::WatcherAlreadyRunning);
            }
        }
        
        // Create a channel for file system events
        let (tx, rx) = mpsc::channel();
        
        // Create the file system watcher
        let mut watcher = RecommendedWatcher::new(tx, notify::Config::default())?;
        
        // Watch the configuration file
        watcher.watch(&config_path, RecursiveMode::NonRecursive)?;
        
        // Create stop signal
        let stop_signal = Arc::new(AtomicBool::new(false));
        let stop_signal_clone = Arc::clone(&stop_signal);
        
        // Clone necessary data for the thread
        let config_manager = self.clone();
        let config_path_clone = config_path.clone();
        
        // Spawn the watcher thread
        let thread_handle = thread::spawn(move || {
            Self::watcher_thread(config_manager, config_path_clone, rx, stop_signal_clone, watcher);
        });
        
        // Store the watcher state
        let watcher_state = WatcherState {
            config_path,
            thread_handle,
            stop_signal,
        };
        
        let mut watcher_guard = self.watcher_state.lock()
            .map_err(|_| ConfigManagerError::LockPoisoned)?;
        *watcher_guard = Some(watcher_state);
        
        tracing::info!("Configuration file watcher started");
        Ok(())
    }
    
    /// Stop watching the configuration file
    /// 
    /// This method stops the background file watcher thread and cleans up
    /// associated resources. If no watcher is currently running, this method
    /// returns an error.
    pub fn stop_watching(&self) -> Result<(), ConfigManagerError> {
        let watcher_state = {
            let mut watcher_guard = self.watcher_state.lock()
                .map_err(|_| ConfigManagerError::LockPoisoned)?;
            
            watcher_guard.take().ok_or(ConfigManagerError::WatcherNotRunning)?
        };
        
        // Signal the thread to stop
        watcher_state.stop_signal.store(true, Ordering::SeqCst);
        
        // Wait for the thread to finish
        watcher_state.thread_handle.join()
            .map_err(|_| ConfigManagerError::ThreadJoinError)?;
        
        tracing::info!("Configuration file watcher stopped");
        Ok(())
    }
    
    /// Check if the configuration file watcher is currently running
    pub fn is_watching(&self) -> bool {
        self.watcher_state.lock()
            .map(|guard| guard.is_some())
            .unwrap_or(false)
    }
    
    /// Get the path of the configuration file being watched, if any
    pub fn watched_path(&self) -> Option<PathBuf> {
        self.watcher_state.lock()
            .ok()?
            .as_ref()
            .map(|state| state.config_path.clone())
    }
    
    /// Background thread function for handling file system events
    fn watcher_thread(
        config_manager: ConfigManager,
        config_path: PathBuf,
        rx: mpsc::Receiver<notify::Result<Event>>,
        stop_signal: Arc<AtomicBool>,
        _watcher: RecommendedWatcher, // Keep watcher alive
    ) {
        let mut last_reload_time = Instant::now();
        const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);
        
        while !stop_signal.load(Ordering::SeqCst) {
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Ok(event)) => {
                    // Check if this event is relevant
                    if Self::should_reload_for_event(&event, &config_path) {
                        // Debounce: only reload if enough time has passed since last reload
                        let now = Instant::now();
                        if now.duration_since(last_reload_time) >= DEBOUNCE_DURATION {
                            last_reload_time = now;
                            
                            // Attempt to reload the configuration
                            if let Err(e) = config_manager.reload_from_file(&config_path) {
                                tracing::warn!(
                                    "Failed to reload configuration from {}: {}",
                                    config_path.display(),
                                    e
                                );
                            } else {
                                tracing::info!(
                                    "Configuration reloaded from {}",
                                    config_path.display()
                                );
                            }
                        }
                    }
                }
                Ok(Err(e)) => {
                    tracing::warn!("File watcher error: {}", e);
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {
                    // Normal timeout, continue loop
                    continue;
                }
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    tracing::debug!("File watcher channel disconnected");
                    break;
                }
            }
        }
        
        tracing::debug!("Configuration file watcher thread exiting");
    }
    
    /// Determine if a file system event should trigger a configuration reload
    fn should_reload_for_event(event: &Event, config_path: &Path) -> bool {
        match &event.kind {
            EventKind::Modify(_) | EventKind::Create(_) => {
                // Check if the event affects our configuration file
                event.paths.iter().any(|path| path == config_path)
            }
            EventKind::Remove(_) => {
                // If the file is removed, we might want to handle this differently
                // For now, we'll just log it and not reload
                if event.paths.iter().any(|path| path == config_path) {
                    tracing::warn!("Configuration file {} was removed", config_path.display());
                }
                false
            }
            _ => false,
        }
    }
    
    /// Get a read guard for low-level access
    /// 
    /// This method provides direct access to the RwLockReadGuard for advanced
    /// use cases. Most users should prefer `get_config()` or `with_config()`.
    /// 
    /// # Warning
    /// Be careful not to hold the guard for extended periods as it can block
    /// writers and potentially cause deadlocks.
    pub fn read_guard(&self) -> Result<RwLockReadGuard<'_, Config>, ConfigManagerError> {
        self.config.read()
            .map_err(|_| ConfigManagerError::LockPoisoned)
    }
    
    /// Get a write guard for low-level access
    /// 
    /// This method provides direct access to the RwLockWriteGuard for advanced
    /// use cases. Most users should prefer `update_config()`.
    /// 
    /// # Warning
    /// Be careful not to hold the guard for extended periods as it can block
    /// all other access and potentially cause deadlocks. Also, remember to
    /// validate the configuration manually when using this method.
    pub fn write_guard(&self) -> Result<RwLockWriteGuard<'_, Config>, ConfigManagerError> {
        self.config.write()
            .map_err(|_| ConfigManagerError::LockPoisoned)
    }
    
    /// Get an Arc clone of the internal RwLock for sharing between components
    /// 
    /// This method allows sharing the same configuration manager state
    /// between different components while maintaining thread safety.
    pub fn get_shared(&self) -> Arc<RwLock<Config>> {
        Arc::clone(&self.config)
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new(Config::default())
    }
}

impl Drop for ConfigManager {
    fn drop(&mut self) {
        // Attempt to stop the watcher gracefully
        if self.is_watching() {
            if let Err(e) = self.stop_watching() {
                tracing::warn!("Failed to stop configuration watcher during drop: {}", e);
            }
        }
    }
}

// Implement Send and Sync explicitly to ensure thread safety
unsafe impl Send for ConfigManager {}
unsafe impl Sync for ConfigManager {}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use notify::Event;

    fn create_test_config() -> Config {
        let mut config = Config::default();
        config.discord.token = "123456789.abcdef.ghijklmnop".to_string();
        config.tautulli.url = "http://localhost:8181".to_string();
        config.tautulli.api_key = "test_api_key".to_string();
        config.database.url = "sqlite::memory:".to_string();
        config
    }

    #[test]
    fn test_config_manager_creation() {
        let config = create_test_config();
        let manager = ConfigManager::new(config.clone());
        
        let retrieved_config = manager.get_config().unwrap();
        assert_eq!(retrieved_config.discord.token, config.discord.token);
    }

    #[test]
    fn test_with_config_read_access() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        let token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(token, "123456789.abcdef.ghijklmnop");
    }

    #[test]
    fn test_update_config() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        // Update the configuration
        manager.update_config(|config| {
            config.discord.token = "new.token.value".to_string();
        }).unwrap();
        
        // Verify the update
        let updated_token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(updated_token, "new.token.value");
    }

    #[test]
    fn test_update_config_validation_failure() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        // Try to update with invalid configuration
        let result = manager.update_config(|config| {
            config.discord.token = "".to_string(); // Invalid: empty token
        });
        
        assert!(result.is_err());
        
        // Verify original configuration is preserved
        let token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(token, "123456789.abcdef.ghijklmnop"); // Original value preserved
    }

    #[test]
    fn test_replace_config() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        let mut new_config = create_test_config();
        new_config.discord.token = "replaced.token.here".to_string();
        
        manager.replace_config(new_config).unwrap();
        
        let token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(token, "replaced.token.here");
    }

    #[test]
    fn test_replace_config_validation_failure() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        let mut invalid_config = create_test_config();
        invalid_config.discord.token = "".to_string(); // Invalid
        
        let result = manager.replace_config(invalid_config);
        assert!(result.is_err());
        
        // Original configuration should be preserved
        let token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(token, "123456789.abcdef.ghijklmnop");
    }

    #[test]
    fn test_concurrent_readers() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        let manager = Arc::new(manager);
        
        let (tx, rx) = mpsc::channel();
        let num_readers = 10;
        
        // Spawn multiple reader threads
        for i in 0..num_readers {
            let manager_clone = Arc::clone(&manager);
            let tx_clone = tx.clone();
            
            thread::spawn(move || {
                let token = manager_clone.with_config(|config| {
                    // Simulate some work
                    thread::sleep(Duration::from_millis(10));
                    config.discord.token.clone()
                }).unwrap();
                
                tx_clone.send((i, token)).unwrap();
            });
        }
        
        drop(tx);
        
        // Collect results
        let mut results = Vec::new();
        while let Ok((thread_id, token)) = rx.recv() {
            results.push((thread_id, token));
        }
        
        assert_eq!(results.len(), num_readers);
        for (_, token) in results {
            assert_eq!(token, "123456789.abcdef.ghijklmnop");
        }
    }

    #[test]
    fn test_concurrent_readers_and_writer() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        let manager = Arc::new(manager);
        
        let (tx, rx) = mpsc::channel();
        let num_readers = 5;
        
        // Spawn reader threads
        for i in 0..num_readers {
            let manager_clone = Arc::clone(&manager);
            let tx_clone = tx.clone();
            
            thread::spawn(move || {
                // Read the configuration multiple times
                for j in 0..3 {
                    let token = manager_clone.with_config(|config| {
                        thread::sleep(Duration::from_millis(5));
                        config.discord.token.clone()
                    }).unwrap();
                    
                    tx_clone.send(format!("reader-{}-{}: {}", i, j, token)).unwrap();
                }
            });
        }
        
        // Spawn a writer thread
        let manager_clone = Arc::clone(&manager);
        let tx_clone = tx.clone();
        thread::spawn(move || {
            thread::sleep(Duration::from_millis(25)); // Let readers start
            
            manager_clone.update_config(|config| {
                config.discord.token = "updated.by.writer".to_string();
            }).unwrap();
            
            tx_clone.send("writer: updated".to_string()).unwrap();
        });
        
        drop(tx);
        
        // Collect all results
        let mut results = Vec::new();
        while let Ok(result) = rx.recv() {
            results.push(result);
        }
        
        // Should have results from all readers and the writer
        assert!(results.len() > num_readers * 3);
        
        // Writer should have executed
        assert!(results.iter().any(|r| r.contains("writer: updated")));
    }

    #[test]
    fn test_manager_is_send_and_sync() {
        fn assert_send<T: Send>() {}
        fn assert_sync<T: Sync>() {}
        
        assert_send::<ConfigManager>();
        assert_sync::<ConfigManager>();
    }

    #[test]
    fn test_load_from_file() {
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        let yaml_content = "discord:\n  token: \"987654321.abcdef.testtoken\"\n  channels: [\"123456789\"]\n  max_concurrent_requests: 5\n  request_timeout_seconds: 30\ntautulli:\n  url: \"http://localhost:8181\"\n  api_key: \"file_api_key\"\n  timeout_seconds: 30\n  max_retries: 3\nscheduling:\n  auto_graph_cron: ~\n  cleanup_cron: ~\n  timezone: ~\n  enabled: false\ngraph:\n  width: 800\n  height: 600\n  background_color: \"#FFFFFF\"\n  primary_color: \"#FF0000\"\n  secondary_color: \"#00FF00\"\n  font_family: \"Arial\"\n  font_size: 12\n  show_grid: true\n  show_legend: true\n  max_data_points: 1000\ndatabase:\n  url: \"sqlite::memory:\"\n  max_connections: 10\n  connection_timeout_seconds: 30\n  query_timeout_seconds: 60\nlogging:\n  level: \"info\"\n  file: ~\n  colored: true\n  include_timestamps: true\n  include_location: false\n  max_file_size_mb: 10\n  max_files: 5";
        write!(temp_file, "{}", yaml_content).unwrap();
        
        let manager = ConfigManager::load_from_file(temp_file.path()).unwrap();
        
        let token = manager.with_config(|config| {
            config.discord.token.clone()
        }).unwrap();
        
        assert_eq!(token, "987654321.abcdef.testtoken");
    }

    #[test]
    fn test_get_shared() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        let shared = manager.get_shared();
        
        // Verify we can access through the shared Arc
        let guard = shared.read().unwrap();
        assert_eq!(guard.discord.token, "123456789.abcdef.ghijklmnop");
    }

    #[test]
    fn test_watcher_state_management() {
        let config = create_test_config();
        let manager = ConfigManager::new(config);
        
        // Initially not watching
        assert!(!manager.is_watching());
        assert!(manager.watched_path().is_none());
        
        // Create a temporary config file
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "discord:\n  token: test_token\ntautulli:\n  url: http://test\n  api_key: test").unwrap();
        
        // Start watching
        let result = manager.start_watching(temp_file.path());
        assert!(result.is_ok());
        assert!(manager.is_watching());
        assert_eq!(manager.watched_path().unwrap(), temp_file.path());
        
        // Try to start watching again (should fail)
        let result = manager.start_watching(temp_file.path());
        assert!(matches!(result, Err(ConfigManagerError::WatcherAlreadyRunning)));
        
        // Stop watching
        let result = manager.stop_watching();
        assert!(result.is_ok());
        assert!(!manager.is_watching());
        assert!(manager.watched_path().is_none());
        
        // Try to stop again (should fail)
        let result = manager.stop_watching();
        assert!(matches!(result, Err(ConfigManagerError::WatcherNotRunning)));
    }

    #[test]
    fn test_should_reload_for_event() {
        use notify::EventKind;
        use std::path::PathBuf;
        
        let config_path = PathBuf::from("/test/config.yaml");
        
        // Test modify event for our config file
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            )),
            paths: vec![config_path.clone()],
            attrs: Default::default(),
        };
        assert!(ConfigManager::should_reload_for_event(&event, &config_path));
        
        // Test modify event for different file
        let event = Event {
            kind: EventKind::Modify(notify::event::ModifyKind::Data(
                notify::event::DataChange::Content
            )),
            paths: vec![PathBuf::from("/test/other.yaml")],
            attrs: Default::default(),
        };
        assert!(!ConfigManager::should_reload_for_event(&event, &config_path));
        
        // Test create event for our config file
        let event = Event {
            kind: EventKind::Create(notify::event::CreateKind::File),
            paths: vec![config_path.clone()],
            attrs: Default::default(),
        };
        assert!(ConfigManager::should_reload_for_event(&event, &config_path));
        
        // Test remove event (should not trigger reload)
        let event = Event {
            kind: EventKind::Remove(notify::event::RemoveKind::File),
            paths: vec![config_path.clone()],
            attrs: Default::default(),
        };
        assert!(!ConfigManager::should_reload_for_event(&event, &config_path));
    }
} 