use crate::{NodeQuery, NodeSituation};
use anyhow::{bail, Context, Result};
use model::dto::node::NodeDescription;
use model::view::node::RegisterNode;
use model::{FogNodeFaaSPortExternal, FogNodeHTTPPort};
use std::net::IpAddr;
use std::sync::Arc;

#[derive(Debug)]
pub struct NodeLife {
    node_situation: Arc<NodeSituation>,
    node_query:     Arc<NodeQuery>,
}

impl NodeLife {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        node_query: Arc<NodeQuery>,
    ) -> Self {
        Self { node_situation, node_query }
    }

    pub async fn register_child_node(
        &self,
        register: RegisterNode,
    ) -> Result<()> {
        trace!("Registering child node");
        match &register {
            RegisterNode::Node { node_id, parent, ip, port_http, .. } => {
                if &self.node_situation.get_my_id() == parent {
                    self.node_situation.register(
                        node_id.clone(),
                        NodeDescription {
                            ip:        *ip,
                            port_http: port_http.clone(),
                        },
                    );
                }
            }
            RegisterNode::MarketNode { .. } => {
                bail!(
                    "Cannot register the market to anything, there is no \
                     parents to a market node"
                );
            }
        }

        self.node_query
            .register_to_parent(register)
            .await
            .context("Failed to register a child node to my parent")?;
        Ok(())
    }

    pub async fn init_registration(
        &self,
        ip: IpAddr,
        port_http: FogNodeHTTPPort,
        port_faas: FogNodeFaaSPortExternal,
    ) -> Result<()> {
        trace!("Init registration");
        let register = if self.node_situation.is_market() {
            RegisterNode::MarketNode {
                node_id: self.node_situation.get_my_id(),
                ip,
                port_http,
                port_faas,
                tags: self.node_situation.get_my_tags(),
            }
        } else {
            RegisterNode::Node {
                ip,
                port_http,
                port_faas,
                node_id: self.node_situation.get_my_id(),
                parent: self
                    .node_situation
                    .get_parent_id()
                    .context("Failed to get my parent's id")?,
                tags: self.node_situation.get_my_tags(),
            }
        };
        self.node_query.register_to_parent(register).await?;
        Ok(())
    }
}
