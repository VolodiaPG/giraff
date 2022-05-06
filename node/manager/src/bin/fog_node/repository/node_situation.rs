use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;
use tokio::sync::RwLock;

use manager::model::dto::node::NodeSituationData::{MarketConnected, NodeConnected};
use manager::model::dto::node::{NodeDescription, NodeSituationData};
use manager::model::NodeId;

#[async_trait]
pub trait NodeSituation: Debug + Sync + Send {
    async fn register(&self, id: NodeId, description: NodeDescription);
    /// Get a node: children, parent
    async fn get_fog_node_neighbor(&self, id: &NodeId) -> Option<NodeDescription>;
    async fn get_my_id(&self) -> NodeId;
    async fn get_parent_id(&self) -> Option<NodeId>;
    /// Whether the node is connected to the market (i.e., doesn't have any parent = root of the network)
    async fn is_market(&self) -> bool;
    async fn get_parent_node_address(&self) -> Option<(IpAddr, u16)>;
    async fn get_market_node_address(&self) -> Option<(IpAddr, u16)>;
    /// Return iter over both the parent and the children node...
    /// Aka all the nodes interesting that can accommodate a function
    async fn get_neighbors(&self) -> Vec<NodeId>;
}

#[derive(Debug)]
pub struct NodeSituationHashSetImpl {
    database: RwLock<NodeSituationData>,
}

impl NodeSituationHashSetImpl {
    pub fn new(situation: NodeSituationData) -> Self {
        Self {
            database: RwLock::new(situation),
        }
    }
}

#[async_trait]
impl NodeSituation for NodeSituationHashSetImpl {
    async fn register(&self, id: NodeId, description: NodeDescription) {
        match &mut *self.database.write().await {
            MarketConnected { children, .. } | NodeConnected { children, .. } => {
                children.insert(id, description);
            }
        }
    }

    async fn get_fog_node_neighbor(&self, id: &NodeId) -> Option<NodeDescription> {
        match &*self.database.read().await {
            MarketConnected { children, .. } => children.get(id).cloned(),
            NodeConnected {
                children,
                parent_node_ip,
                parent_node_port,
                parent_id,
                ..
            } => {
                let ret = children.get(id).cloned();
                if ret.is_none() && parent_id == id {
                    return Some(NodeDescription {
                        ip: *parent_node_ip,
                        port: *parent_node_port,
                    });
                }
                ret
            }
        }
    }

    async fn get_my_id(&self) -> NodeId {
        match &*self.database.read().await {
            MarketConnected { my_id, .. } | NodeConnected { my_id, .. } => my_id.clone(),
        }
    }

    async fn get_parent_id(&self) -> Option<NodeId> {
        match &*self.database.read().await {
            NodeConnected { parent_id, .. } => Some(parent_id.clone()),
            _ => None,
        }
    }

    async fn is_market(&self) -> bool {
        matches!(&*self.database.read().await, MarketConnected { .. })
    }

    async fn get_parent_node_address(&self) -> Option<(IpAddr, u16)> {
        match &*self.database.read().await {
            NodeConnected {
                parent_node_ip,
                parent_node_port,
                ..
            } => Some((*parent_node_ip, *parent_node_port)),
            _ => None,
        }
    }

    async fn get_market_node_address(&self) -> Option<(IpAddr, u16)> {
        match &*self.database.read().await {
            MarketConnected {
                market_ip,
                market_port,
                ..
            } => Some((*market_ip, *market_port)),
            _ => None,
        }
    }

    async fn get_neighbors(&self) -> Vec<NodeId> {
        match &*self.database.read().await {
            MarketConnected { children, .. } => children.keys().cloned().collect(),
            NodeConnected {
                parent_id,
                children,
                ..
            } => vec![parent_id.clone()]
                .into_iter()
                .chain(children.keys().cloned())
                .collect(),
        }
    }
}
