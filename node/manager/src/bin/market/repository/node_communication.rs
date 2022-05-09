use async_trait::async_trait;
use bytes::Bytes;
use std::net::IpAddr;
use uom::si::f64::Time;
use uom::si::time::second;

use manager::model::domain::routing::Packet;
use manager::model::domain::sla::Sla;
use manager::model::view::auction::{BidProposal, BidProposals, BidRequest};
use manager::model::NodeId;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    #[error(transparent)]
    Serialize(#[from] serde_json::Error),
    #[error("The stack route is empty.")]
    EmptyRoutingStack,
    #[error("The status code errored out {0}.")]
    ErrorStatus(reqwest::StatusCode),
}
#[async_trait]
pub trait NodeCommunication: Sync + Send {
    async fn request_bids_from_node(
        &self,
        ip: IpAddr,
        port: u16,
        route_to_stack: Vec<NodeId>,
        sla: Sla,
    ) -> Result<BidProposals, Error>;

    async fn take_offer(
        &self,
        ip: IpAddr,
        port: u16,
        route_to_stack: Vec<NodeId>,
        bid: &BidProposal,
    ) -> Result<(), Error>;
}

pub struct NodeCommunicationThroughRoutingImpl {}

impl NodeCommunicationThroughRoutingImpl {
    pub fn new() -> Self {
        Self {}
    }

    async fn call_routing(
        &self,
        ip: IpAddr,
        port: u16,
        packet: Packet<'_>,
    ) -> Result<Bytes, Error> {
        let client = reqwest::Client::new();
        let url = format!("http://{}:{}/api/routing", ip, port);
        trace!("Posting to {}", &url);
        let response = client.post(&url).json(&packet).send().await?;

        if response.status().is_success() {
            Ok(response.bytes().await?)
        } else {
            Err(Error::ErrorStatus(response.status()))
        }
    }
}

#[async_trait]
impl NodeCommunication for NodeCommunicationThroughRoutingImpl {
    async fn request_bids_from_node(
        &self,
        ip: IpAddr,
        port: u16,
        route_to_stack: Vec<NodeId>,
        sla: Sla,
    ) -> Result<BidProposals, Error> {
        let data = Packet::FogNode {
            resource_uri: "bid".to_string(),
            data: &*serde_json::value::to_raw_value(&BidRequest {
                sla,
                node_origin: route_to_stack
                    .first()
                    .ok_or(Error::EmptyRoutingStack)?
                    .clone(),
                accumulated_latency: Time::new::<second>(0.0),
            })?,
            route_to_stack,
        };

        Ok(serde_json::from_slice(
            &self.call_routing(ip, port, data).await?,
        )?)
    }

    async fn take_offer(
        &self,
        ip: IpAddr,
        port: u16,
        route_to_stack: Vec<NodeId>,
        bid: &BidProposal,
    ) -> Result<(), Error> {
        let data = Packet::FogNode {
            route_to_stack,
            resource_uri: format!("/bid/{}", bid.id),
            data: &*serde_json::value::to_raw_value(&())?,
        };

        self.call_routing(ip, port, data).await?;
        Ok(())
    }
}
