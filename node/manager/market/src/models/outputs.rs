use serde::{Deserialize, Serialize};
use sla::Sla;

use crate::auction::Price;

use super::{BidId, BidProposal};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Satisfiable {
    #[serde(rename = "isSatisfiable")]
    pub is_satisfiable: bool,
    pub sla: Sla,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id: BidId,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MarketBidProposal {
    pub bids: Vec<BidProposal>,
    pub chosen_bid: Option<BidProposal>,
    pub price: Option<Price>,
}
