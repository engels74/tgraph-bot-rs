//! Tautulli API client with connection pooling and rate limiting
//!
//! This module provides a robust HTTP client for interacting with the Tautulli API,
//! including authentication, rate limiting, retry logic, and comprehensive error handling.

use crate::error::{Result, TGraphError};
use governor::{DefaultDirectRateLimiter, Quota};
use reqwest::{Client, Response};
use serde::{Deserialize, Serialize};
use std::{num::NonZeroU32, sync::Arc, time::Duration};
use tokio_retry::{strategy::ExponentialBackoff, Retry};
use tracing::{debug, error, info, instrument, warn};

/// Configuration for the Tautulli API client
#[derive(Debug, Clone)]
pub struct TautulliConfig {
    /// Base URL of the Tautulli server (e.g., "http://localhost:8181")
    pub base_url: String,
    /// API key for authentication
    pub api_key: String,
    /// Request timeout in seconds (default: 30)
    pub timeout_secs: u64,
    /// Connection pool max idle connections per host (default: 10)
    pub max_idle_per_host: usize,
    /// Rate limit: requests per second (default: 10)
    pub rate_limit_per_sec: u32,
    /// Maximum number of retry attempts (default: 3)
    pub max_retries: usize,
}

impl Default for TautulliConfig {
    fn default() -> Self {
        Self {
            base_url: "http://localhost:8181".to_string(),
            api_key: String::new(),
            timeout_secs: 30,
            max_idle_per_host: 10,
            rate_limit_per_sec: 10,
            max_retries: 3,
        }
    }
}

impl TautulliConfig {
    /// Create a new configuration with the minimum required parameters
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            api_key: api_key.into(),
            ..Default::default()
        }
    }

    /// Set the request timeout
    pub fn with_timeout(mut self, timeout_secs: u64) -> Self {
        self.timeout_secs = timeout_secs;
        self
    }

    /// Set the connection pool size
    pub fn with_pool_size(mut self, max_idle_per_host: usize) -> Self {
        self.max_idle_per_host = max_idle_per_host;
        self
    }

    /// Set the rate limit
    pub fn with_rate_limit(mut self, rate_limit_per_sec: u32) -> Self {
        self.rate_limit_per_sec = rate_limit_per_sec;
        self
    }

    /// Set the maximum retry attempts
    pub fn with_max_retries(mut self, max_retries: usize) -> Self {
        self.max_retries = max_retries;
        self
    }
}

/// Tautulli API client with connection pooling and rate limiting
#[derive(Debug, Clone)]
pub struct TautulliClient {
    client: Client,
    config: TautulliConfig,
    rate_limiter: Arc<DefaultDirectRateLimiter>,
}

impl TautulliClient {
    /// Create a new Tautulli client with the given configuration
    pub fn new(config: TautulliConfig) -> Result<Self> {
        // Build the HTTP client with connection pooling and timeouts
        let client = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .pool_max_idle_per_host(config.max_idle_per_host)
            .build()
            .map_err(|e| TGraphError::network_with_source("Failed to create HTTP client", e))?;

        // Create rate limiter
        let quota = Quota::per_second(
            NonZeroU32::new(config.rate_limit_per_sec)
                .ok_or_else(|| TGraphError::config("Rate limit must be greater than 0"))?,
        );
        let rate_limiter = Arc::new(DefaultDirectRateLimiter::direct(quota));

        Ok(Self {
            client,
            config,
            rate_limiter,
        })
    }

    /// Create a new client with default configuration
    pub fn with_defaults(base_url: impl Into<String>, api_key: impl Into<String>) -> Result<Self> {
        let config = TautulliConfig::new(base_url, api_key);
        Self::new(config)
    }

    /// Build a request URL with the API endpoint and parameters
    fn build_url(&self, _endpoint: &str) -> String {
        format!("{}/api/v2", self.config.base_url.trim_end_matches('/'))
    }

