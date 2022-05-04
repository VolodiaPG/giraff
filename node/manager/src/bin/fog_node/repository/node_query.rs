use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use manager::model::view::node::RegisterNode;

use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("The request failed with error code: {0}")]
    RequestStatus(reqwest::StatusCode),
    #[error("Failed to retrieve the URI to either the parent node or the market.")]
    NoURIToUpper,
}

#[async_trait]
pub trait NodeQuery: Debug + Sync + Send {
    /// Update the breadcrumb route to the [BidId] passing by the next [NodeId].
    async fn register_to_parent(&self, register: RegisterNode) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NodeQueryRESTImpl {
    node_situation: Arc<dyn NodeSituation>,
}

impl NodeQueryRESTImpl {
    pub fn new(node_situation: Arc<dyn NodeSituation>) -> Self {
        Self { node_situation }
    }
}

#[async_trait]
impl NodeQuery for NodeQueryRESTImpl {
    async fn register_to_parent(&self, register: RegisterNode) -> Result<(), Error> {
        trace!("Registering to parent or market...");
        let upper_uri;
        if !self.node_situation.is_market().await {
            upper_uri = self
                .node_situation
                .get_parent_node_uri()
                .await
                .ok_or(Error::NoURIToUpper)?;
        } else {
            upper_uri = self
                .node_situation
                .get_market_node_uri()
                .await
                .ok_or(Error::NoURIToUpper)?;
        }

        let client = reqwest::Client::new();
        // Both the market and node APIs offer the same endpoint.
        let response = client
            .post(format!("http://{}/api/register", upper_uri).as_str())
            .json(&register)
            .send()
            .await?;
        if response.status().is_success() {
            trace!("Node has been registered to parent or market node");
            Ok(())
        } else {
            Err(Error::RequestStatus(response.status()))
        }
    }
}
