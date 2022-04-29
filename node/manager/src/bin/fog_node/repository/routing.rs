use std::fmt::Debug;

use async_trait::async_trait;
use reqwest::StatusCode;
use serde::Serialize;

use manager::model::domain::routing::Packet;

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
    async fn forward_to_routing(&self, uri: &String, packet: &Packet) -> Result<(), Error>;

    /// Foward to the url to be handled by aribitrary route
    async fn forward_to_url<'a, T>(
        &self,
        node_uri: &String,
        resource_uri: &String,
        data: &'a T,
    ) -> Result<(), Error>
    where
        T: Serialize + Send + Sync;
}

#[derive(Debug, Default)]
pub struct RoutingImpl;

#[async_trait]
impl Routing for RoutingImpl {
    async fn forward_to_routing(&self, uri: &String, packet: &Packet) -> Result<(), Error> {
        let url = format!("http://{}/api/routing", uri);
        trace!("Posting to routing on node {}", &uri);
        let client = reqwest::Client::new();
        client
            .post(url)
            .json(packet)
            .send()
            .await
            .map_err(Error::from)?;
        Ok(())
    }

    async fn forward_to_url<'a, T>(
        &self,
        node_uri: &String,
        resource_uri: &String,
        data: &'a T,
    ) -> Result<(), Error>
    where
        T: Serialize + Send + Sync,
    {
        let url = format!("http://{}/api/{}", node_uri, resource_uri);
        trace!("Posting (forward) to {}", &url);
        let client = reqwest::Client::new();
        let res = client
            .post(url.to_owned())
            .json(data)
            .send()
            .await
            .map_err(Error::from)?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Error::ForwardingResponse(
                url,
                res.status(),
                res.text().await.unwrap(),
            ))
        }
    }
}
