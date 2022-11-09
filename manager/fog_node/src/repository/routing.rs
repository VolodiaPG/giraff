use std::fmt::Debug;
use std::net::IpAddr;

use async_trait::async_trait;
use bytes::Bytes;
use reqwest::StatusCode;
use serde::Serialize;

use model::domain::routing::Packet;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error forwarding the payload: {0}")]
    Forwarding(#[from] reqwest::Error),
    #[error("Next node {0} answered with code {1}: {2}")]
    ForwardingResponse(String, StatusCode, String),
    #[error(transparent)]
    Serialization(#[from] serde_json::Error),
}

/// Behaviour of the routing
#[async_trait]
pub trait Routing: Debug + Sync + Send {
    /// Forward to the url to be handled by the routing service of the node
    async fn forward_to_routing(
        &self,
        ip: &IpAddr,
        port: &u16,
        packet: &Packet,
    ) -> Result<Bytes, Error>;

    /// Forward to the url to be handled by arbitrary route
    async fn forward_to_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &u16,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Bytes, Error>
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
    ) -> Result<Bytes, Error>
    where
        T: Serialize + Send + Sync,
    {
        let client = reqwest::Client::new();
        let res = client.post(full_url).json(data).send().await?;

        if res.status().is_success() {
            Ok(res.bytes().await?)
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
    async fn forward_to_routing(
        &self,
        ip: &IpAddr,
        port: &u16,
        packet: &Packet,
    ) -> Result<Bytes, Error> {
        let url = format!("http://{}:{}/api/routing", ip, port);
        trace!("Posting to routing on: {}...", &url);
        let ret = self.forward_to(&packet, &url).await;
        trace!("Posted to routing on: {}", &url);
        ret
    }

    async fn forward_to_url<'a, 'b, T>(
        &self,
        node_ip: &IpAddr,
        node_port: &u16,
        resource_uri: &'b str,
        data: &'a T,
    ) -> Result<Bytes, Error>
    where
        T: Serialize + Send + Sync,
    {
        let url =
            format!("http://{}:{}/api/{}", node_ip, node_port, resource_uri);
        trace!("Posting (forward) to {}", &url);
        self.forward_to(data, &url).await
    }
}
