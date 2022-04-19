use serde::{Deserialize, Serialize};
use uom::si::f64::{Information, Ratio, Time};

use crate::NodeId;

#[serde_with::serde_as]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Sla {
    #[serde(rename = "storage")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub storage: Information,

    #[serde(rename = "memory")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub memory: Information,

    #[serde(rename = "cpu")]
    #[serde_as(as = "uom_helpers::ratio::Helper")]
    pub cpu: Ratio,

    #[serde(rename = "latencyMax")]
    #[serde_as(as = "uom_helpers::time::Helper")]
    pub latency_max: Time,

    #[serde(rename = "dataInputMaxSize")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub data_input_max_size: Information,

    #[serde(rename = "dataOutputMaxSize")]
    #[serde_as(as = "uom_helpers::information::Helper")]
    pub data_output_max_size: Information,

    #[serde(rename = "maxTimeBeforeHot")]
    #[serde_as(as = "uom_helpers::time::Helper")]
    pub max_time_before_hot: Time,

    #[serde(rename = "reevaluationPeriod")]
    #[serde_as(as = "Option<uom_helpers::time::Helper>")]
    pub reevaluation_period: Option<Time>,

    #[serde(rename = "functionImage")]
    pub function_image: String,

    #[serde(rename = "functionLiveName")]
    pub function_live_name: Option<String>,
}

/// Structures used to register a SLA, starts the auctionning process and establish the routing
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PutSla {
    pub sla: Sla,
    #[serde(rename = "targetNode")]
    pub target_node: NodeId,
    #[serde(rename = "requestSources")]
    pub request_sources: Vec<NodeId>,
    #[serde(rename = "requestDestinations")]
    pub request_destinations: Vec<NodeId>,
}
