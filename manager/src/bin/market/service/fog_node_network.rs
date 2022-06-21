use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use manager::model::view::node::RegisterNode;

use crate::repository::fog_node::FogNode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeUpdate(#[from] crate::repository::fog_node::Error),
}

#[async_trait]
pub trait FogNodeNetwork: Debug + Sync + Send {
    async fn register_node(&self, node: RegisterNode) -> Result<(), Error>;
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
            RegisterNode::MarketNode { node_id, ip, port } => {
                self.fog_node.append_root(node_id, ip, port).await?;
            }
            RegisterNode::Node {
                node_id, parent, ..
            } => {
                self.fog_node.append_new_child(&parent, node_id).await?;
            }
        }

        Ok(())
    }
}
