use async_trait::async_trait;
use bytes::Bytes;
use manager::model::domain::routing::Packet;
use reqwest::StatusCode;
use serde::Serialize;
use std::fmt::Debug;
use std::net::IpAddr;

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

#[async_trait]
impl Routing for RoutingImpl {
    async fn forward_to_routing(
        &self,
        ip: &IpAddr,
        port: &u16,
        packet: &Packet,
    ) -> Result<Bytes, Error> {
        let url = format!("http://{}:{}/api/routing", ip, port);
        trace!("Posting to routing on: {}", &url);
        let client = reqwest::Client::new();
        Ok(client.post(url).json(packet).send().await?.bytes().await?)
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
        let url = format!("http://{}:{}/api/{}", node_ip, node_port, resource_uri);
        trace!("Posting (forward) to {}", &url);
        let client = reqwest::Client::new();
        let res = client
            .post(url.to_owned())
            .json(data)
            .send()
            .await
            .map_err(Error::from)?;

        if res.status().is_success() {
            Ok(res.bytes().await?)
        } else {
            Err(Error::ForwardingResponse(
                url,
                res.status(),
                res.text().await.unwrap(),
            ))
        }
    }
}
