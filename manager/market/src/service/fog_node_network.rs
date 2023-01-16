use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use model::dto::node::NodeRecord;
use model::view::node::RegisterNode;
use model::NodeId;

use crate::repository::fog_node::FogNode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeUpdate(#[from] crate::repository::fog_node::Error),
}

#[async_trait]
pub trait FogNodeNetwork: Debug + Sync + Send {
    async fn register_node(&self, node: RegisterNode) -> Result<(), Error>;

    /// Get all the connected nodes
    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)>; // TODO danger !

    /// Get all the connected nodes
    async fn get_node(&self, node: &NodeId) -> Option<NodeRecord>;
}

#[derive(Debug)]
pub struct FogNodeNetworkHashTreeImpl {
    fog_node: Arc<dyn FogNode>,
}

impl FogNodeNetworkHashTreeImpl {
    pub fn new(fog_node: Arc<dyn FogNode>) -> Self {
        FogNodeNetworkHashTreeImpl { fog_node }
    }
}

#[async_trait]
impl FogNodeNetwork for FogNodeNetworkHashTreeImpl {
    async fn register_node(&self, node: RegisterNode) -> Result<(), Error> {
        match node {
            RegisterNode::MarketNode {
                node_id,
                ip,
                port_http,
                port_faas,
                tags,
            } => {
                self.fog_node
                    .append_root(node_id, ip, port_http, port_faas, &tags)
                    .await?;
            }
            RegisterNode::Node {
                node_id,
                parent,
                tags,
                ip,
                port_http,
                port_faas,
            } => {
                self.fog_node
                    .append_new_child(
                        &parent, node_id, ip, port_http, port_faas, &tags,
                    )
                    .await?;
            }
        }

        Ok(())
    }

    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)> {
        self.fog_node.get_nodes().await
    }

    async fn get_node(&self, node: &NodeId) -> Option<NodeRecord> {
        self.fog_node.get(node).await.map(|x| x.data)
    }
}
