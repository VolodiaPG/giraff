use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;

use serde::{Deserialize, Serialize};

use crate::view::auction::AcceptedBid;
use crate::{BidId, FogNodeHTTPPort, MarketHTTPPort, NodeId};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Node<T> {
    pub parent:   Option<NodeId>,
    pub children: Vec<NodeId>,

    /// The actual data which will be stored within the tree
    pub data: T,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeRecord {
    /// URI, only in the case of the market node
    pub ip:            IpAddr,
    pub port_http:     FogNodeHTTPPort,
    pub tags:          Vec<String>,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
}

impl NodeRecord {
    pub fn new(
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        tags: &[String],
    ) -> Self {
        Self {
            ip,
            port_http,
            tags: Vec::from(tags),
            accepted_bids: HashMap::new(),
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
    fn from(list: Vec<NodeId>) -> Self { NodeIdList { list } }
}

#[derive(Debug, Clone)]
pub struct NodeDescription {
    pub ip:        IpAddr,
    pub port_http: FogNodeHTTPPort,
}

#[derive(Debug)]
pub enum NodeCategory {
    MarketConnected {
        market_ip:   IpAddr,
        market_port: MarketHTTPPort,
    },
    NodeConnected {
        parent_id:             NodeId,
        parent_node_ip:        IpAddr,
        parent_node_port_http: FogNodeHTTPPort,
    },
}

#[derive(Debug)]
pub struct NodeSituationData {
    pub situation:           NodeCategory,
    pub my_id:               NodeId,
    pub my_public_ip:        IpAddr,
    pub my_public_port_http: FogNodeHTTPPort,
    pub tags:                Vec<String>,
    pub children:            dashmap::DashMap<NodeId, NodeDescription>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NodeSituationDisk {
    MarketConnected {
        market_ip:           IpAddr,
        market_port:         MarketHTTPPort,
        my_id:               NodeId,
        my_public_ip:        IpAddr,
        my_public_port_http: FogNodeHTTPPort,
        tags:                Vec<String>,
    },
    NodeConnected {
        parent_id:             NodeId,
        parent_node_ip:        IpAddr,
        parent_node_port_http: FogNodeHTTPPort,
        my_id:                 NodeId,
        my_public_ip:          IpAddr,
        my_public_port_http:   FogNodeHTTPPort,
        tags:                  Vec<String>,
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
                my_public_port_http,
                tags,
            } => NodeSituationData {
                children: dashmap::DashMap::new(),
                my_id,
                my_public_ip,
                my_public_port_http,
                tags,
                situation: NodeCategory::MarketConnected {
                    market_ip,
                    market_port,
                },
            },
            NodeSituationDisk::NodeConnected {
                parent_id,
                parent_node_port_http,
                parent_node_ip,
                my_id,
                my_public_ip,
                my_public_port_http,
                tags,
            } => NodeSituationData {
                children: dashmap::DashMap::new(),
                my_id,
                my_public_ip,
                my_public_port_http,
                tags,
                situation: NodeCategory::NodeConnected {
                    parent_id,
                    parent_node_ip,
                    parent_node_port_http,
                },
            },
        }
    }
}
