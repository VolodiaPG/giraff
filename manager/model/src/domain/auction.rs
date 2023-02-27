use serde::{Deserialize, Serialize};

use crate::dto::auction::ChosenBid;
use crate::view::auction::BidProposal;

#[derive(Debug)]
pub enum AuctionStatus {
    Active(Vec<BidProposal>),
    Finished(BidProposal),
}

#[derive(Debug)]
pub struct AuctionSummary {
    pub status: AuctionStatus,
}

/// Sums up all proposal received by the market
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AuctionResult {
    pub chosen_bid: ChosenBid,
}
