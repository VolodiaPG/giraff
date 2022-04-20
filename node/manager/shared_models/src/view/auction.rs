use std::cmp::Ordering;

use crate::domain::sla::Sla;

use crate::{BidId, NodeId};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A bid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id: BidId,
}

/// Sums up all proposal received by the market
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct MarketBidProposal {
    pub bids: Vec<BidProposal>,
    pub chosen_bid: Option<BidProposal>,
    pub price: Option<f64>,
}

/// The accepted bid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptedBid {
    pub sla: Sla,
    pub bid: BidProposal,
}

/// The bid proposal and the node who issued it
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
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
