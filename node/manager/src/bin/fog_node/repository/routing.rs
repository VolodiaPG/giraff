use async_trait::async_trait;
use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Error forwarding the payload: {0}")]
    Forwarding(#[from] reqwest::Error),
    #[error("Next node {0} answered with code {1}: {2}")]
    ForwardingResponse(String, StatusCode, String),
}

/// Behaviour of the routing
#[async_trait]
pub trait Routing: Sync + Send {
    /// Forward to the [to_url] (e.g., `http://localhost:3000/<resource>`), handles the transmission of the [payload]
    async fn forward(&self, to_url: String, payload: Vec<u8>) -> Result<(), Error>;
}

pub struct RoutingImpl;

#[async_trait]
impl Routing for RoutingImpl {
    async fn forward(&self, to_url: String, payload: Vec<u8>) -> Result<(), Error> {
        trace!("Forwarding to {}", to_url.to_owned());
        let client = reqwest::Client::new();
        let res = client
            .post(to_url.to_owned())
            .body(payload)
            .send()
            .await
            .map_err(Error::from)?;

        if res.status().is_success() {
            Ok(())
        } else {
            Err(Error::ForwardingResponse(
                to_url.to_owned(),
                res.status(),
                res.text().await.unwrap(),
            ))
        }
    }
}
