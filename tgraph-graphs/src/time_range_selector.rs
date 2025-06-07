//! Time range selection utilities and preset management

use crate::{DateRange, TimeRangePreset, FilterConfig, ComparisonConfig, ComparisonPeriod};
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use tgraph_common::Result;

/// Time range selector with preset management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRangeSelector {
    /// Currently selected preset
    pub selected_preset: Option<TimeRangePreset>,
    /// Custom date range if using custom preset
    pub custom_range: Option<DateRange>,
    /// Available presets
    pub available_presets: Vec<TimeRangePreset>,
    /// Quick access buttons for common ranges
    pub quick_ranges: Vec<QuickRange>,
}

/// Quick range button configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuickRange {
    /// Display label for the button
    pub label: String,
    /// Time range preset this button represents
    pub preset: TimeRangePreset,
    /// Whether this range is currently active
    pub active: bool,
}

/// Time range selection result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRangeSelection {
    /// Selected date range
    pub date_range: DateRange,
    /// Display label for the selection
    pub label: String,
    /// Whether this is a custom range
    pub is_custom: bool,
    /// Associated preset if applicable
    pub preset: Option<TimeRangePreset>,
}

/// Time range validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRangeValidation {
    /// Whether the range is valid
    pub is_valid: bool,
    /// Validation error message if invalid
    pub error_message: Option<String>,
    /// Suggested corrections if any
    pub suggestions: Vec<String>,
    /// Warning messages for potentially problematic ranges
    pub warnings: Vec<String>,
}

impl TimeRangeSelector {
    /// Create a new time range selector with default presets
    pub fn new() -> Self {
        let available_presets = vec![
            TimeRangePreset::LastSevenDays,
            TimeRangePreset::LastThirtyDays,
            TimeRangePreset::LastNinetyDays,
            TimeRangePreset::ThisMonth,
            TimeRangePreset::LastMonth,
            TimeRangePreset::LastQuarter,
            TimeRangePreset::ThisYear,
            TimeRangePreset::LastYear,
            TimeRangePreset::AllTime,
        ];

        let quick_ranges = vec![
            QuickRange {
                label: "7 Days".to_string(),
                preset: TimeRangePreset::LastSevenDays,
                active: false,
            },
            QuickRange {
                label: "30 Days".to_string(),
                preset: TimeRangePreset::LastThirtyDays,
                active: true, // Default selection
            },
            QuickRange {
                label: "90 Days".to_string(),
                preset: TimeRangePreset::LastNinetyDays,
                active: false,
            },
            QuickRange {
                label: "This Month".to_string(),
                preset: TimeRangePreset::ThisMonth,
                active: false,
            },
            QuickRange {
                label: "This Year".to_string(),
                preset: TimeRangePreset::ThisYear,
                active: false,
            },
        ];

        Self {
            selected_preset: Some(TimeRangePreset::LastThirtyDays),
            custom_range: None,
            available_presets,
            quick_ranges,
        }
    }

    /// Create selector with custom presets
    pub fn with_presets(presets: Vec<TimeRangePreset>) -> Self {
        let mut selector = Self::new();
        selector.available_presets = presets;
        selector
    }

    /// Select a time range preset
    pub fn select_preset(&mut self, preset: TimeRangePreset) -> Result<TimeRangeSelection> {
        // Update active state in quick ranges
        for quick_range in &mut self.quick_ranges {
            quick_range.active = std::mem::discriminant(&quick_range.preset) == std::mem::discriminant(&preset);
        }

        self.selected_preset = Some(preset.clone());
        
        let date_range = preset.to_date_range();
        let label = preset.display_name().to_string();
        
        // Validate the range
        let validation = self.validate_range(&date_range)?;
        if !validation.is_valid {
            return Err(tgraph_common::TGraphError::config(
                validation.error_message.unwrap_or_else(|| "Invalid date range".to_string())
            ));
        }

        Ok(TimeRangeSelection {
            date_range,
            label,
            is_custom: matches!(preset, TimeRangePreset::Custom(_)),
            preset: Some(preset),
        })
    }

    /// Set a custom date range
    pub fn set_custom_range(&mut self, start: NaiveDate, end: NaiveDate) -> Result<TimeRangeSelection> {
        let date_range = DateRange::new(start, end);
        
        // Validate the custom range
        let validation = self.validate_range(&date_range)?;
        if !validation.is_valid {
            return Err(tgraph_common::TGraphError::config(
                validation.error_message.unwrap_or_else(|| "Invalid custom date range".to_string())
            ));
        }

        // Deactivate all quick ranges
        for quick_range in &mut self.quick_ranges {
            quick_range.active = false;
        }

        self.selected_preset = Some(TimeRangePreset::Custom(date_range.clone()));
        self.custom_range = Some(date_range.clone());

        Ok(TimeRangeSelection {
            date_range: date_range.clone(),
            label: format!("{} to {}", start.format("%Y-%m-%d"), end.format("%Y-%m-%d")),
            is_custom: true,
            preset: Some(TimeRangePreset::Custom(date_range)),
        })
    }

