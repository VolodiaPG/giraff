use async_trait::async_trait;
use model::dto::node::NodeRecord;
use model::view::auction::AcceptedBid;
use model::NodeId;
use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::repository::fog_node::FogNode;
use crate::repository::node_communication::NodeCommunication;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeCommunication(#[from] crate::repository::node_communication::Error),
    #[error(
        "No trace of the node {0} has been found. It should have been \
         registered as a record though."
    )]
    NodeNotFound(NodeId),
    #[error(transparent)]
    FogNetwork(#[from] crate::service::fog_node_network::Error),
}

#[async_trait]
pub trait FogNodeFaaS: Debug + Sync + Send {
    /// Provision the function on the given node and establish the routes using
    /// [`establish_route`](FogNodeFaaS::establish_route)
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error>;

    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>>;
}

#[derive(Debug)]
pub struct FogNodeFaaSImpl {
    fog_node:           Arc<dyn FogNode>,
    node_communication: Arc<dyn NodeCommunication>,
}

impl FogNodeFaaSImpl {
    pub fn new(
        fog_node: Arc<dyn FogNode>,
        node_communication: Arc<dyn NodeCommunication>,
    ) -> Self {
        Self { fog_node, node_communication }
    }
}

#[async_trait]
impl FogNodeFaaS for FogNodeFaaSImpl {
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error> {
        trace!("Provisioning function...");

        let node = bid.chosen.bid.node_id.clone();
        self.node_communication
            .take_offer(node.clone(), &bid.chosen.bid)
            .await?;

        let mut record: NodeRecord = self
            .fog_node
            .get(&node)
            .await
            .map(|node| node.data)
            .ok_or_else(|| Error::NodeNotFound(node.clone()))?;

        let id = bid.chosen.bid.id.clone();
        record.accepted_bids.insert(id.clone(), bid);
        self.fog_node.update(&node, record).await;

        Ok(())
    }

    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        self.fog_node.get_records().await
    }
}
