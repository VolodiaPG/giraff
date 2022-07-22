use crate::model::{BidId, NodeId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

/// Describe a Route from a Fog node to another in the network
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "camelCase")]
pub struct Route {
    /// Stack to read from start to finish
    /// eg. `[a, b]`, establish `a -> b` while travelling from a to b
    pub stack_asc: Vec<NodeId>,
    /// Stack to read from finish to start
    /// eg. `[a, b]`, establish `b -> a` while travelling from a to b
    pub stack_rev: Vec<NodeId>,
    /// The function
    pub function:  BidId,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RouteDirection {
    StartToFinish,
    // prev last node is the node that comes just before the last node
    FinishToStart { prev_last_node: NodeId },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RouteLinking {
    /// The destination is at the end
    pub stack:     VecDeque<NodeId>,
    pub direction: RouteDirection,
    pub function:  BidId,
}
