//! Top 10 platforms graph implementation.

use crate::traits::GraphRenderer;
use async_trait::async_trait;
use tgraph_common::Result;

/// Top 10 platforms graph renderer.
pub struct Top10PlatformsGraph;

#[async_trait]
impl GraphRenderer for Top10PlatformsGraph {
    type Data = Vec<(String, u32)>; // (platform, count)
    type Config = ();

    async fn render(&self, _data: Self::Data, _config: Self::Config) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "top_10_platforms"
    }

    fn description(&self) -> &'static str {
        "Top 10 platforms by play count"
    }
}
