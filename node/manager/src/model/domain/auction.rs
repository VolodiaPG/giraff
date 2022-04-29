use serde::{Deserialize, Serialize};

use crate::model::dto::auction::ChosenBid;
use crate::model::view::auction::BidProposal;
use schemars::JsonSchema;

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
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AuctionResult {
    pub bids: Vec<BidProposal>,
    pub chosen_bid: ChosenBid,
}