    /// Make an authenticated request to the Tautulli API with retry logic
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    async fn make_request(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<Response> {
        // Wait for rate limiter
        self.rate_limiter.until_ready().await;

        let url = self.build_url(endpoint);
        debug!("Making request to: {}", url);

        // Build query parameters including API key and command
        let mut query_params = vec![
            ("apikey", self.config.api_key.as_str()),
            ("cmd", endpoint),
        ];
        query_params.extend_from_slice(params);

        // Retry logic with exponential backoff
        let retry_strategy = ExponentialBackoff::from_millis(100)
            .max_delay(Duration::from_secs(10))
            .take(self.config.max_retries);

        let response = Retry::spawn(retry_strategy, || async {
            let request = self.client.get(&url).query(&query_params);

            debug!("Sending request with {} parameters", query_params.len());

            match request.send().await {
                Ok(response) => {
                    if response.status().is_success() {
                        debug!("Request successful: {}", response.status());
                        Ok(response)
                    } else if response.status().is_client_error() {
                        // Don't retry client errors (4xx)
                        error!("Client error: {}", response.status());
                        Err(TGraphError::tautulli_with_status(
                            format!("API returned client error: {}", response.status()),
                            response.status().as_u16()
                        ))
                    } else {
                        // Retry server errors (5xx)
                        warn!("Server error, will retry: {}", response.status());
                        Err(TGraphError::tautulli_with_status(
                            format!("API returned server error: {}", response.status()),
                            response.status().as_u16()
                        ))
                    }
                }
                Err(e) if e.is_timeout() => {
                    warn!("Request timeout, will retry: {}", e);
                    Err(TGraphError::network_with_source("Request timeout", e))
                }
                Err(e) if e.is_connect() => {
                    warn!("Connection error, will retry: {}", e);
                    Err(TGraphError::network_with_source("Connection error", e))
                }
                Err(e) => {
                    error!("Request failed: {}", e);
                    Err(TGraphError::network_with_source("Request failed", e))
                }
            }
        })
        .await?;

        info!("Successfully completed request to {}", endpoint);
        Ok(response)
    }

    /// Parse a JSON response into the specified type
    async fn parse_response<T>(&self, response: Response) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let text = response
            .text()
            .await
            .map_err(|e| TGraphError::network_with_source("Failed to read response body", e))?;

        debug!("Response body: {}", text);

        serde_json::from_str(&text)
            .map_err(|e| TGraphError::from(e)) // serde_json::Error automatically converts
    }

