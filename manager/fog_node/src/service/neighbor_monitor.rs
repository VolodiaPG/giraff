use crate::repository::latency_estimation::LatencyEstimation;
use async_trait::async_trait;
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::Time;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error during latency estimation to neighboring nodes: {0}")]
    RttEstimation(#[from] crate::repository::latency_estimation::Error),
}

#[async_trait]
pub trait NeighborMonitor: Debug + Sync + Send {
    async fn ping_neighbors_rtt(&self) -> Result<(), Error>;
    async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time>;
    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time>;
}

#[derive(Debug)]
pub struct NeighborMonitorImpl {
    rtt_estimation: Arc<dyn LatencyEstimation>,
}

impl NeighborMonitorImpl {
    pub fn new(latency_estimation: Arc<dyn LatencyEstimation>) -> Self {
        Self { rtt_estimation: latency_estimation }
    }
}

#[async_trait]
impl NeighborMonitor for NeighborMonitorImpl {
    async fn ping_neighbors_rtt(&self) -> Result<(), Error> {
        self.rtt_estimation.latency_to_neighbors().await?;
        Ok(())
    }

    async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time> {
        self.rtt_estimation.get_latency_to_avg(id).await
    }

    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time> {
        self.rtt_estimation.get_latency_from_avg(id).await
    }
}
