use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::HashMap;
use std::net::IpAddr;

use crate::helper::chrono as chrono_helper;
use crate::model::dto::node::NodeRecord;
use crate::model::view::auction::AcceptedBid;
use crate::model::BidId;

use super::super::NodeId;

/// Update information in the node node
/// - Must indicate the time the request was createdAt in order to update the rolling average
/// - Subsequent requests will also include the last_answered_at time, returned in [PostNodeResponse]
/// - Same for last_answered_at
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostNode {
    #[schemars(schema_with = "chrono_helper::schema_function")]
    #[serde_as(as = "chrono_helper::DateTimeHelper")]
    pub created_at: DateTime<Utc>,

    #[schemars(schema_with = "chrono_helper::schema_function")]
    #[serde_as(as = "Option<chrono_helper::DateTimeHelper>")]
    #[serde(default)]
    pub last_answered_at: Option<DateTime<Utc>>,

    #[schemars(schema_with = "chrono_helper::schema_function")]
    #[serde_as(as = "Option<chrono_helper::DateTimeHelper>")]
    #[serde(default)]
    pub last_answer_received_at: Option<DateTime<Utc>>,

    pub from: NodeId,
}

/// The answer to [PostNode]
#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PostNodeResponse {
    #[schemars(schema_with = "chrono_helper::schema_function")]
    #[serde_as(as = "chrono_helper::DateTimeHelper")]
    pub answered_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum RegisterNode {
    MarketNode {
        node_id: NodeId,
        ip: IpAddr,
        port: u16,
        tags: Vec<String>,
    },
    Node {
        parent: NodeId,
        node_id: NodeId,
        ip: IpAddr,
        port: u16,
        tags: Vec<String>,
    },
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct GetFogNodes {
    pub id: NodeId,
    pub tags: Vec<String>,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
}

impl From<(NodeId, NodeRecord)> for GetFogNodes {
    fn from((id, record): (NodeId, NodeRecord)) -> Self {
        GetFogNodes {
            id,
            tags: record.tags,
            accepted_bids: record.accepted_bids,
        }
    }
}
