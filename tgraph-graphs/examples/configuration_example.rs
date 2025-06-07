//! Example demonstrating the graph configuration and customization system

use tgraph_graphs::{
    ColorScheme, ConfigPresets, ConfigurationManager, GraphConfigBuilder,
    DailyPlayCountConfig, DayOfWeekConfig, HourlyDistributionConfig,
    MonthlyTrendsConfig, TopItemsConfig, DateRange, SortOrder, GridStyle,
};
use chrono::NaiveDate;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Graph Configuration System Example ===\n");

    // 1. Basic Configuration Creation
    println!("1. Creating a basic graph configuration:");
    let basic_config = GraphConfigBuilder::new("Basic Example")
        .title("Daily Play Count")
        .dimensions(800, 600)
        .color_scheme(ColorScheme::Default)
        .labels(Some("Date"), Some("Play Count"))
        .build()?;
    
    println!("   Title: {}", basic_config.base.title);
    println!("   Dimensions: {}x{}", basic_config.base.width, basic_config.base.height);
    println!("   Color Scheme: {:?}\n", basic_config.base.style.color_scheme);

    // 2. Using Preset Configurations
    println!("2. Using preset configurations:");
    
    let presentation_config = ConfigPresets::presentation("Presentation Graph")
        .description("High-resolution graph for presentations")
        .build()?;
    println!("   Presentation: {}x{} with {:?} theme", 
        presentation_config.base.width, 
        presentation_config.base.height,
        presentation_config.base.style.color_scheme);

    let report_config = ConfigPresets::report("Report Graph")
        .description("Professional graph for reports")
        .build()?;
    println!("   Report: {}x{} with {:?} theme", 
        report_config.base.width, 
        report_config.base.height,
        report_config.base.style.color_scheme);

    let dashboard_config = ConfigPresets::dashboard("Dashboard Graph")
        .description("Compact graph for dashboards")
        .build()?;
    println!("   Dashboard: {}x{} with {:?} theme\n", 
        dashboard_config.base.width, 
        dashboard_config.base.height,
        dashboard_config.base.style.color_scheme);

    // 3. Advanced Configuration with Filters
    println!("3. Advanced configuration with data filters:");
    let filtered_config = GraphConfigBuilder::new("Filtered Graph")
        .title("Last 30 Days - Top Platforms Only")
        .last_days(30)
        .platforms(vec!["Plex".to_string(), "Netflix".to_string()])
        .data_limit(50)
        .minimum_threshold(5.0)
        .build()?;
    
    if let Some(date_range) = &filtered_config.filters.date_range {
        println!("   Date range: {} to {}", date_range.start, date_range.end);
    }
    if let Some(platforms) = &filtered_config.filters.platforms {
        println!("   Platforms: {:?}", platforms);
    }
    println!("   Data limit: {:?}", filtered_config.filters.data_point_limit);
    println!("   Min threshold: {:?}\n", filtered_config.filters.minimum_threshold);

    // 4. Graph-Specific Configurations
    println!("4. Graph-specific configurations:");

    // Daily Play Count Configuration
    let daily_config = GraphConfigBuilder::new("Daily Play Count")
        .title("Daily Play Count with Weekend Highlighting")
        .daily_play_count_config(|builder| {
            builder
                .highlight_weekends(true)
                .moving_average(true, 7)
                .growth_trends(true)
                .weekend_color("#ff6b6b")
                .line_style(3, true)
        })
        .build()?;
    println!("   Daily Play Count: Weekend highlighting and 7-day moving average enabled");

    // Day of Week Configuration
    let dow_config = GraphConfigBuilder::new("Day of Week")
        .title("Play Count by Day of Week")
        .day_of_week_config(|builder| {
            builder
                .start_week_monday(true)
                .show_percentages(true)
                .highlight_weekends(true)
                .bar_width_ratio(0.8)
                .show_average_line(true)
        })
        .build()?;
    println!("   Day of Week: Percentages view with average line");

    // Hourly Distribution Configuration
    let hourly_config = GraphConfigBuilder::new("Hourly Distribution")
        .title("Play Count by Hour")
        .hourly_distribution_config(|builder| {
            builder
                .time_format_24h(true)
                .highlight_peak_hours(true, 85.0)
                .smooth_curve(true)
        })
        .build()?;
    println!("   Hourly Distribution: 24h format with peak hour highlighting");

    // Monthly Trends Configuration
    let monthly_config = GraphConfigBuilder::new("Monthly Trends")
        .title("Monthly Play Count Trends")
        .monthly_trends_config(|builder| {
            builder
                .year_over_year_comparison(true)
                .seasonal_trends(true)
                .forecast(true, 3)
                .growth_labels(true)
        })
        .build()?;
    println!("   Monthly Trends: YoY comparison with 3-month forecast");

    // Top Items Configuration
    let top_items_config = GraphConfigBuilder::new("Top Platforms")
        .title("Top 10 Platforms")
        .top_items_config(|builder| {
            builder
                .max_items(10)
                .show_percentages(true)
                .show_others_category(true)
                .horizontal_bars(true)
                .show_data_labels(true)
                .minimum_count(5)
        })
        .build()?;
    println!("   Top Items: Top 10 with percentages and 'Others' category\n");

    // 5. Configuration Management
    println!("5. Configuration management:");
    let mut manager = ConfigurationManager::default();
    
    // Save configurations
    manager.save_config("daily_config".to_string(), daily_config.clone());
    manager.save_config("presentation_config".to_string(), presentation_config.clone());
    
    println!("   Saved configurations: {:?}", manager.list_saved_configs());
    
    // Export configuration to JSON
    let json_export = manager.export_config(&daily_config)?;
    println!("   Exported configuration size: {} bytes", json_export.len());
    
    // Import configuration from JSON
    let imported_config = manager.import_config(&json_export)?;
    println!("   Imported configuration: {}", imported_config.metadata.name);

    // 6. Custom Styling
    println!("\n6. Custom styling examples:");
    
    let dark_theme = GraphConfigBuilder::new("Dark Theme")
        .title("Dark Theme Graph")
        .color_scheme(ColorScheme::Dark)
        .background_color("#1a1a1a")
        .title_font("Arial", 20)
        .margins(30, 30, 50, 70)
        .grid(true, true, GridStyle::Dashed)
        .grid_color("#404040")
        .build()?;
    println!("   Dark theme with dashed grid lines");

    let vibrant_theme = GraphConfigBuilder::new("Vibrant Theme")
        .title("Vibrant Theme Graph")
        .color_scheme(ColorScheme::Vibrant)
        .background_color("#f8f9fa")
        .title_font("Helvetica", 18)
        .display(true, false, true)
        .animations(true)
        .build()?;
    println!("   Vibrant theme with animations enabled");

    let custom_colors = GraphConfigBuilder::new("Custom Colors")
        .title("Custom Color Scheme")
        .color_scheme(ColorScheme::Custom(vec![
            "#e74c3c".to_string(), // Red
            "#3498db".to_string(), // Blue
            "#2ecc71".to_string(), // Green
            "#f39c12".to_string(), // Orange
            "#9b59b6".to_string(), // Purple
        ]))
        .build()?;
    println!("   Custom color scheme with 5 colors");

    // 7. Validation Examples
    println!("\n7. Configuration validation:");
    
    // Valid configuration
    let valid_config = GraphConfigBuilder::new("Valid")
        .dimensions(800, 600)
        .data_limit(100)
        .build();
    println!("   Valid configuration: {:?}", valid_config.is_ok());
    
    // Invalid configuration (will fail validation)
    let invalid_config = GraphConfigBuilder::new("Invalid")
        .dimensions(0, 0) // Invalid dimensions
        .build();
    println!("   Invalid configuration: {:?}", invalid_config.is_err());
    if let Err(e) = invalid_config {
        println!("   Error: {}", e);
    }

    // 8. Preset Usage
    println!("\n8. Using presets from configuration manager:");
    let presets = manager.get_presets();
    println!("   Available presets: {}", presets.len());
    for preset in presets {
        println!("     - {}: {}", 
            preset.metadata.name, 
            preset.metadata.description.as_deref().unwrap_or("No description"));
    }

    // Create configuration from preset
    if let Some(preset_config) = manager.create_from_preset("Dark Presentation") {
        println!("   Created from 'Dark Presentation' preset: {}x{}", 
            preset_config.base.width, preset_config.base.height);
    }

    println!("\n=== Configuration System Example Complete ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_example_configurations() {
        // Test that all example configurations can be created successfully
        let basic = GraphConfigBuilder::new("Test")
            .title("Test Graph")
            .dimensions(800, 600)
            .build();
        assert!(basic.is_ok());

        let presentation = ConfigPresets::presentation("Test").build();
        assert!(presentation.is_ok());

        let with_filters = GraphConfigBuilder::new("Filtered")
            .last_days(7)
            .data_limit(50)
            .build();
        assert!(with_filters.is_ok());
    }

    #[test]
    fn test_graph_specific_configs() {
        let daily = GraphConfigBuilder::new("Daily")
            .daily_play_count_config(|b| b.highlight_weekends(true))
            .build();
        assert!(daily.is_ok());

        let hourly = GraphConfigBuilder::new("Hourly")
            .hourly_distribution_config(|b| b.time_format_24h(false))
            .build();
        assert!(hourly.is_ok());
    }

    #[test]
    fn test_configuration_manager() {
        let mut manager = ConfigurationManager::new();
        let config = GraphConfigBuilder::new("Test").build().unwrap();
        
        manager.save_config("test".to_string(), config.clone());
        assert_eq!(manager.list_saved_configs().len(), 1);
        
        let retrieved = manager.get_saved_config("test");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().metadata.name, "Test");
    }
} 