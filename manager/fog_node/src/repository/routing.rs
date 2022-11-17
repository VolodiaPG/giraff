use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;

use drpc::client::Client;
use drpc::codec::{BinCodec, JsonCodec};
use model::{FogNodeHTTPPort, FogNodeRPCPort, MarketHTTPPort};
use reqwest::StatusCode;
use serde::Serialize;

use model::domain::routing::Packet;
use serde_json::value::RawValue;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error forwarding the payload: {0}")]
    Forwarding(#[from] reqwest::Error),
    #[error("Next node {0} answered with code {1}: {2}")]
    ForwardingResponse(String, StatusCode, String),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    RPCForwarding(#[from] drpc::Error),
}

/// Behaviour of the routing
#[async_trait]
pub trait Routing: Debug + Sync + Send {
    /// Forward to the url to be handled by the routing service of the node
    async fn forward_to_routing(
        &self,
        ip: &IpAddr,
        port: &FogNodeRPCPort,
        packet: &Packet,
    ) -> Result<Box<RawValue>, Error>;

    /// Forward to the url to be handled by arbitrary route, on another node
    async fn forward_to_fog_node_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &FogNodeHTTPPort,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Box<RawValue>, Error>
    where
        T: Serialize + Send + Sync;

    /// Forward to the url to be handled by arbitrary route, on the marke node
    async fn forward_to_market_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &MarketHTTPPort,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Box<RawValue>, Error>
    where
        T: Serialize + Send + Sync;
}

#[derive(Debug, Default)]
pub struct RoutingImpl;

impl RoutingImpl {
    async fn forward_to<'a, T>(
        &self,
        data: &'a T,
        full_url: &'a str,
    ) -> Result<Box<RawValue>, Error>
    where
        T: Serialize + Send + Sync,
    {
        let client = reqwest::Client::new();
        let res = client.post(full_url).json(data).send().await?;

        if res.status().is_success() {
            res.json().await.map_err(Error::from)
        } else {
            Err(Error::ForwardingResponse(
                full_url.to_string(),
                res.status(),
                res.text().await.unwrap(),
            ))
        }
    }
}

#[async_trait]
impl Routing for RoutingImpl {
    #[instrument(level = "trace", skip(self, packet))]
    async fn forward_to_routing(
        &self,
        ip: &IpAddr,
        port: &FogNodeRPCPort,
        packet: &Packet,
    ) -> Result<Box<RawValue>, Error> {
        trace!("RPC-ing to routing on {}:{}...", ip, port);
        let c = Client::<JsonCodec>::dial(&format!("{}:{}", ip, port))
            .await
            .unwrap();
        c.call("routing", packet).await.map_err(Error::from)
    }

    #[instrument(level = "trace", skip(self, data))]
    async fn forward_to_fog_node_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &FogNodeHTTPPort,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Box<RawValue>, Error>
    where
        T: Serialize + Send + Sync,
    {
        let url =
            format!("http://{}:{}/api/{}", node_ip, node_port, resource_uri);
        trace!("Posting (forward) to {}", &url);
        self.forward_to(data, &url).await
    }

    #[instrument(level = "trace", skip(self, data))]
    async fn forward_to_market_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &MarketHTTPPort,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Box<RawValue>, Error>
    where
        T: Serialize + Send + Sync,
    {
        let url =
            format!("http://{}:{}/api/{}", node_ip, node_port, resource_uri);
        trace!("Posting (forward) to {}", &url);
        self.forward_to(data, &url).await
    }
}