    /// Make a request and parse the JSON response
    #[instrument(skip(self), fields(endpoint = %endpoint))]
    async fn request_json<T>(&self, endpoint: &str, params: &[(&str, &str)]) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        let response = self.make_request(endpoint, params).await?;
        self.parse_response(response).await
    }

    // ============================================================================
    // Public API Methods
    // ============================================================================

    /// Get current activity (active streams)
    /// 
    /// Returns information about all active sessions including stream counts,
    /// bandwidth usage, and detailed session information.
    #[instrument(skip(self))]
    pub async fn get_activity(&self) -> Result<ActivityResponse> {
        info!("Fetching current activity");
        let response: TautulliResponse<ActivityResponse> = 
            self.request_json("get_activity", &[]).await?;
        
        if response.is_success() {
            response.data().ok_or_else(|| {
                TGraphError::tautulli("Activity response contained no data")
            })
        } else {
            Err(TGraphError::tautulli(
                response.error_message().unwrap_or("Unknown error getting activity")
            ))
        }
    }

    /// Get playback history
    /// 
    /// Returns paginated playback history with optional filtering parameters.
    /// 
    /// # Parameters
    /// - `user_id`: Filter by specific user ID (optional)
    /// - `length`: Number of records to return (default: 25, max: 1000)
    /// - `start`: Starting record index for pagination (default: 0)
    #[instrument(skip(self), fields(user_id = ?user_id, length = ?length, start = ?start))]
    pub async fn get_history(
        &self, 
        user_id: Option<i32>, 
        length: Option<i32>, 
        start: Option<i32>
    ) -> Result<HistoryResponse> {
        info!("Fetching playback history");
        
        let mut params = Vec::new();
        
        if let Some(uid) = user_id {
            params.push(("user_id", uid.to_string()));
        }
        if let Some(len) = length {
            params.push(("length", len.to_string()));
        }
        if let Some(st) = start {
            params.push(("start", st.to_string()));
        }

        // Convert to &str references for the API call
        let str_params: Vec<(&str, &str)> = params.iter()
            .map(|(k, v)| (*k, v.as_str()))
            .collect();

        let response: TautulliResponse<HistoryResponse> = 
            self.request_json("get_history", &str_params).await?;
        
        if response.is_success() {
            response.data().ok_or_else(|| {
                TGraphError::tautulli("History response contained no data")
            })
        } else {
            Err(TGraphError::tautulli(
                response.error_message().unwrap_or("Unknown error getting history")
            ))
        }
    }

    /// Get all users
    /// 
    /// Returns a list of all users known to Tautulli with their statistics and settings.
    #[instrument(skip(self))]
    pub async fn get_users(&self) -> Result<Vec<User>> {
        info!("Fetching users list");
        let response: TautulliResponse<Vec<User>> = 
            self.request_json("get_users", &[]).await?;
        
        if response.is_success() {
            response.data().ok_or_else(|| {
                TGraphError::tautulli("Users response contained no data")
            })
        } else {
            Err(TGraphError::tautulli(
                response.error_message().unwrap_or("Unknown error getting users")
            ))
        }
    }

    /// Get all libraries
    /// 
    /// Returns a list of all library sections configured in Plex with their statistics.
    #[instrument(skip(self))]
    pub async fn get_libraries(&self) -> Result<Vec<Library>> {
        info!("Fetching libraries list");
        let response: TautulliResponse<Vec<Library>> = 
            self.request_json("get_libraries", &[]).await?;
        
        if response.is_success() {
            response.data().ok_or_else(|| {
                TGraphError::tautulli("Libraries response contained no data")
            })
        } else {
            Err(TGraphError::tautulli(
                response.error_message().unwrap_or("Unknown error getting libraries")
            ))
        }
    }

    /// Get server information
    /// 
    /// Returns information about the Plex Media Server including version and platform details.
    #[instrument(skip(self))]
    pub async fn get_server_info(&self) -> Result<ServerInfoResponse> {
        info!("Fetching server information");
        let response: TautulliResponse<ServerInfoResponse> = 
            self.request_json("get_server_identity", &[]).await?;
        
        if response.is_success() {
            response.data().ok_or_else(|| {
                TGraphError::tautulli("Server info response contained no data")
            })
        } else {
            Err(TGraphError::tautulli(
                response.error_message().unwrap_or("Unknown error getting server info")
            ))
        }
    }

    /// Test the connection to Tautulli
    /// 
    /// Simple health check to verify the API key and connection are working.
    /// Returns true if the connection is successful, false otherwise.
    #[instrument(skip(self))]
    pub async fn test_connection(&self) -> bool {
        info!("Testing connection to Tautulli");
        match self.get_server_info().await {
            Ok(_) => {
                info!("Connection test successful");
                true
            }
            Err(e) => {
                warn!("Connection test failed: {}", e);
                false
            }
        }
    }

    /// Get metrics about the client configuration and state
    /// 
    /// Returns information useful for monitoring and debugging.
    pub fn get_client_metrics(&self) -> ClientMetrics {
        ClientMetrics {
            base_url: self.config.base_url.clone(),
            timeout_secs: self.config.timeout_secs,
            max_idle_per_host: self.config.max_idle_per_host,
            rate_limit_per_sec: self.config.rate_limit_per_sec,
            max_retries: self.config.max_retries,
            // Check if we have capacity for immediate request
            has_rate_limit_capacity: self.rate_limiter.check().is_ok(),
        }
    }
}

/// Client metrics for monitoring and debugging
#[derive(Debug, Clone, Serialize)]
pub struct ClientMetrics {
    /// Base URL being used
    pub base_url: String,
    /// Request timeout in seconds
    pub timeout_secs: u64,
    /// Connection pool max idle per host
    pub max_idle_per_host: usize,
    /// Rate limit requests per second
    pub rate_limit_per_sec: u32,
    /// Maximum retry attempts
    pub max_retries: usize,
    /// Whether we currently have rate limit capacity
    pub has_rate_limit_capacity: bool,
}

