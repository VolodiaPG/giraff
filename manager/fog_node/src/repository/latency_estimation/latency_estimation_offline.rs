use super::{Latency, LatencyEstimation};
use crate::NodeSituation;
use anyhow::Result;
use async_trait::async_trait;
use model::dto::node::NodeDescription;
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::{Ratio, Time};
use uom::si::ratio::ratio;
use uom::si::time::second;

#[derive(Debug)]
pub struct LatencyEstimationOfflineImpl {
    node_situation: Arc<NodeSituation>,
}

#[async_trait]
impl LatencyEstimation for LatencyEstimationOfflineImpl {
    async fn latency_to_neighbors(&self) -> Result<()> { Ok(()) }

    async fn get_latency_to(&self, id: &NodeId) -> Option<Latency> {
        match self.node_situation.get_fog_node_neighbor(id) {
            None => None,
            Some(NodeDescription { latency, .. }) => Some(Latency {
                median:              latency,
                average:             latency,
                interquantile_range: Time::new::<second>(0.0),
                packet_loss:         Ratio::new::<ratio>(0.0),
            }),
        }
    }
}

impl LatencyEstimationOfflineImpl {
    pub fn new(node_situation: Arc<NodeSituation>) -> Self {
        Self { node_situation }
    }
}
