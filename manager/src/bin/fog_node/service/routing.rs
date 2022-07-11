use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use bytes::Bytes;

use manager::model::domain::routing::Packet;
use manager::model::dto::routing::Direction;
use manager::model::{domain::routing::FunctionRoutingStack, BidId, NodeId};
use manager::openfaas::DefaultApi;

use crate::repository::faas_routing_table::FaaSRoutingTable;
use crate::repository::routing::Routing as RoutingRepository;
use crate::service::faas::FaaSBackend;
use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Routing(#[from] crate::repository::routing::Error),
    #[error("The next node doesn't exist: {0}")]
    NextNodeDoesntExist(NodeId),
    #[error("The routing stack was not correct to be utilized")]
    MalformedRoutingStack,
    #[error("The bid id / function id is not known: {0}")]
    UnknownBidId(BidId),
    #[error("Failed to serialize: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error(transparent)]
    OpenFaas(#[from] manager::openfaas::Error<String>),
}

/// Service to manage the behaviour of the routing
#[async_trait]
pub trait Router: Debug + Send + Sync {
    /// Register a new route, from a [RoutingStack], making the follow up requests left to do in the chain
    async fn register_function_route(&self, stack: FunctionRoutingStack) -> Result<(), Error>;
    /// Forward payloads to a neighbour node
    async fn forward(&self, packet: &Packet) -> Result<Bytes, Error>;
}

#[derive(Debug)]
pub struct RouterImpl<R>
where
    R: RoutingRepository,
{
    faas_routing_table: Arc<dyn FaaSRoutingTable>,
    node_situation: Arc<dyn NodeSituation>,
    routing: Arc<R>,
    faas: Arc<dyn FaaSBackend>,
    faas_api: Arc<dyn DefaultApi>,
}

impl<R> RouterImpl<R>
where
    R: RoutingRepository,
{
    pub fn new(
        faas_routing_table: Arc<dyn FaaSRoutingTable>,
        node_situation: Arc<dyn NodeSituation>,
        routing: Arc<R>,
        faas: Arc<dyn FaaSBackend>,
        faas_api: Arc<dyn DefaultApi>,
    ) -> Self {
        Self {
            faas_routing_table,
            node_situation,
            routing,
            faas,
            faas_api,
        }
    }

    async fn forward_register_to_node(
        &self,
        to: &NodeId,
        stack: FunctionRoutingStack,
    ) -> Result<Bytes, Error> {
        let next = self
            .node_situation
            .get_fog_node_neighbor(to)
            .await
            .ok_or_else(|| Error::NextNodeDoesntExist(to.to_owned()))?;
        self.routing
            .forward_to_url(&next.ip, &next.port, "register", &stack)
            .await
            .map_err(Error::from)
    }
}

#[async_trait]
impl<R> Router for RouterImpl<R>
where
    R: RoutingRepository,
{
    async fn register_function_route(&self, mut stack: FunctionRoutingStack) -> Result<(), Error> {
        // At least 1 more stop
        if stack.route_to_first.len() > 1
            && stack
                .route_to_first
                .starts_with(&[self.node_situation.get_my_id().await])
        {
            stack.route_to_first.pop();
            let next = stack
                .route_to_first
                .last()
                .ok_or(Error::MalformedRoutingStack)?
                .to_owned();
            self.forward_register_to_node(&next, stack).await?;
            return Ok(());
        }

        // No more stop, we need to start registering routes
        if stack.route_to_first.is_empty() {
            if stack.route_to_first.len() == 1
                && stack
                    .route_to_first
                    .starts_with(&[self.node_situation.get_my_id().await])
            {
                let my_id = self.node_situation.get_my_id().await;
                stack.route_to_first.pop();
                let mid = stack
                    .routes
                    .iter()
                    .position(|node| node == &my_id)
                    .ok_or(Error::MalformedRoutingStack)?;

                // left: from, right: to
                let (left, right) = stack.routes.split_at(mid);

                self.faas_routing_table
                    .update(
                        stack.function.to_owned(),
                        Direction::NextNode(right[0].to_owned()),
                    )
                    .await;

                if !left.is_empty() {
                    let next = &left[0];
                    self.forward_register_to_node(
                        next,
                        FunctionRoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: left.to_vec(),
                        },
                    )
                    .await?;
                }

                if right.len() > 1 {
                    let next = &right[1];
                    self.forward_register_to_node(
                        next,
                        FunctionRoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: right.to_vec(),
                        },
                    )
                    .await?;
                }

                return Ok(());
            }

            if stack.routes.len() > 1 {
                let next;
                if stack
                    .routes
                    .starts_with(&[self.node_situation.get_my_id().await])
                {
                    stack.routes.remove(0);
                    next = Some(&stack.routes[0]);
                } else if stack
                    .routes
                    .ends_with(&[self.node_situation.get_my_id().await])
                {
                    stack.routes.remove(stack.routes.len() - 1);
                    next = Some(&stack.routes[stack.routes.len() - 1]);
                } else {
                    next = None;
                }

                if let Some(next) = next {
                    self.forward_register_to_node(
                        next,
                        FunctionRoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: stack.routes.to_vec(),
                        },
                    )
                    .await?;
                    return Ok(());
                }
            }

            if stack.routes.len() == 1 {
                let last_node = stack.routes.pop().unwrap();
                return if last_node == self.node_situation.get_my_id().await {
                    trace!("Routing table is complete, I am the arrival point");
                    self.faas_routing_table
                        .update(stack.function, Direction::CurrentNode)
                        .await;
                    Ok(())
                } else {
                    trace!("Routing table is complete, I am the departure point");
                    self.faas_routing_table
                        .update(stack.function, Direction::NextNode(last_node))
                        .await;
                    Ok(())
                };
            }
        }
        Err(Error::MalformedRoutingStack)
    }

    async fn forward(&self, packet: &Packet) -> Result<Bytes, Error> {
        match packet {
            Packet::FaaSFunction { to, data: payload } => {
                let node_to = self
                    .faas_routing_table
                    .get(to)
                    .await
                    .ok_or_else(|| Error::UnknownBidId(to.to_owned()))?;

                match node_to {
                    // TODO: optimization: is it possible to send the packet directly to the node? w/o redoing the same structure, what impact?
                    Direction::NextNode(next) => {
                        let next = self
                            .node_situation
                            .get_fog_node_neighbor(&next)
                            .await
                            .ok_or_else(|| Error::NextNodeDoesntExist(next.to_owned()))?;
                        Ok(self
                            .routing
                            .forward_to_routing(
                                &next.ip,
                                &next.port,
                                &Packet::FaaSFunction {
                                    to: to.to_owned(),
                                    data: payload,
                                },
                            )
                            .await?)
                    }
                    Direction::CurrentNode => {
                        let record = self
                            .faas
                            .get_provisioned_function(to)
                            .await
                            .ok_or_else(|| Error::UnknownBidId(to.to_owned()))?;
                        self.faas_api
                            .async_function_name_post(
                                &*record.function_name,
                                serde_json::to_string(payload)?,
                            )
                            .await
                            .map_err(Error::from)?;
                        // TODO check if that doesn't cause any harm
                        Ok(Bytes::new())
                    }
                }
            }
            Packet::FogNode {
                route_to_stack: route_to,
                resource_uri,
                data,
            } => {
                let mut route_to = route_to.clone();
                let current_node = route_to.pop().ok_or(Error::MalformedRoutingStack)?;

                if current_node != self.node_situation.get_my_id().await {
                    return Err(Error::MalformedRoutingStack);
                }

                if route_to.is_empty() {
                    let my_ip = self.node_situation.get_my_public_ip().await;
                    let my_port = self.node_situation.get_my_public_port().await;

                    Ok(self
                        .routing
                        .forward_to_url(&my_ip, &my_port, resource_uri, data)
                        .await?)
                } else {
                    let next = route_to.last().unwrap();
                    let next = self
                        .node_situation
                        .get_fog_node_neighbor(next)
                        .await
                        .ok_or_else(|| Error::NextNodeDoesntExist(next.to_owned()))?;
                    Ok(self
                        .routing
                        .forward_to_routing(
                            &next.ip,
                            &next.port,
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
                if self.node_situation.is_market().await {
                    trace!("Transmitting market packet to market: {:?}", packet);
                    let (ip, port) = self.node_situation.get_market_node_address().await.unwrap();
                    Ok(self
                        .routing
                        .forward_to_url(&ip, &port, resource_uri, data)
                        .await?)
                } else {
                    trace!("Transmitting market packet to other node: {:?}", packet);
                    let (ip, port) = self.node_situation.get_parent_node_address().await.unwrap();
                    Ok(self.routing.forward_to_routing(&ip, &port, packet).await?)
                }
            }
        }
    }
}
