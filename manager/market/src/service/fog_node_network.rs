use crate::repository::fog_node::FogNode;
use anyhow::{Context, Result};
use model::dto::node::NodeRecord;
use model::view::node::RegisterNode;
use model::NodeId;
use std::fmt::Debug;
use std::sync::Arc;

#[derive(Debug)]
pub struct FogNodeNetwork {
    fog_node: Arc<FogNode>,
}

impl FogNodeNetwork {
    pub fn new(fog_node: Arc<FogNode>) -> Self { Self { fog_node } }

    pub async fn register_node(&self, node: RegisterNode) -> Result<()> {
        match node {
            RegisterNode::MarketNode {
                node_id,
                ip,
                port_http,
                port_faas,
                tags,
                ..
            } => {
                self.fog_node
                    .append_root(
                        node_id.clone(),
                        ip,
                        port_http,
                        port_faas,
                        &tags,
                    )
                    .await
                    .with_context(|| {
                        format!("Failed to register market node {}", node_id)
                    })?;
            }
            RegisterNode::Node {
                node_id,
                parent,
                tags,
                ip,
                port_http,
                port_faas,
                ..
            } => {
                self.fog_node
                    .append_new_child(
                        &parent,
                        node_id.clone(),
                        ip,
                        port_http,
                        port_faas,
                        &tags,
                    )
                    .await
                    .with_context(|| {
                        format!(
                            "Failed to register child node {} whose parent \
                             should be {}",
                            node_id, parent
                        )
                    })?;
            }
        }

        Ok(())
    }

    pub async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)> {
        self.fog_node.get_nodes().await
    }

    pub async fn get_node(&self, node: &NodeId) -> Option<NodeRecord> {
        self.fog_node.get(node).await.map(|x| x.data)
    }
}
