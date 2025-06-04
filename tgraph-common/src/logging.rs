//! Structured logging infrastructure for TGraph

use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter,
};
use std::io;

/// Configuration for the logging system
#[derive(Debug, Clone)]
pub struct LoggingConfig {
    /// Log level filter (e.g., "info", "debug", "trace")
    pub level: String,
    /// Whether to enable JSON formatting
    pub json_format: bool,
    /// Whether to enable pretty formatting with colors
    pub pretty_format: bool,
    /// Optional file path for log output
    pub file_path: Option<String>,
    /// Whether to include spans in the output
    pub include_spans: bool,
    /// Whether to include timestamps
    pub include_timestamps: bool,
    /// Whether to include target module information
    pub include_targets: bool,
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            json_format: false,
            pretty_format: true,
            file_path: None,
            include_spans: true,
            include_timestamps: true,
            include_targets: true,
        }
    }
}

/// Initialize the tracing subscriber with the given configuration
pub fn init_logging(config: LoggingConfig) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Create the environment filter
    let env_filter = EnvFilter::try_new(&config.level)
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Configure span events
    let span_events = if config.include_spans {
        FmtSpan::NEW | FmtSpan::CLOSE
    } else {
        FmtSpan::NONE
    };

    // Build the registry with layers
    let registry = tracing_subscriber::registry().with(env_filter);

    if config.json_format {
        // Use compact format for JSON-like output
        let layer = fmt::layer()
            .with_span_events(span_events)
            .with_thread_ids(true)
            .with_thread_names(true)
            .with_target(config.include_targets)
            .compact();

        if let Some(file_path) = config.file_path {
            // Write to file
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;
            registry.with(layer.with_writer(file)).init();
        } else {
            // Write to stdout
            registry.with(layer).init();
        }
    } else if config.pretty_format {
        // Pretty formatting layer
        let layer = fmt::layer()
            .pretty()
            .with_span_events(span_events)
            .with_ansi(config.pretty_format)
            .with_target(config.include_targets)
            .with_thread_ids(true)
            .with_thread_names(true);

        if let Some(file_path) = config.file_path {
            // Write to file  
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;
            registry.with(layer.with_ansi(false).with_writer(file)).init();
        } else {
            // Write to stdout
            registry.with(layer).init();
        }
    } else {
        // Standard formatting layer
        let layer = fmt::layer()
            .with_span_events(span_events)
            .with_target(config.include_targets)
            .with_thread_ids(true)
            .with_thread_names(true);

        if let Some(file_path) = config.file_path {
            // Write to file  
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(file_path)?;
            registry.with(layer.with_writer(file)).init();
        } else {
            // Write to stdout
            registry.with(layer).init();
        }
    }

    Ok(())
}

/// Initialize logging with default configuration
pub fn init_default_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig::default())
}

/// Initialize logging for development (pretty, debug level)
pub fn init_dev_logging() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig {
        level: "debug".to_string(),
        pretty_format: true,
        json_format: false,
        include_spans: true,
        ..LoggingConfig::default()
    })
}

/// Initialize logging for production (compact format, info level, with file output)
pub fn init_prod_logging(
    log_file: impl Into<String>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    init_logging(LoggingConfig {
        level: "info".to_string(),
        json_format: true, // Use compact format for production
        pretty_format: false,
        file_path: Some(log_file.into()),
        include_spans: false,
        ..LoggingConfig::default()
    })
}

/// Initialize logging with dual output (console and file) using tee
pub fn init_dual_logging(
    log_file: impl Into<String>
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tracing_subscriber::fmt::writer::MakeWriterExt;

    let file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file.into())?;

    let env_filter = EnvFilter::try_new("info")
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(
            fmt::layer()
                .with_writer(io::stdout.and(file))
                .with_ansi(false) // Disable colors for file compatibility
                .with_target(true)
                .with_thread_ids(true)
                .with_thread_names(true)
                .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        )
        .init();

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = LoggingConfig::default();
        assert_eq!(config.level, "info");
        assert!(!config.json_format);
        assert!(config.pretty_format);
        assert!(config.file_path.is_none());
        assert!(config.include_spans);
        assert!(config.include_timestamps);
        assert!(config.include_targets);
    }
} 