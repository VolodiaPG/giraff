use std::net::IpAddr;
use std::sync::Arc;

use async_trait::async_trait;

use model::dto::node::NodeDescription;
use model::view::node::RegisterNode;
use model::{FogNodeHTTPPort, FogNodeRPCPort};

use crate::{NodeQuery, NodeSituation};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("This node has no parent (probably it is the market/root node)")]
    ParentDoesntExist,
    #[error(transparent)]
    NodeQuery(#[from] crate::repository::node_query::Error),
    #[error(
        "Trying to register/pass a register message for a market node,but it \
         should not happen since the market node is always on top of the \
         tree network."
    )]
    CannotRegisterMarketOnRegularNode,
}

/// Service to manage the behaviour of the routing
#[async_trait]
pub trait NodeLife: Send + Sync {
    /// Register locally the child node, but also send the packet towards the
    /// market to register it there, also.
    async fn register_child_node(
        &self,
        register: RegisterNode,
    ) -> Result<(), Error>;
    /// Initialize the negotiating process to get connected to the parent node
    async fn init_registration(
        &self,
        my_ip: IpAddr,
        my_port_http: FogNodeHTTPPort,
        my_port_rpc: FogNodeRPCPort,
    ) -> Result<(), Error>;
}

#[derive(Debug)]
pub struct NodeLifeImpl {
    node_situation: Arc<dyn NodeSituation>,
    node_query:     Arc<dyn NodeQuery>,
}

impl NodeLifeImpl {
    pub fn new(
        node_situation: Arc<dyn NodeSituation>,
        node_query: Arc<dyn NodeQuery>,
    ) -> Self {
        Self { node_situation, node_query }
    }
}

#[async_trait]
impl NodeLife for NodeLifeImpl {
    async fn register_child_node(
        &self,
        register: RegisterNode,
    ) -> Result<(), Error> {
        trace!("Registering child node");
        match &register {
            RegisterNode::Node {
                node_id,
                parent,
                ip,
                port_http,
                port_rpc,
                ..
            } => {
                if &self.node_situation.get_my_id() == parent {
                    self.node_situation.register(
                        node_id.clone(),
                        NodeDescription {
                            ip:        *ip,
                            port_http: port_http.clone(),
                            port_rpc:  port_rpc.clone(),
                        },
                    );
                }
            }
            RegisterNode::MarketNode { .. } => {
                return Err(Error::CannotRegisterMarketOnRegularNode)
            }
        }

        self.node_query.register_to_parent(register).await?;
        Ok(())
    }

    async fn init_registration(
        &self,
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_rpc: FogNodeRPCPort,
    ) -> Result<(), Error> {
        trace!("Init registration");
        let register = if self.node_situation.is_market() {
            RegisterNode::MarketNode {
                node_id: self.node_situation.get_my_id(),
                ip,
                port_http,
                port_rpc,
                tags: self.node_situation.get_my_tags(),
            }
        } else {
            RegisterNode::Node {
                ip,
                port_http,
                port_rpc,
                node_id: self.node_situation.get_my_id(),
                parent: self
                    .node_situation
                    .get_parent_id()
                    .ok_or(Error::ParentDoesntExist)?,
                tags: self.node_situation.get_my_tags(),
            }
        };
        self.node_query.register_to_parent(register).await?;
        Ok(())
    }
}
