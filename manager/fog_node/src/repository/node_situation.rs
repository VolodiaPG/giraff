use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;

use model::dto::node::NodeCategory::{MarketConnected, NodeConnected};
use model::dto::node::{NodeDescription, NodeSituationData};
use model::NodeId;

#[async_trait]
pub trait NodeSituation: Debug + Sync + Send {
    async fn register(&self, id: NodeId, description: NodeDescription);
    /// Get a node: children, parent
    async fn get_fog_node_neighbor(
        &self,
        id: &NodeId,
    ) -> Option<NodeDescription>;
    async fn get_my_id(&self) -> NodeId;
    async fn get_parent_id(&self) -> Option<NodeId>;
    async fn get_my_tags(&self) -> Vec<String>;
    /// Whether the node is connected to the market (i.e., doesn't have any
    /// parent = root of the network)
    async fn is_market(&self) -> bool;
    async fn get_parent_node_address(&self) -> Option<(IpAddr, u16)>;
    async fn get_market_node_address(&self) -> Option<(IpAddr, u16)>;
    /// Return iter over both the parent and the children node...
    /// Aka all the nodes interesting that can accommodate a function
    async fn get_neighbors(&self) -> Vec<NodeId>;
    /// Get the public ip associated with this server
    async fn get_my_public_ip(&self) -> IpAddr;
    /// Get the public port associated with this server
    async fn get_my_public_port(&self) -> u16;
}

#[derive(Debug)]
pub struct NodeSituationHashSetImpl {
    database: NodeSituationData,
}

impl NodeSituationHashSetImpl {
    pub fn new(situation: NodeSituationData) -> Self {
        Self { database: situation }
    }
}

#[async_trait]
impl NodeSituation for NodeSituationHashSetImpl {
    async fn register(&self, id: NodeId, description: NodeDescription) {
        self.database.children.pin().insert(id, description);
    }

    async fn get_fog_node_neighbor(
        &self,
        id: &NodeId,
    ) -> Option<NodeDescription> {
        let ret = self.database.children.pin().get(id).cloned();
        if ret.is_none() {
            match &self.database.situation {
                NodeConnected {
                    parent_node_ip,
                    parent_node_port,
                    parent_id,
                    ..
                } => {
                    if parent_id == id {
                        return Some(NodeDescription {
                            ip:   parent_node_ip.clone(),
                            port: parent_node_port.clone(),
                        });
                    }
                }
                MarketConnected { .. } => (),
            }
        }
        ret
    }

    async fn get_my_id(&self) -> NodeId { self.database.my_id.clone() }

    async fn get_parent_id(&self) -> Option<NodeId> {
        match &self.database.situation {
            NodeConnected { parent_id, .. } => Some(parent_id.clone()),
            _ => None,
        }
    }

    async fn get_my_tags(&self) -> Vec<String> { self.database.tags.clone() }

    async fn is_market(&self) -> bool {
        matches!(self.database.situation, MarketConnected { .. })
    }

    async fn get_parent_node_address(&self) -> Option<(IpAddr, u16)> {
        match self.database.situation {
            NodeConnected { parent_node_ip, parent_node_port, .. } => {
                Some((parent_node_ip.clone(), parent_node_port.clone()))
            }
            _ => None,
        }
    }

    async fn get_market_node_address(&self) -> Option<(IpAddr, u16)> {
        match self.database.situation {
            MarketConnected { market_ip, market_port, .. } => {
                Some((market_ip.clone(), market_port.clone()))
            }
            _ => None,
        }
    }

    async fn get_neighbors(&self) -> Vec<NodeId> {
        let mut ret: Vec<NodeId> =
            self.database.children.pin().keys().cloned().collect();

        match &self.database.situation {
            NodeConnected { parent_id, .. } => {
                ret.push(parent_id.clone());
            }
            MarketConnected { .. } => (),
        }

        ret
    }

    async fn get_my_public_ip(&self) -> IpAddr { self.database.my_public_ip }

    async fn get_my_public_port(&self) -> u16 { self.database.my_public_port }
}
