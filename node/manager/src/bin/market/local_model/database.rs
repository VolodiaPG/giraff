use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use manager::model::{
    BidId,
    domain::{rolling_avg::RollingAvg, sla::Sla},
    view::auction::{AcceptedBid, BidProposal},
};

use super::NodeRecordDisk;

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
    pub latency_to_market: RollingAvg,
    pub latency_to_node: RollingAvg,
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