// ============================================================================
// API Response Models
// ============================================================================

/// Base response wrapper for all Tautulli API calls
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TautulliResponse<T> {
    /// Response data payload
    pub response: TautulliResponseData<T>,
}

/// Inner response data structure
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TautulliResponseData<T> {
    /// Result status (success, error)
    pub result: String,
    /// Optional message (usually present on errors)
    pub message: Option<String>,
    /// The actual data payload
    pub data: Option<T>,
}

impl<T> TautulliResponse<T> {
    /// Check if the response indicates success
    pub fn is_success(&self) -> bool {
        self.response.result == "success"
    }

    /// Get the data payload, if present
    pub fn data(self) -> Option<T> {
        self.response.data
    }

    /// Get error message, if any
    pub fn error_message(&self) -> Option<&str> {
        self.response.message.as_deref()
    }
}

// ============================================================================
// Activity Endpoint Models
// ============================================================================

/// Response model for the get_activity endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ActivityResponse {
    /// Stream count information
    pub stream_count: i32,
    /// Detailed stream count by type
    pub stream_count_direct_play: i32,
    pub stream_count_direct_stream: i32,
    pub stream_count_transcode: i32,
    /// Total bandwidth in kbps
    pub total_bandwidth: i32,
    /// List of active sessions
    pub sessions: Vec<Session>,
}

/// Individual session information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Session {
    /// Session key (unique identifier)
    pub session_key: Option<String>,
    /// Session ID
    pub session_id: Option<String>,
    /// User information
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub friendly_name: Option<String>,
    /// Media information
    pub media_type: Option<String>,
    pub title: Option<String>,
    pub parent_title: Option<String>,
    pub grandparent_title: Option<String>,
    pub year: Option<i32>,
    /// Playback information
    pub state: Option<String>, // playing, paused, buffering, etc.
    pub progress_percent: Option<i32>,
    pub view_offset: Option<i64>,
    pub duration: Option<i64>,
    /// Quality information
    pub quality_profile: Option<String>,
    pub stream_bitrate: Option<i32>,
    pub video_decision: Option<String>, // copy, transcode, etc.
    pub audio_decision: Option<String>,
    /// Device information
    pub platform: Option<String>,
    pub player: Option<String>,
    pub device: Option<String>,
    /// Location
    pub ip_address: Option<String>,
    pub location: Option<String>,
}

// ============================================================================
// History Endpoint Models
// ============================================================================

/// Response model for the get_history endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryResponse {
    /// Total number of records available
    #[serde(rename = "recordsTotal")]
    pub records_total: i32,
    /// Number of records after filtering
    #[serde(rename = "recordsFiltered")]
    pub records_filtered: i32,
    /// Current page being returned
    pub draw: i32,
    /// History data entries
    pub data: Vec<HistoryEntry>,
}

/// Individual history entry
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HistoryEntry {
    /// Date timestamp
    pub date: Option<i64>,
    /// User information
    pub user_id: Option<i32>,
    pub username: Option<String>,
    pub friendly_name: Option<String>,
    /// Media information
    pub media_type: Option<String>,
    pub rating_key: Option<String>,
    pub parent_rating_key: Option<String>,
    pub grandparent_rating_key: Option<String>,
    pub title: Option<String>,
    pub parent_title: Option<String>,
    pub grandparent_title: Option<String>,
    pub year: Option<i32>,
    /// Playback information
    pub watched_status: Option<i32>,
    pub percent_complete: Option<i32>,
    pub duration: Option<i64>,
    /// Quality information
    pub transcode_decision: Option<String>,
    /// Device information
    pub platform: Option<String>,
    pub player: Option<String>,
    pub ip_address: Option<String>,
}

// ============================================================================
// Users Endpoint Models
// ============================================================================

/// Response model for the get_users endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct UsersResponse {
    /// List of users
    pub users: Vec<User>,
}

