use super::NodeRecordDisk;
use serde::{Deserialize, Serialize};
use shared_models::{
    auction::{AcceptedBid, BidProposal},
    sla::Sla,
    BidId, RollingAvg,
};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum AuctionStatus {
    Active(Vec<BidProposal>),
    Finished(BidProposal),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidRecord {
    pub auction: AuctionStatus,
    pub sla: Sla,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeRecord {
    pub ip: String,
    pub latency: RollingAvg,
    pub accepted_bids: HashMap<BidId, AcceptedBid>, // TODO change name
}

impl From<NodeRecordDisk> for NodeRecord {
    fn from(disk: NodeRecordDisk) -> Self {
        NodeRecord {
            ip: disk.ip,
            ..Default::default()
        }
    }
}
