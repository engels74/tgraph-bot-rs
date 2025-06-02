# TGraph Bot Rust Edition - Project Overview

## Introduction

TGraph Bot Rust Edition is a high-performance Discord bot engineered to automatically generate and post Tautulli graphs to designated Discord channels. Built with Rust's safety guarantees and zero-cost abstractions, it delivers exceptional performance while providing comprehensive visualizations and statistics about your Plex Media Server's activity. The project leverages **Poise** (built on Serenity) for Discord interactions, exemplifies modern Rust patterns, rigorous test-driven development (TDD), and seamless internationalization support through the Fluent localization system.

## Key Features

TGraph Bot Rust Edition delivers:

- High-performance automated generation and scheduled posting of Tautulli graphs leveraging Rust's async runtime
- Fully customizable graph rendering with compile-time validated color schemes, grid configurations, and annotation settings
- **Type-safe Discord slash commands using Poise framework** (`/about`, `/config`, `/my_stats`, `/update_graphs`, and `/uptime`) with built-in permission management
- Zero-copy user statistics generation with efficient direct messaging via async channels
- Internationalization using Fluent's powerful localization framework with compile-time message validation
- Comprehensive testing strategy combining unit tests, integration tests, and property-based testing with proptest

## Project Architecture

The project follows Rust's workspace structure with multiple crates for modularity and compilation efficiency:

```
tgraph-bot/
├── Cargo.toml                    # Workspace manifest
├── Cargo.lock
├── LICENSE
├── README.md
├── .rustfmt.toml
├── .clippy.toml
├── fluent/
│   ├── en-US/
│   │   └── main.ftl
│   └── messages.ftl.template
├── crates/
│   ├── tgraph-bot/              # Main binary crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── bot.rs
│   │   │   ├── error.rs
│   │   │   └── lib.rs
│   │   └── tests/
│   ├── tgraph-commands/         # Discord commands crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── framework.rs
│   │   │   ├── commands.rs
│   │   │   ├── about.rs
│   │   │   ├── config.rs
│   │   │   ├── my_stats.rs
│   │   │   ├── update_graphs.rs
│   │   │   └── uptime.rs
│   │   └── tests/
│   ├── tgraph-config/           # Configuration management crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── schema.rs
│   │   │   ├── loader.rs
│   │   │   ├── validator.rs
│   │   │   ├── defaults.rs
│   │   │   └── cache.rs
│   │   └── tests/
│   ├── tgraph-graphs/           # Graph generation crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── manager.rs
│   │   │   ├── user_manager.rs
│   │   │   ├── traits.rs
│   │   │   ├── daily_play_count.rs
│   │   │   ├── play_count_by_dayofweek.rs
│   │   │   ├── play_count_by_hourofday.rs
│   │   │   ├── play_count_by_month.rs
│   │   │   ├── top_10_platforms.rs
│   │   │   ├── top_10_users.rs
│   │   │   ├── data_fetcher.rs
│   │   │   └── utils.rs
│   │   └── tests/
│   ├── tgraph-i18n/             # Internationalization crate
│   │   ├── Cargo.toml
│   │   ├── src/
│   │   │   ├── lib.rs
│   │   │   ├── loader.rs
│   │   │   └── messages.rs
│   │   └── tests/
│   └── tgraph-common/           # Shared utilities crate
│       ├── Cargo.toml
│       ├── src/
│       │   ├── lib.rs
│       │   ├── types.rs
│       │   └── utils.rs
│       └── tests/
├── config/
│   └── config.yml.sample
└── target/                      # Build artifacts
```

## Crate Descriptions

### Workspace Root

