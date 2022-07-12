use std::cmp::Ordering;

use crate::model::dto::auction::ChosenBid;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uom::si::f64::Time;

use super::super::domain::sla::Sla;
use super::super::{BidId, NodeId};

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BidRequest {
    pub node_origin:         NodeId,
    pub sla:                 Sla,
    #[schemars(schema_with = "crate::helper::uom::time::schema_function")]
    #[serde_as(as = "crate::helper::uom::time::Helper")]
    pub accumulated_latency: Time,
}

/// A bid
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id:  BidId,
}

/// The accepted bid
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct AcceptedBid {
    pub chosen:    ChosenBid,
    pub proposals: BidProposals,
}

/// The bid proposal and the node who issued it
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BidProposal {
    pub node_id: NodeId,
    pub id:      BidId,
    pub bid:     f64,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct BidProposals {
    pub bids: Vec<BidProposal>,
}

impl PartialOrd for BidProposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { self.bid.partial_cmp(&other.bid) }
}

impl PartialEq for BidProposal {
    fn eq(&self, other: &Self) -> bool { self.bid == other.bid }
}

impl Eq for BidProposal {}
