use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use reqwest::Response;
use serde::Serialize;

use model::dto::node::NodeDescription;
use model::view::auction::{BidProposals, BidRequest};
use model::view::node::RegisterNode;
use model::NodeId;

use crate::NodeSituation;

#[cfg(feature = "jaeger")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "jaeger"))]
type HttpClient = reqwest::Client;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "jaeger")]
    #[error(transparent)]
    ReqwestMiddleware(#[from] reqwest_middleware::Error),
    #[error("The request failed with error code: {0}")]
    RequestStatus(reqwest::StatusCode),
    #[error(
        "Failed to retrieve the URI to either the parent node or the market."
    )]
    NoURIToUpper,
    #[error("Failed to retrieve the address of the node: {0}")]
    NodeIdNotFound(NodeId),
}

#[async_trait]
pub trait NodeQuery: Debug + Sync + Send {
    /// Update the breadcrumb route to the [BidId] passing by the next
    /// [NodeId].
    async fn register_to_parent(
        &self,
        register: RegisterNode,
    ) -> Result<(), Error>;
    async fn request_neighbor_bid(
        &self,
        request: &BidRequest,
        node: NodeId,
    ) -> Result<BidProposals, Error>;
}

#[derive(Debug)]
pub struct NodeQueryRESTImpl {
    node_situation: Arc<dyn NodeSituation>,
    client:         Arc<HttpClient>,
}

impl NodeQueryRESTImpl {
    pub fn new(
        node_situation: Arc<dyn NodeSituation>,
        client: Arc<HttpClient>,
    ) -> Self {
        Self { node_situation, client }
    }

    async fn post<T: Serialize>(
        &self,
        url: &str,
        data: &T,
    ) -> Result<Response, Error> {
        let response = self.client.post(url).json(data).send().await?;
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
    #[instrument(level = "trace", skip(self))]
    async fn register_to_parent(
        &self,
        register: RegisterNode,
    ) -> Result<(), Error> {
        trace!("Registering to parent or market...");

        // Ignore the type of port since it doesn't matter-explicitely-here
        let url = if self.node_situation.is_market() {
            let (addr, port) = self
                .node_situation
                .get_market_node_address()
                .ok_or(Error::NoURIToUpper)?;
            format!("http://{}:{}/api/register", addr, port)
        } else {
            let (addr, port, _) = self
                .node_situation
                .get_parent_node_address()
                .ok_or(Error::NoURIToUpper)?;
            format!("http://{}:{}/api/register", addr, port)
        };

        trace!("Registering to {}", url);
        self.post(url.as_str(), &register).await?;

        // Both the market and node APIs offer the same endpoint.

        Ok(())
    }

    #[instrument(level = "trace", skip(self, request))]
    async fn request_neighbor_bid(
        &self,
        request: &BidRequest,
        id: NodeId,
    ) -> Result<BidProposals, Error> {
        let NodeDescription { ip, port_http, .. } = self
            .node_situation
            .get_fog_node_neighbor(&id)
            .ok_or_else(|| Error::NodeIdNotFound(id.clone()))?;

        Ok(self
            .post(
                format!("http://{}:{}/api/bid", ip, port_http).as_str(),
                request,
            )
            .await?
            .json()
            .await?)
    }
}
