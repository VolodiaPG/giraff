use serde::{Deserialize, Serialize};

use super::super::domain::sla::Sla;
use super::super::NodeId;

/// Structure used to register a SLA, starts the auctionning process and
/// establish the routing
#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PutSla {
    pub sla:         Sla,
    pub target_node: NodeId,
}
