//! Common type definitions and newtype wrappers for domain modeling.
//!
//! This module provides zero-cost abstractions for domain-specific types,
//! ensuring type safety and preventing value confusion at compile time.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

// ============================================================================
// ID Types - Zero-cost wrappers for various identifier types
// ============================================================================

/// A Discord channel ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ChannelId(pub u64);

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ChannelId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<ChannelId> for u64 {
    fn from(id: ChannelId) -> Self {
        id.0
    }
}

/// A Discord user ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(pub u64);

impl fmt::Display for UserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for UserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<UserId> for u64 {
    fn from(id: UserId) -> Self {
        id.0
    }
}

/// A Tautulli user ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TautulliUserId(pub u64);

impl fmt::Display for TautulliUserId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for TautulliUserId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<TautulliUserId> for u64 {
    fn from(id: TautulliUserId) -> Self {
        id.0
    }
}

/// A media item ID from Tautulli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MediaItemId(pub u64);

impl fmt::Display for MediaItemId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for MediaItemId {
    fn from(id: u64) -> Self {
        Self(id)
    }
}

impl From<MediaItemId> for u64 {
    fn from(id: MediaItemId) -> Self {
        id.0
    }
}

// ============================================================================
// Session and String-based Types
// ============================================================================

/// A validated session ID with length and character constraints.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

impl SessionId {
    /// Create a new SessionId without validation (for trusted sources).
    pub fn new(id: String) -> Self {
        Self(id)
    }

    /// Try to create a new SessionId with validation.
    pub fn try_new(id: String) -> std::result::Result<Self, TGraphError> {
        if id.is_empty() {
            return Err(TGraphError::Config(
                "Session ID cannot be empty".to_string(),
            ));
        }

        if id.len() > 255 {
            return Err(TGraphError::Config(
                "Session ID cannot exceed 255 characters".to_string(),
            ));
        }

        // Check for invalid characters (spaces, tabs, newlines)
        if id.chars().any(|c| c.is_whitespace()) {
            return Err(TGraphError::Config(
                "Session ID cannot contain whitespace characters".to_string(),
            ));
        }

        Ok(Self(id))
    }

    /// Get the session ID as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Time-related Types
// ============================================================================

/// A type-safe timestamp wrapper around chrono::DateTime<Utc>.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Timestamp(DateTime<Utc>);

impl Timestamp {
    /// Create a new timestamp representing the current time.
    pub fn now() -> Self {
        Self(Utc::now())
    }

    /// Create a timestamp from a Unix timestamp.
    pub fn from_timestamp(timestamp: i64) -> Self {
        let datetime = DateTime::from_timestamp(timestamp, 0).unwrap_or_else(|| Utc::now());
        Self(datetime)
    }

    /// Get the Unix timestamp.
    pub fn timestamp(&self) -> i64 {
        self.0.timestamp()
    }

    /// Get the year component.
    pub fn year(&self) -> i32 {
        self.0.format("%Y").to_string().parse().unwrap_or(2024)
    }
}

impl fmt::Display for Timestamp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.format("%Y-%m-%d %H:%M:%S UTC"))
    }
}

impl From<DateTime<Utc>> for Timestamp {
    fn from(datetime: DateTime<Utc>) -> Self {
        Self(datetime)
    }
}

impl From<Timestamp> for DateTime<Utc> {
    fn from(timestamp: Timestamp) -> Self {
        timestamp.0
    }
}

/// A validated duration wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct Duration(u32);

impl Duration {
    /// Try to create a new duration from seconds.
    pub fn try_new(seconds: u32) -> std::result::Result<Self, TGraphError> {
        // Duration can be zero for edge cases (e.g., very short clips)
        Ok(Self(seconds))
    }

    /// Get the duration in seconds.
    pub fn seconds(&self) -> u32 {
        self.0
    }

    /// Get the hours component.
    pub fn hours(&self) -> u32 {
        self.0 / 3600
    }

    /// Get the minutes component (without hours).
    pub fn minutes(&self) -> u32 {
        (self.0 % 3600) / 60
    }

    /// Get the remaining seconds component (without hours and minutes).
    pub fn remaining_seconds(&self) -> u32 {
        self.0 % 60
    }
}

impl fmt::Display for Duration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let hours = self.hours();
        let minutes = self.minutes();
        let seconds = self.remaining_seconds();

        if hours > 0 {
            write!(f, "{}h {}m {}s", hours, minutes, seconds)
        } else if minutes > 0 {
            write!(f, "{}m {}s", minutes, seconds)
        } else {
            write!(f, "{}s", seconds)
        }
    }
}

// ============================================================================
// Count and Metric Types
// ============================================================================

/// A validated play count wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PlayCount(u32);

