# TGraph Bot Rust Edition

[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License: AGPL v3](https://img.shields.io/badge/License-AGPL%20v3-blue.svg)](https://www.gnu.org/licenses/agpl-3.0)

High-performance Discord bot for automated Tautulli graph generation and posting, built with Rust and the Poise framework.

## Overview

TGraph Bot Rust Edition is a complete rewrite of the original Python TGraph Bot, designed for maximum performance, reliability, and maintainability. It automatically generates and posts beautiful graphs from your Tautulli (Plex) data to Discord channels.

## Features

- **High Performance**: Built with Rust for zero-cost abstractions and memory safety
- **Discord Integration**: Modern slash commands using the Poise framework
- **Automated Scheduling**: Configurable graph generation and posting
- **Multiple Graph Types**: Daily play counts, user statistics, platform analytics, and more
- **Internationalization**: Multi-language support using Fluent localization
- **Hot Configuration Reloading**: Update settings without restarting the bot
- **Privacy Controls**: Configurable username censoring and data filtering
- **Type Safety**: Compile-time guarantees for configuration and data handling

## Architecture

This project uses a modular workspace structure:

- **`tgraph-bot`**: Main binary crate and application entry point
- **`tgraph-commands`**: Discord command implementations using Poise
- **`tgraph-config`**: Type-safe configuration management with hot-reloading
- **`tgraph-graphs`**: High-performance graph generation and rendering
- **`tgraph-i18n`**: Internationalization support using Fluent
- **`tgraph-common`**: Shared types, utilities, and common functionality

## Development Status

🚧 **This project is currently under active development.**

See [docs/development_plan.md](docs/development_plan.md) for the complete implementation roadmap.

## Requirements

- Rust 1.75 or later
- A Discord bot token
- Tautulli API access
- System dependencies: `libfontconfig1-dev`, `pkg-config`

## Quick Start

```bash
# Clone the repository
git clone https://github.com/engels74/tgraph-bot-rs.git
cd tgraph-bot-rs

# Install system dependencies (Ubuntu/Debian)
sudo apt update && sudo apt install -y libfontconfig1-dev pkg-config

# Build the project
cargo build --release

# Set environment variables
export DISCORD_TOKEN="your_discord_bot_token"
export TAUTULLI_API_KEY="your_tautulli_api_key"
export TAUTULLI_URL="http://your-tautulli-server:8181/api/v2"

# Run the bot
cargo run --release
```

## Configuration

The bot supports configuration through environment variables and configuration files. See the [configuration documentation](docs/configuration.md) for detailed setup instructions.

## Contributing

Contributions are welcome! Please read our [contributing guidelines](CONTRIBUTING.md) and follow the established development practices:

- Use Test-Driven Development (TDD)
- Run `cargo check`, `cargo clippy`, `cargo fmt`, and `cargo test` before submitting
- Follow Rust 2024/2025 edition best practices
- Maintain comprehensive test coverage

## License

This project is licensed under the GNU Affero General Public License v3.0 - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Original TGraph Bot Python implementation
- [Poise](https://github.com/serenity-rs/poise) Discord framework
- [Serenity](https://github.com/serenity-rs/serenity) Discord library
- [Plotters](https://github.com/plotters-rs/plotters) for graph rendering