- **Cargo.toml**: Defines the workspace members and shared dependencies, enabling consistent versioning and feature flags across all crates
- **Cargo.lock**: Ensures reproducible builds by locking exact dependency versions
- **.rustfmt.toml**: Enforces consistent code formatting following Rust community standards
- **.clippy.toml**: Configures Clippy lints for enhanced code quality and idiomaticity
- **fluent/**: Contains localization files in Fluent format, supporting complex pluralization and context-aware translations

### Main Binary Crate (`tgraph-bot`)

The entry point crate orchestrates the entire application lifecycle:

- **main.rs**: Initializes the tokio runtime, sets up structured logging with tracing, and launches the bot
- **bot.rs**: Implements the core bot logic using the Poise framework, managing the Discord client and background tasks
- **error.rs**: Defines the application-wide error types using thiserror, ensuring comprehensive error handling
- **lib.rs**: Exposes the public API for integration testing

### Commands Crate (`tgraph-commands`)

Implements Discord slash commands with **Poise's type-safe command framework**:

- **framework.rs**: Defines Poise framework setup and command registration logic
- **commands.rs**: Exports all command functions for registration with the framework
- **about.rs**: `/about` command implementation using Poise's command macro with automatic help generation
- **config.rs**: `/config` subcommands with Poise's built-in subcommand support for viewing and editing configuration
- **my_stats.rs**: `/my_stats` command with user parameter parsing and DM support using Poise's argument system
- **update_graphs.rs**: `/update_graphs` administrative command with Poise's permission checks
- **uptime.rs**: `/uptime` command accessing framework data through Poise's context

### Configuration Crate (`tgraph-config`)

Provides type-safe configuration management:

- **schema.rs**: Defines configuration structures using serde with validation attributes
- **loader.rs**: Implements atomic file operations for configuration persistence
- **validator.rs**: Compile-time and runtime validation using custom derive macros
- **defaults.rs**: Type-safe default values using const functions
- **cache.rs**: Thread-safe configuration caching with arc-swap for lock-free reads

### Graph Generation Crate (`tgraph-graphs`)

Handles all graph rendering logic:

- **traits.rs**: Defines the `GraphRenderer` trait for polymorphic graph types
- **manager.rs**: Orchestrates server-wide graph generation with parallel processing
- **user_manager.rs**: Manages user-specific graph generation with privacy controls
- **data_fetcher.rs**: Implements efficient Tautulli API client with connection pooling
- Individual graph modules implement specific visualizations using plotters for native Rust rendering

### Internationalization Crate (`tgraph-i18n`)

Provides compile-time validated translations:

- **loader.rs**: Fluent bundle initialization with lazy static loading
- **messages.rs**: Type-safe message accessors generated via build script

### Common Utilities Crate (`tgraph-common`)

Shared types and utilities:

- **types.rs**: Common type definitions and newtype wrappers for domain modeling
- **utils.rs**: Shared utility functions with zero-cost abstractions

## Configuration Schema

The configuration system leverages Rust's type system for validation:

```yaml
# config/config.yml.sample
tautulli:
  api_key: "your_tautulli_api_key"
  url: "http://your_tautulli_ip:port/api/v2"

discord:
  token: "your_discord_bot_token"
  channel_id: "your_channel_id"

scheduling:
  update_days: 7
  fixed_update_time: null  # Optional<String> - HH:MM format
  keep_days: 7

data:
  time_range_days: 30
  language: "en-US"

graphs:
  enabled:
    daily_play_count: true
    play_count_by_dayofweek: true
    play_count_by_hourofday: true
    top_10_platforms: true
    top_10_users: true
    play_count_by_month: true
  
  privacy:
    censor_usernames: true
  
  styling:
    enable_grid: false
    colors:
      tv: "#1f77b4"
      movie: "#ff7f0e"
      background: "#ffffff"
      annotation: "#ff0000"
      annotation_outline: "#000000"
    
    annotations:
      enable_outline: true
      graphs:
        daily_play_count: true
        play_count_by_dayofweek: true
        play_count_by_hourofday: true
        top_10_platforms: true
        top_10_users: true
        play_count_by_month: true

rate_limiting:
  config_cooldown_minutes: 0
  config_global_cooldown_seconds: 0
  update_graphs_cooldown_minutes: 0
  update_graphs_global_cooldown_seconds: 0
  my_stats_cooldown_minutes: 5
  my_stats_global_cooldown_seconds: 60
```

## Rust Best Practices

The project exemplifies modern Rust patterns:

• **Zero-Cost Abstractions**: Leverages traits and generics for runtime performance without overhead

• **Type Safety**: Utilizes newtype patterns, phantom types, and sealed traits for compile-time guarantees

• **Error Handling**: Implements custom error types with thiserror and comprehensive Result propagation

• **Async Excellence**: Uses tokio with careful attention to cancellation safety and structured concurrency

• **Memory Safety**: Ensures zero unsafe code outside of well-audited dependencies

• **Builder Patterns**: Employs type-state builders for complex object construction

• **Trait Objects**: Strategic use of dynamic dispatch only where flexibility is required

## Poise Framework Integration

The bot leverages Poise's powerful features for Discord interaction:

• **Unified Command Definition**: Single function signature works for both prefix and slash commands

• **Type-Safe Arguments**: Command parameters use normal Rust types with automatic parsing and validation

• **Edit Tracking**: When users edit their command message, the bot automatically updates its response

• **Built-in Cooldowns**: Per-user and global cooldowns are handled by the framework

• **Subcommand Support**: Natural subcommand hierarchies with the `/config view` and `/config edit` pattern

• **Permission Checks**: Declarative permission requirements using Poise's check system

• **Context Data**: Shared application state accessible in all commands through Poise's context

## Test-Driven Development Strategy

The project follows rigorous TDD practices tailored for Rust:

1. **Test First**: Each feature begins with failing tests that define expected behavior
2. **Red-Green-Refactor**: Tests fail initially, implementation makes them pass, then code is refined
3. **Property Testing**: Utilizes proptest for exhaustive edge case coverage
4. **Integration Testing**: Comprehensive integration tests in separate test crates
5. **Benchmarking**: Performance regression tests using criterion
6. **Mocking**: Strategic use of mockall for external service testing
7. **Coverage**: Enforces minimum coverage thresholds with tarpaulin

## Internationalization Architecture

The bot implements state-of-the-art localization:

• **Fluent Localization System**: Mozilla's Fluent format for natural-sounding translations with proper pluralization

• **Compile-Time Validation**: Build scripts validate all message keys exist across locales

• **Context-Aware Messages**: Supports gender, case, and number agreement in translations

• **Lazy Loading**: Translations loaded on-demand with efficient caching

• **Type-Safe Access**: Generated accessor functions prevent runtime key errors

• **Continuous Localization**: Integrated with translation management platforms via Fluent's toolchain

## Performance Characteristics

• **Memory Efficiency**: Zero-copy parsing where possible, arena allocators for graph data

• **Concurrency**: Lock-free data structures for hot paths, careful synchronization primitives

• **Resource Management**: RAII patterns ensure proper cleanup of system resources

• **Compilation**: Leverages Link-Time Optimization (LTO) and Profile-Guided Optimization (PGO)
