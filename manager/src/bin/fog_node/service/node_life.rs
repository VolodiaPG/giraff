use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;

use manager::model::domain::routing::Packet;
use manager::model::dto::node::NodeDescription;
use manager::model::view::node::RegisterNode;

use crate::{NodeQuery, NodeSituation, Router};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("A node tried to register here, but I am not her parent")]
    NotTheParent,
    #[error("This node has no parent (probably it is the market/root node)")]
    ParentDoesntExist,
    #[error(transparent)]
    NodeQuery(#[from] crate::repository::node_query::Error),
    #[error(transparent)]
    Routing(#[from] crate::service::routing::Error),
    #[error("Trying to register/pass a register message for a market node,but it should not happen since the market node is always on top of the tree network.")]
    CannotRegisterMarketOnRegularNode,
}

/// Service to manage the behaviour of the routing
#[async_trait]
pub trait NodeLife: Send + Sync {
    /// Register locally the child node, but also send the packet towards the market to register it there, also.
    async fn register_child_node(&self, register: RegisterNode) -> Result<(), Error>;
    /// Initialize the negotiating process to get connected to the parent node
    async fn init_registration(&self, my_ip: IpAddr, my_port: u16) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NodeLifeImpl {
    router: Arc<dyn Router>,
    node_situation: Arc<dyn NodeSituation>,
    node_query: Arc<dyn NodeQuery>,
}

impl NodeLifeImpl {
    pub fn new(
        router: Arc<dyn Router>,
        node_situation: Arc<dyn NodeSituation>,
        node_query: Arc<dyn NodeQuery>,
    ) -> Self {
        Self {
            router,
            node_situation,
            node_query,
        }
    }
}

#[async_trait]
impl NodeLife for NodeLifeImpl {
    async fn register_child_node(&self, register: RegisterNode) -> Result<(), Error> {
        trace!("Registering child node");
        match &register {
            RegisterNode::Node {
                node_id,
                parent,
                ip,
                port,
                ..
            } => {
                if &self.node_situation.get_my_id().await != parent {
                    return Err(Error::NotTheParent);
                }

                self.node_situation
                    .register(
                        node_id.clone(),
                        NodeDescription {
                            ip: *ip,
                            port: *port,
                        },
                    )
                    .await;
            }
            RegisterNode::MarketNode { .. } => {
                return Err(Error::CannotRegisterMarketOnRegularNode)
            }
        }

        self.router
            .forward(&Packet::Market {
                resource_uri: "register".to_string(),
                data: &*serde_json::value::to_raw_value(&register).unwrap(),
            })
            .await?;
        Ok(())
    }

    async fn init_registration(&self, ip: IpAddr, port: u16) -> Result<(), Error> {
        trace!("Init registration");
        let register = if self.node_situation.is_market().await {
            RegisterNode::MarketNode {
                node_id: self.node_situation.get_my_id().await,
                ip,
                port,
                tags: self.node_situation.get_my_tags().await.clone(),
            }
        } else {
            RegisterNode::Node {
                ip,
                port,
                node_id: self.node_situation.get_my_id().await,
                parent: self
                    .node_situation
                    .get_parent_id()
                    .await
                    .ok_or(Error::ParentDoesntExist)?,
                tags: self.node_situation.get_my_tags().await.clone(),
            }
        };
        self.node_query.register_to_parent(register).await?;
        Ok(())
    }
}
