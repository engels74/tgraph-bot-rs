//! Enhanced timezone support for global scheduling
//!
//! This module provides comprehensive timezone handling for the scheduling system,
//! including DST transitions, timezone conversions, and global deployment support.

use anyhow::Result;
use chrono::{DateTime, Offset, TimeZone, Utc};
use chrono_tz::{OffsetComponents, Tz, TZ_VARIANTS};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use tracing::{debug, info, warn};

/// Timezone configuration for a schedule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneConfig {
    /// IANA timezone identifier (e.g., "America/New_York")
    pub timezone: String,
    /// Whether to automatically adjust for DST
    pub auto_dst: bool,
    /// Fallback timezone if the primary timezone is invalid
    pub fallback_timezone: Option<String>,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self {
            timezone: "UTC".to_string(),
            auto_dst: true,
            fallback_timezone: Some("UTC".to_string()),
        }
    }
}

/// Timezone information with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezoneInfo {
    /// IANA timezone identifier
    pub name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Current UTC offset in seconds
    pub utc_offset_seconds: i32,
    /// Whether the timezone is currently observing DST
    pub is_dst: bool,
    /// Abbreviation (e.g., "EST", "PDT")
    pub abbreviation: String,
    /// Region/continent
    pub region: String,
}

/// DST transition information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DstTransition {
    /// When the transition occurs (in the local timezone)
    pub transition_time: DateTime<Utc>,
    /// UTC offset before the transition
    pub offset_before: i32,
    /// UTC offset after the transition
    pub offset_after: i32,
    /// Whether this is a spring forward (true) or fall back (false)
    pub is_spring_forward: bool,
    /// Timezone this transition applies to
    pub timezone: String,
}

/// Timezone manager for handling global timezone operations
pub struct TimezoneManager {
    /// Cache of timezone information
    timezone_cache: HashMap<String, TimezoneInfo>,
    /// Supported timezone aliases
    timezone_aliases: HashMap<String, String>,
}

impl TimezoneManager {
    /// Create a new timezone manager
    pub fn new() -> Self {
        info!("Initializing timezone manager");
        
        let mut manager = Self {
            timezone_cache: HashMap::new(),
            timezone_aliases: HashMap::new(),
        };

        // Initialize timezone cache and aliases
        manager.initialize_timezone_data();
        
        info!("Timezone manager initialized with {} timezones", manager.timezone_cache.len());
        manager
    }

    /// Initialize timezone data and aliases
    fn initialize_timezone_data(&mut self) {
        // Add common timezone aliases
        self.timezone_aliases.insert("EST".to_string(), "America/New_York".to_string());
        self.timezone_aliases.insert("PST".to_string(), "America/Los_Angeles".to_string());
        self.timezone_aliases.insert("CST".to_string(), "America/Chicago".to_string());
        self.timezone_aliases.insert("MST".to_string(), "America/Denver".to_string());
        self.timezone_aliases.insert("GMT".to_string(), "UTC".to_string());
        self.timezone_aliases.insert("BST".to_string(), "Europe/London".to_string());
        self.timezone_aliases.insert("CET".to_string(), "Europe/Paris".to_string());
        self.timezone_aliases.insert("JST".to_string(), "Asia/Tokyo".to_string());
        self.timezone_aliases.insert("AEST".to_string(), "Australia/Sydney".to_string());

        // Populate timezone cache with information for all supported timezones
        for tz in TZ_VARIANTS {
            let now = Utc::now();
            let local_time = tz.from_utc_datetime(&now.naive_utc());
            let tz_name = tz.name();

            let timezone_info = TimezoneInfo {
                name: tz_name.to_string(),
                display_name: self.generate_display_name(tz_name),
                utc_offset_seconds: local_time.offset().fix().local_minus_utc(),
                is_dst: local_time.offset().dst_offset().num_seconds() != 0,
                abbreviation: format!("{}", local_time.format("%Z")),
                region: self.extract_region(tz_name),
            };

            self.timezone_cache.insert(tz_name.to_string(), timezone_info);
        }
    }

