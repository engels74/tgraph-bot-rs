//! Play count by hour of day graph implementation.

use crate::traits::GraphRenderer;
use async_trait::async_trait;
use tgraph_common::Result;

/// Play count by hour of day graph renderer.
pub struct PlayCountByHourOfDayGraph;

#[async_trait]
impl GraphRenderer for PlayCountByHourOfDayGraph {
    type Data = Vec<(u8, u32)>; // (hour, count)
    type Config = ();

    async fn render(&self, _data: Self::Data, _config: Self::Config) -> Result<Vec<u8>> {
        Ok(Vec::new())
    }

    fn name(&self) -> &'static str {
        "play_count_by_hourofday"
    }

    fn description(&self) -> &'static str {
        "Play count by hour of day"
    }
}
