use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

use crate::model::domain::rolling_avg::RollingAvg;
use crate::model::domain::sla::Sla;
use crate::model::view::auction::{AcceptedBid, BidProposal};
use crate::model::{BidId, NodeId};

#[derive(Debug, Deserialize, Clone)]
pub struct Node<T> {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeRecord {
    pub ip: String,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NodeRecordDisk {
    pub ip: String,
}

impl From<NodeRecordDisk> for NodeRecord {
    fn from(disk: NodeRecordDisk) -> Self {
        NodeRecord {
            ip: disk.ip,
            ..Default::default()
        }
    }
}

impl From<Node<NodeRecordDisk>> for Node<NodeRecord> {
    fn from(disk: Node<NodeRecordDisk>) -> Self {
        Node {
            parent: disk.parent,
            children: disk.children,
            data: NodeRecord::from(disk.data),
        }
    }
}

#[derive(Debug)]
pub struct NodeIdList {
    list: Vec<NodeId>,
}

impl fmt::Display for NodeIdList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<NodeId>> for NodeIdList {
    fn from(list: Vec<NodeId>) -> Self {
        NodeIdList { list }
    }
}