impl PlayCount {
    /// Try to create a new play count.
    pub fn try_new(count: u32) -> std::result::Result<Self, TGraphError> {
        // Zero is valid for media that hasn't been played yet
        Ok(Self(count))
    }

    /// Get the play count value.
    pub fn value(&self) -> u32 {
        self.0
    }

    /// Add to the play count, returning a new PlayCount.
    pub fn add(&self, other: u32) -> Self {
        Self(self.0.saturating_add(other))
    }
}

impl fmt::Display for PlayCount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

// ============================================================================
// Enum Types for Domain Modeling
// ============================================================================

/// Enumeration of media types supported by Tautulli.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MediaType {
    /// Movie content
    Movie,
    /// TV show/episode content
    TvShow,
    /// Music/audio content
    Music,
}

impl MediaType {
    /// Get the string representation for Tautulli API.
    pub fn as_str(&self) -> &'static str {
        match self {
            MediaType::Movie => "movie",
            MediaType::TvShow => "show",
            MediaType::Music => "track",
        }
    }
}

impl fmt::Display for MediaType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for MediaType {
    type Err = TGraphError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s {
            "movie" => Ok(MediaType::Movie),
            "show" => Ok(MediaType::TvShow),
            "track" => Ok(MediaType::Music),
            _ => Err(TGraphError::Config(format!("Invalid media type: {}", s))),
        }
    }
}

/// Enumeration of graph types supported by the bot.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GraphType {
    /// Daily play count over time
    DailyPlayCount,
    /// Play count by day of week
    PlayCountByDayOfWeek,
    /// Play count by hour of day
    PlayCountByHourOfDay,
    /// Top 10 platforms
    Top10Platforms,
    /// Top 10 users
    Top10Users,
    /// Play count by month
    PlayCountByMonth,
}

impl GraphType {
    /// Get the string representation for configuration.
    pub fn as_str(&self) -> &'static str {
        match self {
            GraphType::DailyPlayCount => "daily_play_count",
            GraphType::PlayCountByDayOfWeek => "play_count_by_dayofweek",
            GraphType::PlayCountByHourOfDay => "play_count_by_hourofday",
            GraphType::Top10Platforms => "top_10_platforms",
            GraphType::Top10Users => "top_10_users",
            GraphType::PlayCountByMonth => "play_count_by_month",
        }
    }
}

impl fmt::Display for GraphType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// ============================================================================
// Error Types and Result Aliases
// ============================================================================

/// Application-wide error type.
#[derive(thiserror::Error, Debug)]
pub enum TGraphError {
    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Discord API error.
    #[error("Discord API error: {0}")]
    Discord(String),

    /// Tautulli API error.
    #[error("Tautulli API error: {0}")]
    Tautulli(String),

    /// Graph generation error.
    #[error("Graph generation error: {0}")]
    Graph(String),

    /// I/O error.
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    Serialization(String),
}

/// Utilities-specific error type.
#[derive(thiserror::Error, Debug)]
pub enum UtilsError {
    /// Time range validation error.
    #[error("Invalid time range: start {start} must be before end {end}")]
    InvalidTimeRange { start: String, end: String },

    /// Timezone conversion error.
    #[error("Invalid timezone: {timezone}")]
    InvalidTimezone { timezone: String },

    /// Timeout error.
    #[error("Operation timed out after {duration_ms}ms")]
    Timeout { duration_ms: u64 },

    /// Retry operation failed.
    #[error("Operation failed after {attempts} attempts. Last error: {last_error}")]
    RetryFailed { attempts: u32, last_error: String },

    /// Unicode processing error.
    #[error("Unicode processing error: {message}")]
    Unicode { message: String },
}

/// Common result type for the application.
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

// ============================================================================
// Tests for zero-cost abstractions verification
// ============================================================================

#[cfg(test)]
mod zero_cost_tests {
    use super::*;
    use std::mem;

    #[test]
    fn test_id_types_are_zero_cost() {
        // These should have the same memory footprint as their wrapped types
        assert_eq!(mem::size_of::<ChannelId>(), mem::size_of::<u64>());
        assert_eq!(mem::size_of::<UserId>(), mem::size_of::<u64>());
        assert_eq!(mem::size_of::<TautulliUserId>(), mem::size_of::<u64>());
        assert_eq!(mem::size_of::<MediaItemId>(), mem::size_of::<u64>());
    }

    #[test]
    fn test_numeric_types_are_zero_cost() {
        assert_eq!(mem::size_of::<PlayCount>(), mem::size_of::<u32>());
        assert_eq!(mem::size_of::<Duration>(), mem::size_of::<u32>());
    }

    #[test]
    fn test_timestamp_wrapper_minimal_overhead() {
        // Timestamp should have minimal overhead over DateTime<Utc>
        assert_eq!(mem::size_of::<Timestamp>(), mem::size_of::<DateTime<Utc>>());
    }
}
