# TGraph Discord Bot

A Discord bot for data visualization and analytics, integrated with Tautulli for media server statistics and graph generation.

## Features

- **Discord Integration**: Built with Poise framework for modern Discord bot functionality
- **Data Visualization**: Generate graphs and charts from Tautulli data
- **Internationalization**: Multi-language support
- **Configurable**: Flexible configuration system with TOML files
- **Modular Architecture**: Clean separation of concerns with workspace crates

## Quick Start

### Prerequisites

- Rust 1.70+ (2021 edition)
- Discord bot token (from Discord Developer Portal)
- Tautulli server with API access

### Installation

1. Clone the repository:
```bash
git clone https://github.com/engels74/tgraph-bot-rs
cd tgraph-bot-rs
```

2. Copy the example configuration:
```bash
cp config.example.toml config.toml
```

3. Edit `config.toml` with your Discord bot token and Tautulli settings:
```toml
[discord]
token = "YOUR_BOT_TOKEN_HERE"

[tautulli]
url = "http://your-tautulli-server:8181"
api_key = "YOUR_TAUTULLI_API_KEY"
```

4. Build and run:
```bash
cargo build --release
cargo run --bin tgraph-bot
```

### Discord Bot Setup

1. Go to the [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application and bot
3. Copy the bot token to your `config.toml`
4. Add the bot to your server with appropriate permissions:
   - Send Messages
   - Use Slash Commands
   - Embed Links
   - Attach Files

## Architecture

This project uses a Rust workspace with the following crates:

- **tgraph-bot**: Main bot application with Discord integration
- **tgraph-commands**: Discord command implementations
- **tgraph-config**: Configuration management and validation
- **tgraph-graphs**: Graph generation and visualization
- **tgraph-i18n**: Internationalization support
- **tgraph-common**: Shared utilities and types

## Development

### Running Tests
```bash
cargo test
```

### Code Quality
```bash
cargo clippy
cargo fmt
```

### Development Mode
```bash
cargo run --bin tgraph-bot -- --log-level debug
```

## Configuration

See `config.example.toml` for all available configuration options including:

- Discord bot settings
- Tautulli API configuration
- Graph rendering options
- Database settings
- Logging configuration
- Scheduling options

## License

Licensed under the Apache License, Version 2.0. See [LICENSE](LICENSE) for details.

## Contributing

Contributions are welcome! Please feel free to submit pull requests.