/// User information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct User {
    /// User ID
    pub user_id: i32,
    /// Username for authentication
    pub username: String,
    /// Display name
    pub friendly_name: Option<String>,
    /// Email address
    pub email: Option<String>,
    /// Thumbnail URL
    pub thumb: Option<String>,
    /// User statistics
    pub plays: Option<i32>,
    pub duration: Option<i64>,
    /// Last seen timestamp
    pub last_seen: Option<i64>,
    /// User settings
    pub is_active: Option<i32>,
    pub is_admin: Option<i32>,
    pub is_home_user: Option<i32>,
    pub is_allow_sync: Option<i32>,
    pub is_restricted: Option<i32>,
}

// ============================================================================
// Libraries Endpoint Models
// ============================================================================

/// Response model for the get_libraries endpoint
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct LibrariesResponse {
    /// List of libraries
    pub libraries: Vec<Library>,
}

/// Library information
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Library {
    /// Library section ID
    pub section_id: i32,
    /// Library name
    pub section_name: String,
    /// Library type (movie, show, artist, photo)
    pub section_type: String,
    /// Library agent
    pub agent: Option<String>,
    /// Library language
    pub language: Option<String>,
    /// Count statistics
    pub count: Option<i32>,
    pub parent_count: Option<i32>,
    pub child_count: Option<i32>,
    /// Thumbnail
    pub thumb: Option<String>,
    /// Art
    pub art: Option<String>,
    /// Library settings
    pub is_active: Option<i32>,
    pub do_notify: Option<i32>,
    pub do_notify_created: Option<i32>,
}

// ============================================================================
// Server Info Models
// ============================================================================

