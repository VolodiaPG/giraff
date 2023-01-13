use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;

use model::domain::routing::Packet;
use model::NodeId;
use serde_json::Value;

use crate::repository::routing::Routing as RoutingRepository;
use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Routing(#[from] crate::repository::routing::Error),
    #[error("The next node doesn't exist: {0}")]
    NextNodeDoesntExist(NodeId),
    #[error("The routing stack was malformed and could not be utilized")]
    MalformedRoutingStack,
    #[error("Failed to serialize: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    OpenFaas(#[from] openfaas::Error<String>),
}

/// Service to manage the behaviour of the routing
#[async_trait]
pub trait Router: Debug + Send + Sync {
    /// Forward payloads to a neighbour node
    async fn forward(&self, packet: Packet) -> Result<Option<Value>, Error>;
}

#[derive(Debug)]
pub struct RouterImpl<R>
where
    R: RoutingRepository,
{
    node_situation: Arc<dyn NodeSituation>,
    routing:        Arc<R>,
}

impl<R> RouterImpl<R>
where
    R: RoutingRepository,
{
    pub fn new(
        node_situation: Arc<dyn NodeSituation>,
        routing: Arc<R>,
    ) -> Self {
        Self { node_situation, routing }
    }
}

#[async_trait]
impl<R> Router for RouterImpl<R>
where
    R: RoutingRepository,
{
    #[instrument(level = "trace", skip(self, packet))]
    async fn forward(&self, packet: Packet) -> Result<Option<Value>, Error> {
        trace!("Forwarding packet...");
        match packet {
            Packet::FogNode {
                route_to_stack: route_to,
                resource_uri,
                data,
            } => {
                let mut route_to = route_to.clone();
                let current_node =
                    route_to.pop().ok_or(Error::MalformedRoutingStack)?;

                if current_node != self.node_situation.get_my_id() {
                    return Err(Error::MalformedRoutingStack);
                }

                if route_to.is_empty() {
                    let my_ip = self.node_situation.get_my_public_ip();
                    let my_port =
                        self.node_situation.get_my_public_port_http();

                    Ok(self
                        .routing
                        .forward_to_fog_node_url(
                            &my_ip,
                            &my_port,
                            &resource_uri,
                            &data,
                        )
                        .await?)
                } else {
                    let next = route_to.last().unwrap();
                    let next = self
                        .node_situation
                        .get_fog_node_neighbor(next)
                        .ok_or_else(|| {
                            Error::NextNodeDoesntExist(next.to_owned())
                        })?;
                    Ok(self
                        .routing
                        .forward_to_routing(
                            &next.ip,
                            &next.port_http,
                            &Packet::FogNode {
                                route_to_stack: route_to,
                                resource_uri: resource_uri.to_owned(),
                                data,
                            },
                        )
                        .await?)
                }
            }
            Packet::Market { resource_uri, data } => {
                if self.node_situation.is_market() {
                    trace!(
                        "Transmitting market packet to market on {:?}: {:?}",
                        &resource_uri,
                        &data
                    );
                    let (ip, port) =
                        self.node_situation.get_market_node_address().unwrap();
                    Ok(self
                        .routing
                        .forward_to_market_url(
                            &ip,
                            &port,
                            &resource_uri,
                            &data,
                        )
                        .await?)
                } else {
                    trace!(
                        "Transmitting market packet to other node on {:?}: \
                         {:?}",
                        &resource_uri,
                        &data
                    );
                    let (ip, port, _port) =
                        self.node_situation.get_parent_node_address().unwrap();
                    Ok(self
                        .routing
                        .forward_to_routing(
                            &ip,
                            &port,
                            &Packet::Market { resource_uri, data },
                        )
                        .await?)
                }
            }
        }
    }
}
