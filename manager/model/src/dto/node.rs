use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::view::auction::AcceptedBid;
use crate::{BidId, NodeId};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node<T> {
    pub parent:   Option<NodeId>,
    pub children: Vec<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeRecord {
    /// URI, only in the case of the market node
    pub ip:            Option<IpAddr>,
    pub port:          Option<u16>,
    pub tags:          Vec<String>,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
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
    fn from(list: Vec<NodeId>) -> Self { NodeIdList { list } }
}

#[derive(Debug, Clone)]
pub struct NodeDescription {
    pub ip:   IpAddr,
    pub port: u16,
}

#[derive(Debug)]
pub enum NodeSituationData {
    MarketConnected {
        children:       HashMap<NodeId, NodeDescription>,
        market_ip:      IpAddr,
        market_port:    u16,
        my_id:          NodeId,
        my_public_ip:   IpAddr,
        my_public_port: u16,
        tags:           Vec<String>,
    },
    NodeConnected {
        children:         HashMap<NodeId, NodeDescription>,
        parent_id:        NodeId,
        parent_node_ip:   IpAddr,
        parent_node_port: u16,
        my_id:            NodeId,
        my_public_ip:     IpAddr,
        my_public_port:   u16,
        tags:             Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSituationDisk {
    MarketConnected {
        market_ip:      IpAddr,
        market_port:    u16,
        my_id:          NodeId,
        my_public_ip:   IpAddr,
        my_public_port: u16,
        tags:           Vec<String>,
    },
    NodeConnected {
        parent_id:        NodeId,
        parent_node_ip:   IpAddr,
        parent_node_port: u16,
        my_id:            NodeId,
        my_public_ip:     IpAddr,
        my_public_port:   u16,
        tags:             Vec<String>,
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
    pub fn new(content: String) -> anyhow::Result<Self> {
        let situation = ron::from_str::<NodeSituationDisk>(&content)?;
        Ok(situation)
    }
}

impl From<NodeSituationDisk> for NodeSituationData {
    fn from(disk: NodeSituationDisk) -> Self {
        match disk {
            NodeSituationDisk::MarketConnected {
                market_port,
                market_ip,
                my_id,
                my_public_ip,
                my_public_port,
                tags,
            } => NodeSituationData::MarketConnected {
                children: HashMap::new(),
                market_ip,
                market_port,
                my_id,
                my_public_ip,
                my_public_port,
                tags,
            },
            NodeSituationDisk::NodeConnected {
                parent_id,
                parent_node_port,
                parent_node_ip,
                my_id,
                my_public_ip,
                my_public_port,
                tags,
            } => NodeSituationData::NodeConnected {
                children: HashMap::new(),
                parent_id,
                parent_node_ip,
                parent_node_port,
                my_id,
                my_public_ip,
                my_public_port,
                tags,
            },
        }
    }
}
