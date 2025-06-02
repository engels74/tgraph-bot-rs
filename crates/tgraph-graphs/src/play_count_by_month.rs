//! Play count by month graph implementation.

use crate::traits::GraphRenderer;
use async_trait::async_trait;
use tgraph_common::Result;

/// Play count by month graph renderer.
pub struct PlayCountByMonthGraph;

#[async_trait]
impl GraphRenderer for PlayCountByMonthGraph {
    type Data = Vec<(String, u32)>; // (month, count)
    type Config = ();

    async fn render(&self, _data: Self::Data, _config: Self::Config) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "play_count_by_month"
    }

    fn description(&self) -> &'static str {
        "Play count by month"
    }
}
