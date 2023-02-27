use crate::domain::sla::Sla;
use crate::view::auction::BidProposal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidRecord {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChosenBid {
    pub bid:   BidProposal,
    pub price: f64,
}
