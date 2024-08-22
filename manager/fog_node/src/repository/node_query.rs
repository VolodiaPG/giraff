use crate::NodeSituation;
use anyhow::{Context, Result};
use backoff::ExponentialBackoffBuilder;
use model::dto::node::NodeDescription;
use model::view::auction::{BidProposals, BidRequest};
use model::view::node::RegisterNode;
use model::NodeId;
use reqwest::Response;
use serde::Serialize;
use std::fmt::Debug;
use std::sync::Arc;
use tracing::{instrument, trace};

type HttpClient = reqwest_middleware::ClientWithMiddleware;

#[derive(Debug)]
pub struct NodeQuery {
    node_situation: Arc<NodeSituation>,
    client:         Arc<HttpClient>,
}

impl NodeQuery {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        client: Arc<HttpClient>,
    ) -> Self {
        Self { node_situation, client }
    }

    async fn post<T: Serialize>(
        &self,
        url: &str,
        data: &T,
    ) -> Result<Response> {
        let response = self.client.post(url).json(data).send().await?;
        let response = response
            .error_for_status()
            .with_context(|| format!("Request to {} failed", url))?;
        Ok(response)
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn register_to_parent(
        &self,
        register: RegisterNode,
    ) -> Result<()> {
        // Ignore the type of port since it doesn't matter-explicitely-here
        let url = if self.node_situation.is_market() {
            let (addr, port) =
                self.node_situation.get_market_node_address().context(
                    "Failed to retrieve the URL of the parent (market) node",
                )?;
            format!("http://{addr}:{port}/api/register")
        } else {
            let (addr, port) =
                self.node_situation.get_parent_node_address().context(
                    "Failed to retrieve the URL of the parent (other node) \
                     node",
                )?;
            format!("http://{addr}:{port}/api/register")
        };

        trace!("Registering to {}", url);
        let backoff = ExponentialBackoffBuilder::default().build();
        backoff::future::retry(backoff, || async {
            self.post(url.as_str(), &register)
                .await
                .with_context(|| format!("Failed to register to {}", url))?;
            Ok(())
        })
        .await?;

        Ok(())
    }

    #[instrument(level = "trace", skip(self, request))]
    pub async fn request_neighbor_bid(
        &self,
        request: &BidRequest<'_>,
        id: NodeId,
    ) -> Result<BidProposals> {
        let NodeDescription { ip, port_http, .. } = self
            .node_situation
            .get_fog_node_neighbor(&id)
            .with_context(|| {
                format!(
                    "Failed to request bid from neighbor fog node with id: {}",
                    id
                )
            })?;

        let url = format!("http://{ip}:{port_http}/api/bid");
        let response = self
            .post(url.as_str(), request)
            .await
            .with_context(|| format!("POST request to {} failed", url))?;

        helper::reqwest_helper::deserialize_response(response).await
    }
}
