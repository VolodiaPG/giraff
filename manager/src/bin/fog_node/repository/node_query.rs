use std::{fmt::Debug, net::IpAddr, sync::Arc};

use async_trait::async_trait;
use reqwest::Response;
use serde::Serialize;

use manager::model::{
    dto::node::NodeDescription,
    view::{
        auction::{BidProposals, BidRequest},
        node::RegisterNode,
    },
    NodeId,
};

use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error("The request failed with error code: {0}")]
    RequestStatus(reqwest::StatusCode),
    #[error("Failed to retrieve the URI to either the parent node or the market.")]
    NoURIToUpper,
    #[error("Failed to retrieve the address of the node: {0}")]
    NodeIdNotFound(NodeId),
}

#[async_trait]
pub trait NodeQuery: Debug + Sync + Send {
    /// Update the breadcrumb route to the [BidId] passing by the next [NodeId].
    async fn register_to_parent(&self, register: RegisterNode) -> Result<(), Error>;
    async fn request_neighbor_bid(
        &self,
        request: BidRequest,
        node: NodeId,
    ) -> Result<BidProposals, Error>;
}

#[derive(Debug)]
pub struct NodeQueryRESTImpl {
    node_situation: Arc<dyn NodeSituation>,
}

impl NodeQueryRESTImpl {
    pub fn new(node_situation: Arc<dyn NodeSituation>) -> Self { Self { node_situation } }

    async fn post<T: Serialize>(
        &self,
        ip: &IpAddr,
        port: &u16,
        uri: &str,
        data: &T,
    ) -> Result<Response, Error> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!("http://{}:{}/api/{}", ip, port, uri).as_str())
            .json(data)
            .send()
            .await?;
        if response.status().is_success() {
            trace!("Node has been registered to parent or market node");
            Ok(response)
        } else {
            Err(Error::RequestStatus(response.status()))
        }
    }
}

#[async_trait]
impl NodeQuery for NodeQueryRESTImpl {
    async fn register_to_parent(&self, register: RegisterNode) -> Result<(), Error> {
        trace!("Registering to parent or market...");
        let upper_node_address = if self.node_situation.is_market().await {
            self.node_situation.get_market_node_address().await.ok_or(Error::NoURIToUpper)?
        } else {
            self.node_situation.get_parent_node_address().await.ok_or(Error::NoURIToUpper)?
        };

        // Both the market and node APIs offer the same endpoint.
        trace!("Registering to {}:{}", upper_node_address.0, upper_node_address.1);
        self.post(&upper_node_address.0, &upper_node_address.1, "register", &register).await?;
        Ok(())
    }

    async fn request_neighbor_bid(
        &self,
        request: BidRequest,
        id: NodeId,
    ) -> Result<BidProposals, Error> {
        let NodeDescription { ip, port, .. } = self
            .node_situation
            .get_fog_node_neighbor(&id)
            .await
            .ok_or_else(|| Error::NodeIdNotFound(id.clone()))?;
        let (ip, port) = (ip, port);

        Ok(self.post(&ip, &port, "bid", &request).await?.json().await?)
    }
}