    /// Generate a human-readable display name for a timezone
    fn generate_display_name(&self, timezone: &str) -> String {
        // Convert "America/New_York" to "America - New York"
        timezone.replace('_', " ").replace('/', " - ")
    }

    /// Extract the region from a timezone name
    fn extract_region(&self, timezone: &str) -> String {
        timezone.split('/').next().unwrap_or("Unknown").to_string()
    }

    /// Validate a timezone string
    pub fn validate_timezone(&self, timezone: &str) -> Result<()> {
        // Check if it's a direct timezone name
        if self.timezone_cache.contains_key(timezone) {
            return Ok(());
        }

        // Check if it's an alias
        if self.timezone_aliases.contains_key(timezone) {
            return Ok(());
        }

        // Try to parse it directly
        if timezone.parse::<Tz>().is_ok() {
            return Ok(());
        }

        Err(anyhow::anyhow!("Invalid timezone: {}", timezone))
    }

    /// Resolve a timezone string to its canonical IANA name
    pub fn resolve_timezone(&self, timezone: &str) -> Result<String> {
        // Check if it's already a canonical name
        if self.timezone_cache.contains_key(timezone) {
            return Ok(timezone.to_string());
        }

        // Check if it's an alias
        if let Some(canonical) = self.timezone_aliases.get(timezone) {
            return Ok(canonical.clone());
        }

        // Try to parse it directly
        if timezone.parse::<Tz>().is_ok() {
            return Ok(timezone.to_string());
        }

        Err(anyhow::anyhow!("Cannot resolve timezone: {}", timezone))
    }

    /// Get timezone information
    pub fn get_timezone_info(&self, timezone: &str) -> Result<TimezoneInfo> {
        let canonical_name = self.resolve_timezone(timezone)?;
        
        self.timezone_cache.get(&canonical_name)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Timezone info not found: {}", canonical_name))
    }

    /// Convert a UTC time to a specific timezone
    pub fn convert_utc_to_timezone(&self, utc_time: DateTime<Utc>, timezone: &str) -> Result<DateTime<Tz>> {
        let canonical_name = self.resolve_timezone(timezone)?;
        let tz: Tz = canonical_name.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse timezone {}: {}", canonical_name, e))?;

        Ok(utc_time.with_timezone(&tz))
    }

    /// Convert a timezone-specific time to UTC
    pub fn convert_timezone_to_utc(&self, local_time: DateTime<Tz>) -> DateTime<Utc> {
        local_time.with_timezone(&Utc)
    }

    /// Get the current time in a specific timezone
    pub fn now_in_timezone(&self, timezone: &str) -> Result<DateTime<Tz>> {
        let utc_now = Utc::now();
        self.convert_utc_to_timezone(utc_now, timezone)
    }

    /// Check if a timezone is currently observing DST
    pub fn is_dst_active(&self, timezone: &str) -> Result<bool> {
        let timezone_info = self.get_timezone_info(timezone)?;
        Ok(timezone_info.is_dst)
    }

