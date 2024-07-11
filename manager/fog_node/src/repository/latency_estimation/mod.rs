use anyhow::Result;
use async_trait::async_trait;
use model::NodeId;
use std::fmt::Debug;
use uom::si::f64::{Ratio, Time};

#[derive(Debug)]
pub struct Latency {
    pub median:              Time,
    pub average:             Time,
    pub interquantile_range: Time,
    pub packet_loss:         Ratio,
}

#[async_trait]
pub trait LatencyEstimation: Debug + Send + Sync {
    async fn get_latency_to(&self, id: &NodeId) -> Option<Latency>;
    async fn latency_to_neighbors(&self) -> Result<()>;
}

#[cfg(not(feature = "offline"))]
pub mod latency_estimation;
#[cfg(not(feature = "offline"))]
pub use latency_estimation::*;

#[cfg(feature = "offline")]
pub mod latency_estimation_offline;
#[cfg(feature = "offline")]
pub use latency_estimation_offline::*;
