use crate::repository::latency_estimation::LatencyEstimation;
use anyhow::{Context, Result};
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;

impl From<crate::repository::latency_estimation::Latency>
    for model::view::auction::Latency
{
    fn from(val: crate::repository::latency_estimation::Latency) -> Self {
        model::view::auction::Latency {
            median:              val.median,
            average:             val.average,
            interquantile_range: val.interquantile_range,
            packet_loss:         val.packet_loss,
        }
    }
}

#[derive(Debug)]
pub struct NeighborMonitor {
    rtt_estimation: Arc<Box<dyn LatencyEstimation>>,
}

impl NeighborMonitor {
    pub fn new(latency_estimation: Arc<Box<dyn LatencyEstimation>>) -> Self {
        Self { rtt_estimation: latency_estimation }
    }

    #[allow(dead_code)]
    pub async fn ping_neighbors_rtt(&self) -> Result<()> {
        self.rtt_estimation.latency_to_neighbors().await.context(
            "Failed to get the latencies between me and my neighbors",
        )?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_latency_to(
        &self,
        id: &NodeId,
    ) -> Option<model::view::auction::Latency> {
        self.rtt_estimation.get_latency_to(id).await.map(|x| x.into())
    }
}
