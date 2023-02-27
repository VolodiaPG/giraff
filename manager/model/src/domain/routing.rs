use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::NodeId;

/// Describe a Route from a Fog node to another in the network
#[derive(Debug)]
pub struct RoutingStacks {
    pub least_common_ancestor: NodeId,
    /// Stack to read from start to finish
    /// eg. `[a, b]`, establish `a -> b` while travelling from a to b
    pub stack_asc:             Vec<NodeId>,
    /// Stack to read from finish to start
    /// eg. `[a, b]`, establish `b -> a` while travelling from a to b
    pub stack_rev:             Vec<NodeId>,
}

/// A path between two points in the Fog
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FogSegment {
    pub from: NodeId,
    pub to:   NodeId,
}

/// [PacketPacket] with its direction:
/// - [Packet::FaaSFunction] directs to the hosted faaSFunction
/// - [Packet::FogNode] directs to the fog node itself (at the start of the
///   routing stack transmitted)
/// - [Packet::Market] directs to the market
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Packet {
    FogNode {
        route_to_stack: Vec<NodeId>,
        resource_uri:   String,
        data:           Value,
    },
    Market {
        resource_uri: String,
        data:         Value,
    },
}
