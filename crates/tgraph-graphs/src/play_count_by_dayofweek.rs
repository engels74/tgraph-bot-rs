//! Play count by day of week graph implementation.

use crate::traits::GraphRenderer;
use async_trait::async_trait;
use tgraph_common::Result;

/// Play count by day of week graph renderer.
pub struct PlayCountByDayOfWeekGraph;

#[async_trait]
impl GraphRenderer for PlayCountByDayOfWeekGraph {
    type Data = Vec<(String, u32)>; // (day, count)
    type Config = ();

    async fn render(&self, _data: Self::Data, _config: Self::Config) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "play_count_by_dayofweek"
    }

    fn description(&self) -> &'static str {
        "Play count by day of week"
    }
}
