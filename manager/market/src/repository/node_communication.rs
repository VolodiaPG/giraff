use crate::service::fog_node_network::FogNodeNetwork;
use anyhow::{bail, Context, Result};
use model::domain::sla::Sla;
use model::dto::node::NodeRecord;
use model::view::auction::{AccumulatedLatency, BidProposals, BidRequest};
use model::{NodeId, SlaId};
use std::fmt::Debug;
use std::sync::Arc;
use tracing::instrument;

type HttpClient = reqwest_middleware::ClientWithMiddleware;

#[derive(Debug)]
pub struct NodeCommunication {
    network: Arc<FogNodeNetwork>,
    client:  Arc<HttpClient>,
}

impl NodeCommunication {
    pub fn new(network: Arc<FogNodeNetwork>, client: Arc<HttpClient>) -> Self {
        Self { network, client }
    }

    /// Sends to <ip:port from nodeId>/api/<route>
    async fn send(
        &self,
        to: &NodeId,
        route: &str,
    ) -> Result<reqwest_middleware::RequestBuilder> {
        let Some(NodeRecord { ip, port_http, .. }) =
            self.network.get_node(to).await
        else {
            bail!("Failed to find a record correspod to the node {}", to);
        };

        Ok(self.client.post(format!("http://{ip}:{port_http}/api/{route}")))
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn request_bids_from_node(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals> {
        let response = self
            .send(&to, "bid")
            .await
            .with_context(|| {
                format!("Failed to obtained the url to contact {}", to)
            })?
            .json(&BidRequest {
                sla,
                node_origin: to.clone(),
                accumulated_latency: AccumulatedLatency::default(),
                #[cfg(feature = "mincpurandom")]
                nb_propositions_required: 2,
            })
            .send()
            .await
            .with_context(|| format!("Failed to send the sla to {}", to))?;

        helper::reqwest_helper::deserialize_response(response)
            .await
            .with_context(|| {
                format!(
                    "Failed to deserialize the response when trying to get \
                     bids from node {} in response of sla {:?}",
                    to, sla
                )
            })
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn take_offer(&self, to: NodeId, id: &SlaId) -> Result<()> {
        let response = self
            .send(&to, &format!("accept/{}", id))
            .await
            .with_context(|| {
                format!("Failed to obtained the url to contact {}", to)
            })?
            .send()
            .await
            .with_context(|| {
                format!("Failed to send an offering to {}", to)
            })?;

        helper::reqwest_helper::deserialize_response(response)
            .await
            .with_context(|| {
                format!(
                    "Failed to deserialize the response when trying to take \
                     the offer of node {}",
                    to
                )
            })?;
        Ok(())
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn provision_function(
        &self,
        to: NodeId,
        id: &SlaId,
    ) -> Result<()> {
        let response = self
            .send(&to, &format!("provision/{}", id))
            .await
            .with_context(|| {
                format!("Failed to obtained the url to contact {}", to)
            })?
            .send()
            .await
            .with_context(|| {
                format!("Failed to send an offering to {}", to)
            })?;

        helper::reqwest_helper::deserialize_response(response)
            .await
            .with_context(|| {
                format!(
                    "Failed to deserialize the response when trying to take \
                     the offer of node {}",
                    to
                )
            })?;
        Ok(())
    }
}
