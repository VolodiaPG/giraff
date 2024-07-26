use super::super::domain::sla::Sla;
use super::super::NodeId;
use crate::domain::sla::DataFlow;
use helper::uom_helper::{cpu, information, time};
use serde::{Deserialize, Serialize};
use uom::si::f64::Time;
use uom::si::rational64::{Information, Ratio};

/// Describe the SLA of a function submitted to be provisioned
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SlaRequest {
    // #[serde_as(as = "information::Helper")]
    // pub storage: Information,
    #[serde_as(as = "information::Helper")]
    pub memory: Information,

    #[serde_as(as = "cpu::Helper")]
    pub cpu: Ratio,

    #[serde_as(as = "time::Helper")]
    pub latency_max: Time,

    #[serde_as(as = "information::Helper")]
    pub input_max_size: Information,
    // #[serde_as(as = "information::Helper")]
    // pub data_output_max_size: Information,
    // #[serde_as(as = "time::Helper")]
    // pub max_time_before_hot: Time,
    #[serde_as(as = "time::Helper")]
    pub duration:       Time,

    pub max_replica: u64,

    pub function_image: String,

    pub function_live_name: String,

    pub data_flow: Vec<DataFlow>,

    pub env_vars: Option<Vec<(String, String)>>,
}

impl From<SlaRequest> for Sla {
    fn from(val: SlaRequest) -> Self {
        Sla {
            id:                 uuid::Uuid::new_v4().into(),
            memory:             val.memory,
            cpu:                val.cpu,
            latency_max:        val.latency_max,
            duration:           val.duration,
            max_replica:        val.max_replica,
            function_image:     val.function_image,
            function_live_name: val.function_live_name,
            data_flow:          val.data_flow,
            env_vars:           val.env_vars.unwrap_or_default(),
            input_max_size:     val.input_max_size,
        }
    }
}

/// Structure used to register a SLA, starts the auctionning process and
/// establish the routing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PutSlaRequest {
    pub sla:         SlaRequest,
    pub target_node: NodeId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PutSla {
    pub sla:         Sla,
    pub target_node: NodeId,
}

impl From<PutSlaRequest> for PutSla {
    fn from(value: PutSlaRequest) -> Self {
        PutSla {
            sla:         value.sla.into(),
            target_node: value.target_node,
        }
    }
}
