//! Graph renderer trait definitions for polymorphic graph types.

use async_trait::async_trait;
use tgraph_common::Result;

/// Trait for graph renderers that can generate visualizations.
#[async_trait]
pub trait GraphRenderer: Send + Sync {
    /// The type of data this renderer expects.
    type Data;

    /// The type of configuration this renderer uses.
    type Config;

    /// Renders a graph with the given data and configuration.
    async fn render(&self, data: Self::Data, config: Self::Config) -> Result<Vec<u8>>;

    /// Gets the name of this graph type.
    fn name(&self) -> &'static str;

    /// Gets the description of this graph type.
    fn description(&self) -> &'static str;
}
