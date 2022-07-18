use std::fmt::Debug;
use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;
use uom::si::f64::Time;
use uom::si::time::second;

use manager::model::domain::routing::Packet;
use manager::model::domain::sla::Sla;
use manager::model::dto::node::NodeRecord;
use manager::model::view::auction::{BidProposal, BidProposals, BidRequest};
use manager::model::view::routing::Route;
use manager::model::NodeId;

use crate::repository::fog_node::FogNode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("The stack route is empty.")]
    EmptyRoutingStack,
    #[error("Routing failed with status code {0}: {1:?}")]
    ErrorStatus(reqwest::StatusCode, Option<String>),
    #[error("Packets can only be sent to fog nodes. (Wrong packet type)")]
    WrongPacketType,
    #[error("Cannot found the fog node with the id {0}.")]
    NodeIdNotFound(NodeId),
    #[error(
        "The fog node with the id {0} doesn't have a valid ip address and/or \
         port."
    )]
    NodeIpNotFound(NodeId),
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

    async fn establish_route(
        &self,
        starting_from: NodeId,
        route: Route,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NodeCommunicationThroughRoutingImpl {
    network: Arc<dyn FogNode>,
}

impl NodeCommunicationThroughRoutingImpl {
    pub fn new(network: Arc<dyn FogNode>) -> Self { Self { network } }

    async fn get_address_of_first_node(
        &self,
        route_stack: &[NodeId],
    ) -> Result<(IpAddr, u16), Error> {
        let node = route_stack.first().ok_or(Error::EmptyRoutingStack)?;
        let NodeRecord { ip, port, .. } = self
            .network
            .get(route_stack.last().ok_or(Error::EmptyRoutingStack)?)
            .await
            .ok_or_else(|| Error::NodeIdNotFound(node.clone()))?
            .data;

        let ip = ip.ok_or_else(|| Error::NodeIpNotFound(node.clone()))?;
        let port = port.ok_or_else(|| Error::NodeIpNotFound(node.clone()))?;
        Ok((ip, port))
    }

    async fn call_routing(&self, packet: Packet<'_>) -> Result<Bytes, Error> {
        let (ip, port) = match &packet {
            Packet::FogNode { route_to_stack, .. } => {
                self.get_address_of_first_node(route_to_stack).await?
            }
            _ => return Err(Error::WrongPacketType),
        };
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/api/routing", ip, port);
        trace!("Posting to {}", &url);
        let response = client.post(&url).json(&packet).send().await?;

        if response.status().is_success() {
            Ok(response.bytes().await?)
        } else {
            Err(Error::ErrorStatus(
                response.status(),
                response.text().await.ok(),
            ))
        }
    }
}

#[async_trait]
impl NodeCommunication for NodeCommunicationThroughRoutingImpl {
    async fn request_bids_from_node(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals, Error> {
        let data = Packet::FogNode {
            resource_uri:   "bid".to_string(),
            data:           &serde_json::value::to_raw_value(&BidRequest {
                sla,
                node_origin: to.clone(),
                accumulated_latency: Time::new::<second>(0.0),
            })?,
            route_to_stack: self.network.get_route_to_node(to).await,
        };

        Ok(serde_json::from_slice(&self.call_routing(data).await?)?)
    }

    async fn take_offer(
        &self,
        to: NodeId,
        bid: &BidProposal,
    ) -> Result<(), Error> {
        let data = Packet::FogNode {
            route_to_stack: self.network.get_route_to_node(to).await,
            resource_uri:   format!("bid/{}", bid.id),
            data:           &serde_json::value::to_raw_value(&())?,
        };

        self.call_routing(data).await?;
        Ok(())
    }

    async fn establish_route(
        &self,
        starting_from: NodeId,
        route: Route,
    ) -> Result<(), Error> {
        let data = Packet::FogNode {
            route_to_stack: self
                .network
                .get_route_to_node(starting_from)
                .await,
            resource_uri:   "register_route".to_string(),
            data:           &serde_json::value::to_raw_value(&route)?,
        };

        self.call_routing(data).await?;
        Ok(())
    }
}