    /// Get upcoming DST transitions for a timezone
    pub fn get_upcoming_dst_transitions(&self, timezone: &str, months_ahead: u32) -> Result<Vec<DstTransition>> {
        let canonical_name = self.resolve_timezone(timezone)?;
        let tz: Tz = canonical_name.parse()
            .map_err(|e| anyhow::anyhow!("Failed to parse timezone {}: {}", canonical_name, e))?;
        
        let mut transitions = Vec::new();
        let start_time = Utc::now();
        let end_time = start_time + chrono::Duration::days((months_ahead * 30) as i64);

        // This is a simplified implementation - in practice, you'd want to use
        // a more sophisticated DST transition detection algorithm
        let mut current_time = start_time;
        let mut last_offset = None;

        while current_time < end_time {
            let local_time = tz.from_utc_datetime(&current_time.naive_utc());
            let current_offset = local_time.offset().fix().local_minus_utc();

            if let Some(prev_offset) = last_offset {
                if current_offset != prev_offset {
                    // DST transition detected
                    let transition = DstTransition {
                        transition_time: current_time,
                        offset_before: prev_offset,
                        offset_after: current_offset,
                        is_spring_forward: current_offset > prev_offset,
                        timezone: canonical_name.clone(),
                    };
                    transitions.push(transition);
                }
            }

            last_offset = Some(current_offset);
            current_time = current_time + chrono::Duration::days(1);
        }

        debug!("Found {} DST transitions for timezone {} in next {} months", 
               transitions.len(), canonical_name, months_ahead);
        
        Ok(transitions)
    }

    /// Get all available timezones grouped by region
    pub fn get_timezones_by_region(&self) -> HashMap<String, Vec<TimezoneInfo>> {
        let mut regions: HashMap<String, Vec<TimezoneInfo>> = HashMap::new();

        for timezone_info in self.timezone_cache.values() {
            let region = timezone_info.region.clone();
            regions.entry(region).or_insert_with(Vec::new).push(timezone_info.clone());
        }

        // Sort timezones within each region
        for timezone_list in regions.values_mut() {
            timezone_list.sort_by(|a, b| a.name.cmp(&b.name));
        }

        regions
    }

    /// Search for timezones by name or region
    pub fn search_timezones(&self, query: &str) -> Vec<TimezoneInfo> {
        let query_lower = query.to_lowercase();
        
        self.timezone_cache.values()
            .filter(|tz| {
                tz.name.to_lowercase().contains(&query_lower) ||
                tz.display_name.to_lowercase().contains(&query_lower) ||
                tz.region.to_lowercase().contains(&query_lower) ||
                tz.abbreviation.to_lowercase().contains(&query_lower)
            })
            .cloned()
            .collect()
    }

    /// Get timezone suggestions for a given UTC offset
    pub fn get_timezones_for_offset(&self, utc_offset_hours: i32) -> Vec<TimezoneInfo> {
        let target_offset_seconds = utc_offset_hours * 3600;
        
        self.timezone_cache.values()
            .filter(|tz| tz.utc_offset_seconds == target_offset_seconds)
            .cloned()
            .collect()
    }

    /// Validate and normalize a timezone configuration
    pub fn validate_timezone_config(&self, config: &TimezoneConfig) -> Result<TimezoneConfig> {
        let mut validated_config = config.clone();

        // Validate primary timezone
        match self.resolve_timezone(&config.timezone) {
            Ok(canonical) => {
                validated_config.timezone = canonical;
            }
            Err(_) => {
                warn!("Invalid primary timezone: {}", config.timezone);
                
                // Try fallback timezone
                if let Some(fallback) = &config.fallback_timezone {
                    match self.resolve_timezone(fallback) {
                        Ok(canonical) => {
                            warn!("Using fallback timezone: {}", canonical);
                            validated_config.timezone = canonical;
                        }
                        Err(_) => {
                            warn!("Invalid fallback timezone: {}, using UTC", fallback);
                            validated_config.timezone = "UTC".to_string();
                        }
                    }
                } else {
                    warn!("No fallback timezone specified, using UTC");
                    validated_config.timezone = "UTC".to_string();
                }
            }
        }

        // Validate fallback timezone if specified
        if let Some(fallback) = &validated_config.fallback_timezone {
            if self.resolve_timezone(fallback).is_err() {
                warn!("Invalid fallback timezone: {}, removing", fallback);
                validated_config.fallback_timezone = Some("UTC".to_string());
            }
        }

        Ok(validated_config)
    }
}

impl Default for TimezoneManager {
    fn default() -> Self {
        Self::new()
    }
}
