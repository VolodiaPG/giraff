use std::cmp::Ordering;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::super::domain::sla::Sla;
use super::super::{BidId, NodeId};

/// A bid
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id: BidId,
}

/// The accepted bid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptedBid {
    pub sla: Sla,
    pub bid: BidProposal,
}

/// The bid proposal and the node who issued it
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BidProposal {
    pub node_id: NodeId,
    pub id: BidId,
    pub bid: f64,
}

impl PartialOrd for BidProposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bid.partial_cmp(&other.bid)
    }
}

impl PartialEq for BidProposal {
    fn eq(&self, other: &Self) -> bool {
        self.bid == other.bid
    }
}

impl Eq for BidProposal {}
