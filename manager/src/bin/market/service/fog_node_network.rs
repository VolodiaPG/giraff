use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use futures::future::join;

use manager::model::domain::routing::{FogSegment, RoutingStacks};
use manager::model::dto::node::NodeRecord;
use manager::model::view::node::RegisterNode;
use manager::model::NodeId;

use crate::repository::fog_node::FogNode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeUpdate(#[from] crate::repository::fog_node::Error),
    #[error("No solution was found to route from {origin} to {dest}")]
    NoRoutingSolution { origin: NodeId, dest: NodeId },
}

#[async_trait]
pub trait FogNodeNetwork: Debug + Sync + Send {
    async fn register_node(&self, node: RegisterNode) -> Result<(), Error>;

    /// Get all the connected nodes
    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)>; // TODO danger !

    /// Get a solution to establish a [Route] between two points in the Fog
    /// network
    async fn get_route(
        &self,
        segment: FogSegment,
    ) -> Result<RoutingStacks, Error>;
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
            RegisterNode::MarketNode { node_id, ip, port, tags } => {
                self.fog_node.append_root(node_id, ip, port, tags).await?;
            }
            RegisterNode::Node { node_id, parent, tags, .. } => {
                self.fog_node.append_new_child(&parent, node_id, tags).await?;
            }
        }

        Ok(())
    }

    async fn get_nodes(&self) -> Vec<(NodeId, NodeRecord)> {
        self.fog_node.get_nodes().await
    }

    async fn get_route(
        &self,
        segment: FogSegment,
    ) -> Result<RoutingStacks, Error> {
        let path_to_from =
            self.fog_node.get_route_to_node(segment.from.clone());
        let path_to_to = self.fog_node.get_route_to_node(segment.to.clone());
        let (mut path_to_from, mut path_to_to) =
            join(path_to_from, path_to_to).await;

        trace!("{:?}", path_to_from);
        trace!("{:?}", path_to_to);

        // Remove common bits at the start (ie the end of the vec)
        let mut last_common_node = None;
        loop {
            let from = path_to_from.last();
            let to = path_to_to.last();

            match (from, to) {
                (Some(a), Some(b)) => {
                    if a.eq(b) {
                        path_to_from.pop();
                        last_common_node = path_to_to.pop();
                    } else {
                        break;
                    }
                }
                (None, None) | (None, Some(_)) | (Some(_), None) => break,
            }
        }

        if let Some(least_common_ancestor) = last_common_node {
            return Ok(RoutingStacks {
                least_common_ancestor,
                stack_rev: path_to_from,
                stack_asc: path_to_to,
            });
        }

        Err(Error::NoRoutingSolution {
            origin: segment.from,
            dest:   segment.to,
        })
    }
}
