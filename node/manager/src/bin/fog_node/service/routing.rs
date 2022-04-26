use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;

use manager::model::{domain::routing::RoutingStack, BidId, NodeId};
use manager::openfaas::DefaultApi;

use crate::repository::routing::Routing as RoutingRepository;
use crate::routing::RoutingTable;
use crate::service::faas::FaaSBackend;
use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Repository(#[from] crate::repository::routing::Error),
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
pub trait Router: Send + Sync {
    /// Register a new route, from a [RoutingStack], making the follow up requests left to do in the chain
    async fn register_route(&self, stack: RoutingStack) -> Result<(), Error>;
    /// Forward payloads to a neighbour node
    async fn forward(
        self: &Self,
        to: &BidId,
        resource_url: String,
        payload: Vec<u8>,
    ) -> Result<(), Error>;
}

pub struct RouterImpl {
    routing_table: Arc<RwLock<RoutingTable>>,
    node_situation: Arc<NodeSituation>,
    routing: Arc<dyn RoutingRepository>,
    faas: Arc<dyn FaaSBackend>,
    faas_api: Arc<dyn DefaultApi>,
}

impl RouterImpl {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        routing: Arc<dyn RoutingRepository>,
        faas: Arc<dyn FaaSBackend>,
        faas_api: Arc<dyn DefaultApi>,
    ) -> Self {
        Self {
            routing_table: Arc::new(RwLock::new(RoutingTable::default())),
            node_situation,
            routing,
            faas,
            faas_api,
        }
    }

    async fn forward_to_node(
        &self,
        to: &NodeId,
        resource_url: String,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        let next = self
            .node_situation
            .get(to)
            .ok_or(Error::NextNodeDoesntExist(to.to_owned()))?;
        let url = format!("http://{}/routing/{}", &next.uri, &resource_url);
        self.routing.forward(url, payload).await.map_err(Into::into)
    }
}

#[async_trait]
impl Router for RouterImpl {
    async fn register_route(&self, mut stack: RoutingStack) -> Result<(), Error> {
        // At least 1 more stop
        if stack.route_to_first.len() > 1
            && stack
                .route_to_first
                .starts_with(&[self.node_situation.my_id.to_owned()])
        {
            stack.route_to_first.pop();
            let next = stack
                .route_to_first
                .get(0)
                .ok_or(Error::MalformedRoutingStack)?;
            self.forward_to_node(
                next,
                "/api/routing/register".to_string(),
                serde_json::to_vec(&stack).map_err(Error::from)?,
            )
            .await?;
            return Ok(());
        }

        // No more stop, we need to start registering routes
        if stack.route_to_first.is_empty() {
            if stack.route_to_first.len() == 1
                && stack
                    .route_to_first
                    .starts_with(&[self.node_situation.my_id.to_owned()])
            {
                stack.route_to_first.pop();
                let mid = stack
                    .routes
                    .iter()
                    .position(|node| node == &self.node_situation.my_id)
                    .ok_or(Error::MalformedRoutingStack)?;

                // left: from, right: to
                let (left, right) = stack.routes.split_at(mid);

                self.routing_table
                    .blocking_write()
                    .update_route(stack.function.to_owned(), right[0].to_owned())
                    .await;

                if left.len() > 0 {
                    let next = &left[0];
                    self.forward_to_node(
                        next,
                        "/api/routing/register".to_string(),
                        serde_json::to_vec(&RoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: left.to_vec(),
                        })
                        .map_err(Error::from)?,
                    )
                    .await?;
                }

                if right.len() > 1 {
                    let next = &right[1];
                    self.forward_to_node(
                        next,
                        "/api/routing/register".to_string(),
                        serde_json::to_vec(&RoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: right.to_vec(),
                        })
                        .map_err(Error::from)?,
                    )
                    .await?;
                }

                return Ok(());
            }

            if stack.routes.len() > 1 {
                let next;
                if stack
                    .routes
                    .starts_with(&[self.node_situation.my_id.to_owned()])
                {
                    stack.routes.remove(0);
                    next = Some(&stack.routes[0]);
                } else if stack
                    .routes
                    .ends_with(&[self.node_situation.my_id.to_owned()])
                {
                    stack.routes.remove(stack.routes.len() - 1);
                    next = Some(&stack.routes[stack.routes.len() - 1]);
                } else {
                    next = None;
                }

                if let Some(next) = next {
                    self.forward_to_node(
                        next,
                        "/api/routing/register".to_string(),
                        serde_json::to_vec(&RoutingStack {
                            function: stack.function.to_owned(),
                            route_to_first: vec![],
                            routes: stack.routes.to_vec(),
                        })
                        .map_err(Error::from)?,
                    )
                    .await?;
                    return Ok(());
                }
            }

            if stack.routes.len() == 1 {
                let last_node = stack.routes.pop().unwrap();
                return if last_node == self.node_situation.my_id {
                    trace!("Routing table is complete, I am the arrival point");
                    Ok(())
                } else {
                    trace!("Routing table is complete, I am the departure point");
                    self.routing_table
                        .blocking_write()
                        .update_route(stack.function, last_node)
                        .await;
                    Ok(())
                };
            }
        }
        Err(Error::MalformedRoutingStack)
    }

    async fn forward(
        &self,
        to: &BidId,
        resource_url: String,
        payload: Vec<u8>,
    ) -> Result<(), Error> {
        let node_to = self.routing_table.read().await.get_node(to).cloned();

        match node_to {
            Some(node_to) => self.forward_to_node(&node_to, resource_url, payload).await,
            None => {
                let record = self
                    .faas
                    .get_provisioned_function(to)
                    .await
                    .ok_or(Error::UnknownBidId(to.to_owned()))?;
                self.faas_api
                    .async_function_name_post(&*record.function_name, payload)
                    .await
                    .map_err(Error::from)
            }
        }
    }
}
