use super::super::domain::sla::Sla;
use super::super::NodeId;
use crate::domain::sla::DataFlow;
use helper::uom_helper::{information, ratio, time};
use serde::{Deserialize, Serialize};
use uom::si::f64::{Information, Ratio, Time};

/// Describe the SLA of a function submitted to be provisioned
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SlaRequest {
    // #[serde_as(as = "information::Helper")]
    // pub storage: Information,
    #[serde_as(as = "information::Helper")]
    pub memory: Information,

    #[serde_as(as = "ratio::Helper")]
    pub cpu: Ratio,

    #[serde_as(as = "time::Helper")]
    pub latency_max: Time,

    // #[serde_as(as = "information::Helper")]
    // pub data_input_max_size: Information,
    // #[serde_as(as = "information::Helper")]
    // pub data_output_max_size: Information,
    // #[serde_as(as = "time::Helper")]
    // pub max_time_before_hot: Time,
    #[serde_as(as = "time::Helper")]
    pub duration: Time,

    pub max_replica: u64,

    pub function_image: String,

    pub function_live_name: String,

    pub data_flow: Vec<DataFlow>,
}

impl Into<Sla> for SlaRequest {
    fn into(self) -> Sla {
        Sla {
            id:                 uuid::Uuid::new_v4().into(),
            memory:             self.memory,
            cpu:                self.cpu,
            latency_max:        self.latency_max,
            duration:           self.duration,
            max_replica:        self.max_replica,
            function_image:     self.function_image,
            function_live_name: self.function_live_name,
            data_flow:          self.data_flow,
        }
    }
}

/// Structure used to register a SLA, starts the auctionning process and
/// establish the routing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PutSla {
    pub sla:         SlaRequest,
    pub target_node: NodeId,
}