    /// Get the currently selected time range
    pub fn get_current_selection(&self) -> Option<TimeRangeSelection> {
        if let Some(ref preset) = self.selected_preset {
            let date_range = preset.to_date_range();
            let label = preset.display_name().to_string();
            
            Some(TimeRangeSelection {
                date_range,
                label,
                is_custom: matches!(preset, TimeRangePreset::Custom(_)),
                preset: Some(preset.clone()),
            })
        } else {
            None
        }
    }

    /// Validate a date range
    pub fn validate_range(&self, range: &DateRange) -> Result<TimeRangeValidation> {
        let mut validation = TimeRangeValidation {
            is_valid: true,
            error_message: None,
            suggestions: vec![],
            warnings: vec![],
        };

        let today = chrono::Utc::now().date_naive();

        // Check if start date is after end date
        if range.start > range.end {
            validation.is_valid = false;
            validation.error_message = Some("Start date cannot be after end date".to_string());
            validation.suggestions.push("Please select a start date that comes before the end date".to_string());
            return Ok(validation);
        }

        // Check if end date is in the future
        if range.end > today {
            validation.warnings.push("End date is in the future - no data may be available for future dates".to_string());
        }

        // Check for very large date ranges that might impact performance
        let days_diff = (range.end - range.start).num_days();
        if days_diff > 1095 { // More than 3 years
            validation.warnings.push("Very large date range selected - this may impact performance".to_string());
            validation.suggestions.push("Consider using a smaller date range for better performance".to_string());
        }

        // Check for very small date ranges
        if days_diff < 1 {
            validation.warnings.push("Very small date range - consider selecting a longer period for better insights".to_string());
        }

        // Check if start date is too far in the past (assuming data might not be available)
        let years_ago = (today - range.start).num_days() / 365;
        if years_ago > 5 {
            validation.warnings.push("Start date is very far in the past - data may not be available".to_string());
        }

        Ok(validation)
    }

    /// Get suggested comparison ranges for the current selection
    pub fn get_comparison_suggestions(&self) -> Vec<ComparisonPeriod> {
        let Some(current_selection) = self.get_current_selection() else {
            return vec![];
        };

        let current_range = current_selection.date_range;
        let days_in_range = (current_range.end - current_range.start).num_days();
        
        let mut suggestions = vec![];

        // Previous period (same length)
        let previous_start = current_range.start - chrono::Duration::days(days_in_range + 1);
        let previous_end = current_range.start - chrono::Duration::days(1);
        suggestions.push(ComparisonPeriod {
            label: "Previous Period".to_string(),
            date_range: DateRange::new(previous_start, previous_end),
            color: Some("#FF6B6B".to_string()),
            enabled: true,
        });

        // Year-over-year comparison (if applicable)
        if days_in_range <= 365 {
            let yoy_start = current_range.start - chrono::Duration::days(365);
            let yoy_end = current_range.end - chrono::Duration::days(365);
            suggestions.push(ComparisonPeriod {
                label: "Same Period Last Year".to_string(),
                date_range: DateRange::new(yoy_start, yoy_end),
                color: Some("#4ECDC4".to_string()),
                enabled: false,
            });
        }

        // Month-over-month (if range is shorter than a month)
        if days_in_range <= 31 {
            let mom_start = current_range.start - chrono::Duration::days(30);
            let mom_end = current_range.end - chrono::Duration::days(30);
            suggestions.push(ComparisonPeriod {
                label: "Same Period Last Month".to_string(),
                date_range: DateRange::new(mom_start, mom_end),
                color: Some("#45B7D1".to_string()),
                enabled: false,
            });
        }

        suggestions
    }

    /// Create a filter configuration from the current selection
    pub fn to_filter_config(&self) -> FilterConfig {
        let mut filter_config = FilterConfig::default();
        
        if let Some(selection) = self.get_current_selection() {
            filter_config.date_range = Some(selection.date_range);
            filter_config.time_range_preset = selection.preset;
        }

        filter_config
    }

    /// Create a comparison configuration with suggested periods
    pub fn to_comparison_config(&self, enable_comparisons: bool) -> ComparisonConfig {
        if !enable_comparisons {
            return ComparisonConfig::default();
        }

        let Some(current_selection) = self.get_current_selection() else {
            return ComparisonConfig::default();
        };

        let comparison_ranges = self.get_comparison_suggestions();
        
        ComparisonConfig {
            enabled: true,
            primary_range: current_selection.date_range,
            comparison_ranges,
            display_mode: crate::ComparisonDisplayMode::Overlay,
            show_differences: true,
            show_growth_percentages: true,
            comparison_colors: vec![
                "#FF6B6B".to_string(),
                "#4ECDC4".to_string(),
                "#45B7D1".to_string(),
                "#FFA07A".to_string(),
            ],
        }
    }

