use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use uom::si::f64::Time;
use uom::si::time::second;

use model::domain::sla::Sla;
use model::dto::node::NodeRecord;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::NodeId;

use crate::service::fog_node_network::FogNodeNetwork;

#[cfg(feature = "jaeger")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "jaeger"))]
type HttpClient = reqwest::Client;

#[cfg(feature = "jaeger")]
type HttpRequestBuilder = reqwest_middleware::RequestBuilder;
#[cfg(not(feature = "jaeger"))]
type HttpRequestBuilder = reqwest::RequestBuilder;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Failed to retrieve the record of the node {0}")]
    RecordOfNodeNotFound(NodeId),
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[cfg(feature = "jaeger")]
    #[error(transparent)]
    ReqwestMiddlewareError(#[from] reqwest_middleware::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
}
#[async_trait]
pub trait NodeCommunication: Debug + Sync + Send {
    async fn request_bids_from_node(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals, Error>;

    async fn take_offer(
        &self,
        to: NodeId,
        bid: &BidProposal,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NodeCommunicationImpl {
    network: Arc<dyn FogNodeNetwork>,
    client:  Arc<HttpClient>,
}

impl NodeCommunicationImpl {
    pub fn new(
        network: Arc<dyn FogNodeNetwork>,
        client: Arc<HttpClient>,
    ) -> Self {
        Self { network, client }
    }

    /// Sends to <ip:port from nodeId>/api/<route>
    async fn send(
        &self,
        to: &NodeId,
        route: &str,
    ) -> Result<HttpRequestBuilder, Error> {
        let Some(NodeRecord {ip, port_http, ..}) = self.network.get_node(to).await else {
            return Err(Error::RecordOfNodeNotFound(to.clone()));
        };

        Ok(self.client.post(format!("http://{ip}:{port_http}/api/{route}")))
    }
}

#[async_trait]
impl NodeCommunication for NodeCommunicationImpl {
    async fn request_bids_from_node(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals, Error> {
        let resp: BidProposals = self
            .send(&to, "bid")
            .await?
            .json(&BidRequest {
                sla,
                node_origin: to.clone(),
                accumulated_latency: Time::new::<second>(0.0),
            })
            .send()
            .await?
            .json()
            .await?;

        Ok(resp)
    }

    async fn take_offer(
        &self,
        to: NodeId,
        bid: &BidProposal,
    ) -> Result<(), Error> {
        self.send(&to, &format!("bid/{}", bid.id))
            .await?
            .send()
            .await?
            .json()
            .await?;
        Ok(())
    }
}
