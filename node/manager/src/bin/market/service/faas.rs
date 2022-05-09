use async_trait::async_trait;
use manager::model::dto::auction::ChosenBid;
use std::fmt::Debug;
use std::sync::Arc;

use crate::repository::fog_node::FogNode;
use crate::repository::node_communication::NodeCommunication;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    NodeCommunication(#[from] crate::repository::node_communication::Error),
}

#[async_trait]
pub trait FogNodeFaaS: Debug + Sync + Send {
    async fn provision_function(&self, bid: &ChosenBid) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct FogNodeFaaSImpl {
    fog_node: Arc<dyn FogNode>,
    node_communication: Arc<dyn NodeCommunication>,
}

impl FogNodeFaaSImpl {
    pub fn new(fog_node: Arc<dyn FogNode>, node_communication: Arc<dyn NodeCommunication>) -> Self {
        Self {
            fog_node,
            node_communication,
        }
    }
}

#[async_trait]
impl FogNodeFaaS for FogNodeFaaSImpl {
    async fn provision_function(&self, bid: &ChosenBid) -> Result<(), Error> {
        // TODO implemented CRUD operations for fog node
        Ok(self
            .node_communication
            .take_offer(bid.bid.node_id.clone(), &bid.bid)
            .await?)
    }
}
