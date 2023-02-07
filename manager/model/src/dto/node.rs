use std::collections::HashMap;
use std::fmt;
use std::net::IpAddr;

use helper::uom_helper::{information, ratio};
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use uom::si::f64::{Information, Ratio};

use crate::view::auction::AcceptedBid;
use crate::{
    BidId, FogNodeFaaSPortExternal, FogNodeHTTPPort, MarketHTTPPort, NodeId,
};

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
    pub port_faas:     FogNodeFaaSPortExternal,
    pub tags:          Vec<String>,
    pub accepted_bids: HashMap<BidId, AcceptedBid>,
}

impl NodeRecord {
    pub fn new(
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_faas: FogNodeFaaSPortExternal,
        tags: &[String],
    ) -> Self {
        Self {
            ip,
            port_http,
            port_faas,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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
    pub my_public_port_faas: FogNodeFaaSPortExternal,
    pub tags:                Vec<String>,
    pub reserved_memory:     Information,
    pub reserved_cpu:        Ratio,
    pub children:            dashmap::DashMap<NodeId, NodeDescription>,
}

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeSituationDisk {
    pub situation:           NodeCategory,
    pub my_id:               NodeId,
    pub my_public_ip:        IpAddr,
    pub my_public_port_http: FogNodeHTTPPort,
    pub tags:                Vec<String>,
    #[serde_as(as = "information::Helper")]
    pub reserved_memory:     Information,
    #[serde_as(as = "ratio::Helper")]
    pub reserved_cpu:        Ratio,
}

/// Loads from a file objects of the form:
impl NodeSituationDisk {
    pub fn new(content: String) -> anyhow::Result<Self> {
        let situation = ron::from_str::<NodeSituationDisk>(&content)?;
        Ok(situation)
    }
}

impl NodeSituationData {
    pub fn new(
        disk: NodeSituationDisk,
        my_public_port_faas: FogNodeFaaSPortExternal,
    ) -> Self {
        let NodeSituationDisk {
            situation,
            my_id,
            my_public_ip,
            my_public_port_http,
            tags,
            reserved_memory,
            reserved_cpu,
        } = disk;

        Self {
            situation,
            my_id,
            my_public_ip,
            my_public_port_http,
            my_public_port_faas,
            tags,
            reserved_memory,
            reserved_cpu,
            children: dashmap::DashMap::new(),
        }
    }
}
