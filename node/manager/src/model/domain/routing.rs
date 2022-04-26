use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::model::{BidId, NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct RoutingStack {
    pub function: BidId,

    /// Route to the first node where the route need to be registered
    pub route_to_first: Vec<NodeId>,

    /// Route to be registered, starting at the [route_to_first_node]
    pub routes: Vec<NodeId>,
}