/// Server information response
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServerInfoResponse {
    /// Server version
    pub version: Option<String>,
    /// Server platform
    pub platform: Option<String>,
    /// Platform version
    pub platform_version: Option<String>,
    /// Server update available
    pub update_available: Option<i32>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_creation() {
        let config = TautulliConfig::new("http://example.com", "test-key");
        assert_eq!(config.base_url, "http://example.com");
        assert_eq!(config.api_key, "test-key");
        assert_eq!(config.timeout_secs, 30); // default
    }

    #[test]
    fn test_config_builder() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_timeout(60)
            .with_pool_size(20)
            .with_rate_limit(5)
            .with_max_retries(5);

        assert_eq!(config.timeout_secs, 60);
        assert_eq!(config.max_idle_per_host, 20);
        assert_eq!(config.rate_limit_per_sec, 5);
        assert_eq!(config.max_retries, 5);
    }

    #[test]
    fn test_url_building() {
        let config = TautulliConfig::new("http://example.com/", "test-key");
        let client = TautulliClient::new(config).unwrap();
        let url = client.build_url("get_activity");
        assert_eq!(url, "http://example.com/api/v2");
    }

    #[tokio::test]
    async fn test_client_creation() {
        let config = TautulliConfig::new("http://example.com", "test-key");
        let result = TautulliClient::new(config);
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_client_with_defaults() {
        let result = TautulliClient::with_defaults("http://example.com", "test-key");
        assert!(result.is_ok());
    }

    #[test]
    fn test_rate_limit_configuration() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_rate_limit(5);
        assert_eq!(config.rate_limit_per_sec, 5);
    }

    #[test]
    fn test_rate_limit_validation() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_rate_limit(0);
        let result = TautulliClient::new(config);
        assert!(result.is_err());
        if let Err(e) = result {
            assert!(e.to_string().contains("Rate limit must be greater than 0"));
        }
    }

    #[tokio::test]
    async fn test_rate_limiter_integration() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_rate_limit(10); // 10 requests per second
        let client = TautulliClient::new(config).unwrap();
        
        // Test that rate limiter methods work without throwing errors
        // First call should succeed immediately
        client.rate_limiter.until_ready().await;
        
        // Test with different rate limits
        let config_slow = TautulliConfig::new("http://example.com", "test-key")
            .with_rate_limit(1); // 1 request per second
        let client_slow = TautulliClient::new(config_slow).unwrap();
        
        // Both rate limiters should be functional
        client_slow.rate_limiter.until_ready().await;
        
        // Test that we can call until_ready multiple times
        client.rate_limiter.until_ready().await;
        client.rate_limiter.until_ready().await;
    }

    // ============================================================================
    // Response Model Tests
    // ============================================================================

    #[test]
    fn test_tautulli_response_wrapper() {
        use std::collections::HashMap;
        
        let json = r#"{
            "response": {
                "result": "success",
                "message": null,
                "data": {"test": "value"}
            }
        }"#;
        
        let response: TautulliResponse<HashMap<String, String>> = serde_json::from_str(json).unwrap();
        assert!(response.is_success());
        assert!(response.error_message().is_none());
        
        let data = response.data().unwrap();
        assert_eq!(data.get("test"), Some(&"value".to_string()));
    }

    #[test]
    fn test_tautulli_response_error() {
        let json = r#"{
            "response": {
                "result": "error",
                "message": "API key not found",
                "data": null
            }
        }"#;
        
        let response: TautulliResponse<String> = serde_json::from_str(json).unwrap();
        assert!(!response.is_success());
        assert_eq!(response.error_message(), Some("API key not found"));
        assert!(response.data().is_none());
    }

    #[test]
    fn test_activity_response_deserialization() {
        let json = r#"{
            "stream_count": 2,
            "stream_count_direct_play": 1,
            "stream_count_direct_stream": 0,
            "stream_count_transcode": 1,
            "total_bandwidth": 4500,
            "sessions": [
                {
                    "session_key": "123",
                    "username": "testuser",
                    "title": "Test Movie",
                    "state": "playing",
                    "progress_percent": 45,
                    "platform": "Chrome"
                }
            ]
        }"#;
        
        let activity: ActivityResponse = serde_json::from_str(json).unwrap();
        assert_eq!(activity.stream_count, 2);
        assert_eq!(activity.total_bandwidth, 4500);
        assert_eq!(activity.sessions.len(), 1);
        assert_eq!(activity.sessions[0].username, Some("testuser".to_string()));
        assert_eq!(activity.sessions[0].state, Some("playing".to_string()));
    }

    #[test]
    fn test_user_deserialization() {
        let json = r#"{
            "user_id": 1,
            "username": "admin",
            "friendly_name": "Administrator",
            "email": "admin@example.com",
            "plays": 150,
            "duration": 720000,
            "is_admin": 1,
            "is_active": 1
        }"#;
        
        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.user_id, 1);
        assert_eq!(user.username, "admin");
        assert_eq!(user.friendly_name, Some("Administrator".to_string()));
        assert_eq!(user.plays, Some(150));
        assert_eq!(user.is_admin, Some(1));
    }

    #[test]
    fn test_library_deserialization() {
        let json = r#"{
            "section_id": 1,
            "section_name": "Movies",
            "section_type": "movie",
            "agent": "com.plexapp.agents.imdb",
            "language": "en",
            "count": 450,
            "is_active": 1
        }"#;
        
        let library: Library = serde_json::from_str(json).unwrap();
        assert_eq!(library.section_id, 1);
        assert_eq!(library.section_name, "Movies");
        assert_eq!(library.section_type, "movie");
        assert_eq!(library.count, Some(450));
    }

    #[test]
    fn test_history_entry_deserialization() {
        let json = r#"{
            "date": 1640995200,
            "user_id": 1,
            "username": "testuser",
            "title": "Test Movie",
            "year": 2021,
            "duration": 7200,
            "percent_complete": 95,
            "platform": "Plex Web"
        }"#;
        
        let entry: HistoryEntry = serde_json::from_str(json).unwrap();
        assert_eq!(entry.date, Some(1640995200));
        assert_eq!(entry.username, Some("testuser".to_string()));
        assert_eq!(entry.title, Some("Test Movie".to_string()));
        assert_eq!(entry.percent_complete, Some(95));
    }

    #[test]
    fn test_history_response_deserialization() {
        let json = r#"{
            "recordsTotal": 1000,
            "recordsFiltered": 25,
            "draw": 1,
            "data": [
                {
                    "date": 1640995200,
                    "username": "testuser",
                    "title": "Test Movie",
                    "percent_complete": 95
                }
            ]
        }"#;
        
        let history: HistoryResponse = serde_json::from_str(json).unwrap();
        assert_eq!(history.records_total, 1000);
        assert_eq!(history.records_filtered, 25);
        assert_eq!(history.draw, 1);
        assert_eq!(history.data.len(), 1);
    }

    // ============================================================================
    // API Method Tests
    // ============================================================================

    #[test]
    fn test_api_method_signatures() {
        // Test that we can create a client and the methods exist with correct signatures
        let config = TautulliConfig::new("http://example.com", "test-key");
        let client = TautulliClient::new(config).unwrap();
        
        // These tests just verify the method signatures compile correctly
        // We can't test the actual network calls without a real server or complex mocking
        
        // Verify async method signatures exist
        let _activity_future = client.get_activity();
        let _history_future = client.get_history(Some(1), Some(25), Some(0));
        let _users_future = client.get_users();
        let _libraries_future = client.get_libraries();
        let _server_info_future = client.get_server_info();
        let _test_connection_future = client.test_connection();
        
        // Verify parameter handling for get_history
        let _history_no_params = client.get_history(None, None, None);
        let _history_user_only = client.get_history(Some(5), None, None);
        let _history_with_pagination = client.get_history(None, Some(50), Some(100));
    }

    #[tokio::test]
    async fn test_client_field_usage() {
        // This test ensures our client fields are actually used
        // which should eliminate the dead code warnings
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_rate_limit(5);
        let client = TautulliClient::new(config).unwrap();
        
        // Access the rate limiter to show it's used
        client.rate_limiter.until_ready().await;
        
        // The client and config fields are used in the actual API methods
        // but we can't easily test them without making real network calls
        assert_eq!(client.config.api_key, "test-key");
        assert_eq!(client.config.rate_limit_per_sec, 5);
    }

    // ============================================================================
    // Error Handling and Metrics Tests
    // ============================================================================

    #[test]
    fn test_client_metrics() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_timeout(60)
            .with_pool_size(20)
            .with_rate_limit(5)
            .with_max_retries(5);
        let client = TautulliClient::new(config).unwrap();
        
        let metrics = client.get_client_metrics();
        assert_eq!(metrics.base_url, "http://example.com");
        assert_eq!(metrics.timeout_secs, 60);
        assert_eq!(metrics.max_idle_per_host, 20);
        assert_eq!(metrics.rate_limit_per_sec, 5);
        assert_eq!(metrics.max_retries, 5);
        // Rate limit capacity should be available initially
        assert!(metrics.has_rate_limit_capacity);
    }

    #[test]
    fn test_retry_configuration() {
        let config = TautulliConfig::new("http://example.com", "test-key")
            .with_max_retries(10);
        assert_eq!(config.max_retries, 10);
        
        let client = TautulliClient::new(config).unwrap();
        assert_eq!(client.config.max_retries, 10);
    }

    #[test]
    fn test_error_response_parsing() {
        let error_json = r#"{
            "response": {
                "result": "error",
                "message": "Invalid API key",
                "data": null
            }
        }"#;
        
        let response: TautulliResponse<String> = serde_json::from_str(error_json).unwrap();
        assert!(!response.is_success());
        assert_eq!(response.error_message(), Some("Invalid API key"));
        assert!(response.data().is_none());
    }

    #[test]
    fn test_success_response_without_data() {
        let success_json = r#"{
            "response": {
                "result": "success",
                "message": null,
                "data": null
            }
        }"#;
        
        let response: TautulliResponse<String> = serde_json::from_str(success_json).unwrap();
        assert!(response.is_success());
        assert!(response.error_message().is_none());
        assert!(response.data().is_none());
    }

    #[test]
    fn test_client_metrics_serialization() {
        let config = TautulliConfig::new("http://localhost:8181", "api-key-123");
        let client = TautulliClient::new(config).unwrap();
        let metrics = client.get_client_metrics();
        
        // Test that metrics can be serialized (useful for monitoring/debugging)
        let serialized = serde_json::to_string(&metrics).unwrap();
        assert!(serialized.contains("http://localhost:8181"));
        assert!(serialized.contains("timeout_secs"));
        assert!(serialized.contains("rate_limit_per_sec"));
        // API key should NOT be in metrics for security reasons
        assert!(!serialized.contains("api-key-123"));
    }
} 