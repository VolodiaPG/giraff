use std::collections::HashMap;
use std::fmt;
use std::fs;

use log::trace;
use serde::{Deserialize, Serialize};

use crate::model::view::auction::AcceptedBid;
use crate::model::{BidId, NodeId};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node<T> {
    pub parent: Option<NodeId>,
    pub children: Vec<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeRecord {
    /// URI, only in the case of the market node
    pub uri: Option<String>,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct NodeRecordDisk {
//     pub ip: String,
// }
//
// impl From<NodeRecordDisk> for NodeRecord {
//     fn from(disk: NodeRecordDisk) -> Self {
//         NodeRecord {
//             ip: disk.ip,
//             ..Default::default()
//         }
//     }
// }
//
// impl From<Node<NodeRecordDisk>> for Node<NodeRecord> {
//     fn from(disk: Node<NodeRecordDisk>) -> Self {
//         Node {
//             parent: disk.parent,
//             children: disk.children,
//             data: NodeRecord::from(disk.data),
//         }
//     }
// }

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

#[derive(Debug, Clone)]
pub struct NodeDescription {
    pub uri: String,
}

#[derive(Debug)]
pub enum NodeSituationData {
    MarketConnected {
        children: HashMap<NodeId, NodeDescription>,
        market_uri: String,
        my_id: NodeId,
    },
    NodeConnected {
        children: HashMap<NodeId, NodeDescription>,
        parent_id: NodeId,
        parent_node_uri: String,
        my_id: NodeId,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSituationDisk {
    MarketConnected {
        market_uri: String,
        my_id: NodeId,
    },
    NodeConnected {
        parent_id: NodeId,
        parent_node_uri: String,
        my_id: NodeId,
    },
}

/// Loads from a file objects of the form:
/// ```ron
/// MarketConnected (
///    market_uri: "localhost:8080",
///    my_id: "e13f2a63-2934-480a-a448-b1b01af7e170",
/// )
/// ```
/// or another example:
/// ```ron
/// NodeConnected (
///   parent_id: "e13f2a63-2934-480a-a448-b1b01af7e170",
///   parent_node_uri: "localhost:8080",
///   my_id: "49aaea47-7af7-4c68-b29a-b445ef194d3a",
/// )
/// ```
impl NodeSituationDisk {
    pub fn new(path: String) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path.clone())?;
        let situation = ron::from_str::<NodeSituationDisk>(&content)?;
        trace!("Loading nodes from disk, path: {}", path);
        Ok(situation)
    }
}

impl From<NodeSituationDisk> for NodeSituationData {
    fn from(disk: NodeSituationDisk) -> Self {
        match disk {
            NodeSituationDisk::MarketConnected { market_uri, my_id } => {
                NodeSituationData::MarketConnected {
                    children: HashMap::new(),
                    market_uri,
                    my_id,
                }
            }
            NodeSituationDisk::NodeConnected {
                parent_id,
                parent_node_uri,
                my_id,
            } => NodeSituationData::NodeConnected {
                children: HashMap::new(),
                parent_id,
                parent_node_uri,
                my_id,
            },
        }
    }
}
