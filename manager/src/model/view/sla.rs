use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::domain::sla::Sla;
use super::super::NodeId;

/// Structure used to register a SLA, starts the auctionning process and establish the routing
#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PutSla {
    pub sla:                  Sla,
    pub target_node:          NodeId,
    pub request_sources:      Vec<NodeId>,
    pub request_destinations: Vec<NodeId>,
}
