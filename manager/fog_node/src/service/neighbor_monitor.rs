use crate::repository::latency_estimation::LatencyEstimation;
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::Time;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during latency estimation to neighboring nodes: {0}")]
    RttEstimation(#[from] crate::repository::latency_estimation::Error),
}

#[derive(Debug)]
pub struct NeighborMonitor {
    rtt_estimation: Arc<LatencyEstimation>,
}

impl NeighborMonitor {
    pub fn new(latency_estimation: Arc<LatencyEstimation>) -> Self {
        Self { rtt_estimation: latency_estimation }
    }

    #[allow(dead_code)]
    pub async fn ping_neighbors_rtt(&self) -> Result<(), Error> {
        self.rtt_estimation.latency_to_neighbors().await?;
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time> {
        self.rtt_estimation.get_latency_to(id).await.map(|x| x * 3.0)
    }
}
