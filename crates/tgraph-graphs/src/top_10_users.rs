//! Top 10 users graph implementation.

use crate::traits::GraphRenderer;
use async_trait::async_trait;
use tgraph_common::Result;

/// Top 10 users graph renderer.
pub struct Top10UsersGraph;

#[async_trait]
impl GraphRenderer for Top10UsersGraph {
    type Data = Vec<(String, u32)>; // (user, count)
    type Config = ();

    async fn render(&self, _data: Self::Data, _config: Self::Config) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "top_10_users"
    }

    fn description(&self) -> &'static str {
        "Top 10 users by play count"
    }
}
