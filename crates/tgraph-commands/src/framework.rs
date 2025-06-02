//! Poise framework setup and command registration logic.

use std::sync::Arc;
use tgraph_config::Config;

/// Application data accessible in all commands.
pub struct Data {
    /// Application configuration.
    pub config: Arc<Config>,
}

/// Application error type for commands.
pub type Error = Box<dyn std::error::Error + Send + Sync>;

/// Command context type.
pub type Context<'a> = poise::Context<'a, Data, Error>;

/// Creates a new Poise framework.
pub fn create_framework() -> poise::FrameworkBuilder<Data, Error> {
    poise::Framework::builder().options(poise::FrameworkOptions {
        commands: vec![
            crate::about::about(),
            crate::config::config(),
            crate::my_stats::my_stats(),
            crate::update_graphs::update_graphs(),
            crate::uptime::uptime(),
        ],
        prefix_options: poise::PrefixFrameworkOptions {
            prefix: Some("!".into()),
            ..Default::default()
        },
        ..Default::default()
    })
}
