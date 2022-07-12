use crate::model::domain::sla::Sla;
use crate::model::view::auction::BidProposal;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidRecord {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ChosenBid {
    pub bid:   BidProposal,
    pub price: f64,
}
