use schemars::gen::SchemaGenerator;
use schemars::schema::{InstanceType, Schema, SchemaObject};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;

use crate::model::{BidId, NodeId};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct FunctionRoutingStack {
    pub function: BidId,

    /// Route to the first node where the route need to be registered
    pub route_to_first: Vec<NodeId>,

    /// Route to be registered, starting at the [route_to_first_node]
    pub routes: Vec<NodeId>,
}

/// [PacketPacket] with its direction:
/// - [Packet::FaaSFunction] directs to the hosted faaSFunction
/// - [Packet::FogNode] directs to the fog node itself (at the start of the routing stack
///   transmitted)
/// - [Packet::Market] directs to the market
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Packet<'a> {
    FaaSFunction {
        to:   BidId,
        #[serde(borrow)]
        #[schemars(schema_with = "schema_function")]
        data: &'a RawValue,
    },
    FogNode {
        route_to_stack: Vec<NodeId>,
        resource_uri:   String,
        #[serde(borrow)]
        #[schemars(schema_with = "schema_function")]
        data:           &'a RawValue,
    },
    Market {
        resource_uri: String,
        #[serde(borrow)]
        #[schemars(schema_with = "schema_function")]
        data:         &'a RawValue,
    },
}

pub fn schema_function(_: &mut SchemaGenerator) -> Schema {
    SchemaObject { instance_type: Some(InstanceType::Object.into()), ..Default::default() }.into()
}
