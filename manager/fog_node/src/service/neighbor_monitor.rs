use crate::repository::latency_estimation::LatencyEstimation;
use anyhow::{Context, Result};
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::Time;

#[derive(Debug)]
pub struct NeighborMonitor {
    rtt_estimation: Arc<LatencyEstimation>,
}

impl NeighborMonitor {
    pub fn new(latency_estimation: Arc<LatencyEstimation>) -> Self {
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
    pub async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time> {
        self.rtt_estimation.get_latency_to(id).await.map(|x| x * 3.0)
    }
}