    /// Get common relative date ranges
    pub fn get_relative_ranges() -> Vec<(String, TimeRangePreset)> {
        vec![
            ("Last 7 days".to_string(), TimeRangePreset::LastSevenDays),
            ("Last 30 days".to_string(), TimeRangePreset::LastThirtyDays),
            ("Last 90 days".to_string(), TimeRangePreset::LastNinetyDays),
            ("This month".to_string(), TimeRangePreset::ThisMonth),
            ("Last month".to_string(), TimeRangePreset::LastMonth),
            ("Last quarter".to_string(), TimeRangePreset::LastQuarter),
            ("This year".to_string(), TimeRangePreset::ThisYear),
            ("Last year".to_string(), TimeRangePreset::LastYear),
        ]
    }

    /// Get business-focused preset ranges
    pub fn get_business_ranges() -> Vec<(String, TimeRangePreset)> {
        vec![
            ("This week".to_string(), TimeRangePreset::LastSevenDays),
            ("This month".to_string(), TimeRangePreset::ThisMonth),
            ("This quarter".to_string(), TimeRangePreset::LastQuarter),
            ("This year".to_string(), TimeRangePreset::ThisYear),
            ("Year to date".to_string(), TimeRangePreset::ThisYear),
        ]
    }

    /// Convert relative time description to preset
    pub fn parse_relative_time(input: &str) -> Option<TimeRangePreset> {
        let input_lower = input.to_lowercase();
        
        match input_lower.as_str() {
            "yesterday" => Some(TimeRangePreset::Custom(DateRange::last_days(1))),
            "last week" | "past week" | "7 days" => Some(TimeRangePreset::LastSevenDays),
            "last month" | "past month" | "30 days" => Some(TimeRangePreset::LastThirtyDays),
            "last quarter" | "past quarter" | "90 days" => Some(TimeRangePreset::LastNinetyDays),
            "last year" | "past year" | "365 days" => Some(TimeRangePreset::LastYear),
            "this month" => Some(TimeRangePreset::ThisMonth),
            "this year" => Some(TimeRangePreset::ThisYear),
            "all time" | "everything" => Some(TimeRangePreset::AllTime),
            _ => None,
        }
    }
}

impl Default for TimeRangeSelector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_time_range_selector_creation() {
        let selector = TimeRangeSelector::new();
        assert!(!selector.available_presets.is_empty());
        assert_eq!(selector.quick_ranges.len(), 5);
        assert!(selector.selected_preset.is_some());
    }

    #[test]
    fn test_preset_selection() {
        let mut selector = TimeRangeSelector::new();
        let result = selector.select_preset(TimeRangePreset::LastSevenDays).unwrap();
        
        assert_eq!(result.label, "Last 7 Days");
        assert!(!result.is_custom);
        assert!(result.preset.is_some());
    }

    #[test]
    fn test_custom_range_setting() {
        let mut selector = TimeRangeSelector::new();
        let start = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        let end = NaiveDate::from_ymd_opt(2024, 1, 31).unwrap();
        
        let result = selector.set_custom_range(start, end).unwrap();
        assert!(result.is_custom);
        assert_eq!(result.label, "2024-01-01 to 2024-01-31");
    }

    #[test]
    fn test_range_validation() {
        let selector = TimeRangeSelector::new();
        
        // Valid range
        let valid_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
        );
        let validation = selector.validate_range(&valid_range).unwrap();
        assert!(validation.is_valid);

        // Invalid range (start after end)
        let invalid_range = DateRange::new(
            NaiveDate::from_ymd_opt(2024, 1, 31).unwrap(),
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        );
        let validation = selector.validate_range(&invalid_range).unwrap();
        assert!(!validation.is_valid);
        assert!(validation.error_message.is_some());
    }

    #[test]
    fn test_comparison_suggestions() {
        let mut selector = TimeRangeSelector::new();
        selector.select_preset(TimeRangePreset::LastThirtyDays).unwrap();
        
        let suggestions = selector.get_comparison_suggestions();
        assert!(!suggestions.is_empty());
        assert_eq!(suggestions[0].label, "Previous Period");
    }

    #[test]
    fn test_relative_time_parsing() {
        assert!(TimeRangeSelector::parse_relative_time("last week").is_some());
        assert!(TimeRangeSelector::parse_relative_time("this month").is_some());
        assert!(TimeRangeSelector::parse_relative_time("invalid").is_none());
    }

    #[test]
    fn test_filter_config_creation() {
        let mut selector = TimeRangeSelector::new();
        selector.select_preset(TimeRangePreset::LastSevenDays).unwrap();
        
        let filter_config = selector.to_filter_config();
        assert!(filter_config.date_range.is_some());
        assert!(filter_config.time_range_preset.is_some());
    }
} 