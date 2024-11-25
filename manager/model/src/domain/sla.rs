use crate::{NodeId, SlaId};
use helper::uom_helper::{cpu, information, time};
use serde::{Deserialize, Serialize};
use uom::si::f64::Time;
use uom::si::rational64::{Information, Ratio};

/// Describe the SLA of a function submitted to be provisioned
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Sla {
    pub id: SlaId,

    #[serde_as(as = "information::Helper")]
    pub memory: Information,

    #[serde_as(as = "cpu::Helper")]
    pub cpu: Ratio,

    #[serde_as(as = "time::Helper")]
    pub latency_max: Time,

    #[serde_as(as = "time::Helper")]
    pub duration: Time,

    pub replicas: u64,

    pub function_image: String,

    pub function_live_name: String,

    pub data_flow: Vec<DataFlow>,

    pub env_vars: Vec<(String, String)>,

    #[serde_as(as = "information::Helper")]
    pub input_max_size: Information,
}

/// A point in the Fog
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub enum SlaFogPoint {
    ThisFunction,
    DataSource(NodeId),
    //FunctionSink(String), // Livename
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DataFlow {
    pub from: SlaFogPoint,
    pub to:   SlaFogPoint,
}
