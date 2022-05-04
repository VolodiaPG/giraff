use std::fmt::Debug;

use async_trait::async_trait;
use tokio::sync::RwLock;

use manager::model::dto::node::NodeSituationData::{MarketConnected, NodeConnected};
use manager::model::dto::node::{NodeDescription, NodeSituationData};
use manager::model::NodeId;

#[async_trait]
pub trait NodeSituation: Debug + Sync + Send {
    async fn register(&self, id: NodeId, description: NodeDescription);
    async fn get(&self, id: &NodeId) -> Option<NodeDescription>;
    async fn get_my_id(&self) -> NodeId;
    async fn get_parent_id(&self) -> Option<NodeId>;
    /// Whether the node is connected to the market (i.e., doesn't have any parent = root of the network)
    async fn is_market(&self) -> bool;
    async fn get_parent_node_uri(&self) -> Option<String>;
    async fn get_market_node_uri(&self) -> Option<String>;
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

    async fn get(&self, id: &NodeId) -> Option<NodeDescription> {
        match &*self.database.read().await {
            MarketConnected { children, .. } | NodeConnected { children, .. } => {
                children.get(id).cloned()
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
        match &*self.database.read().await {
            MarketConnected { .. } => true,
            _ => false,
        }
    }

    async fn get_parent_node_uri(&self) -> Option<String> {
        match &*self.database.read().await {
            NodeConnected {
                parent_node_uri, ..
            } => Some(parent_node_uri.clone()),
            _ => None,
        }
    }

    async fn get_market_node_uri(&self) -> Option<String> {
        match &*self.database.read().await {
            MarketConnected { market_uri, .. } => Some(market_uri.clone()),
            _ => None,
        }
    }
}
