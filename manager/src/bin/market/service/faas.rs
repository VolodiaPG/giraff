use async_trait::async_trait;
use manager::model::dto::node::NodeRecord;
use manager::model::view::auction::AcceptedBid;
use manager::model::NodeId;
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
        "No trace of the node {0} has been found. It should have been registered as a record \
         though."
    )]
    NodeNotFound(NodeId),
}

#[async_trait]
pub trait FogNodeFaaS: Debug + Sync + Send {
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error>;
    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>>;
}

#[derive(Debug)]
pub struct FogNodeFaaSImpl {
    fog_node:           Arc<dyn FogNode>,
    node_communication: Arc<dyn NodeCommunication>,
}

impl FogNodeFaaSImpl {
    pub fn new(fog_node: Arc<dyn FogNode>, node_communication: Arc<dyn NodeCommunication>) -> Self {
        Self { fog_node, node_communication }
    }
}

#[async_trait]
impl FogNodeFaaS for FogNodeFaaSImpl {
    async fn provision_function(&self, bid: AcceptedBid) -> Result<(), Error> {
        let node = bid.chosen.bid.node_id.clone();
        let res = self.node_communication.take_offer(node.clone(), &bid.chosen.bid).await?;

        let mut record: NodeRecord = self
            .fog_node
            .get(&node)
            .await
            .map(|node| node.data)
            .ok_or_else(|| Error::NodeNotFound(node.clone()))?;
        record.accepted_bids.insert(bid.chosen.bid.id.clone(), bid);
        self.fog_node.update(&node, record).await;

        Ok(res)
    }

    async fn get_functions(&self) -> HashMap<NodeId, Vec<AcceptedBid>> {
        self.fog_node.get_records().await
    }
}
