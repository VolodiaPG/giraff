use std::fmt::Debug;
use std::net::IpAddr;

use model::dto::node::NodeCategory::{MarketConnected, NodeConnected};
use model::dto::node::{NodeDescription, NodeSituationData};
use model::{
    FogNodeFaaSPortExternal, FogNodeHTTPPort, MarketHTTPPort, NodeId,
};
use uom::si::f64::{Information, Ratio};

pub trait NodeSituation: Debug + Sync + Send {
    fn register(&self, id: NodeId, description: NodeDescription);
    /// Get a node: children, parent
    fn get_fog_node_neighbor(&self, id: &NodeId) -> Option<NodeDescription>;
    fn get_my_id(&self) -> NodeId;
    fn get_parent_id(&self) -> Option<NodeId>;
    fn get_my_tags(&self) -> Vec<String>;
    /// Whether the node is connected to the market (i.e., doesn't have any
    /// parent = root of the network)
    fn is_market(&self) -> bool;
    fn get_parent_node_address(&self) -> Option<(IpAddr, FogNodeHTTPPort)>;
    fn get_market_node_address(&self) -> Option<(IpAddr, MarketHTTPPort)>;
    /// Return iter over both the parent and the children node...
    /// Aka all the nodes interesting that can accommodate a function
    fn get_neighbors(&self) -> Vec<NodeId>;
    /// Get the public ip associated with this server
    fn get_my_public_ip(&self) -> IpAddr;
    /// Get the public port associated with this server (HTTP)
    fn get_my_public_port_http(&self) -> FogNodeHTTPPort;
    /// Get the public port associated with this server (HTTP)
    fn get_my_public_port_faas(&self) -> FogNodeFaaSPortExternal;
    /// Get the specfied reserved memory configured
    fn get_reserved_memory(&self) -> Information;
    /// Get the specfied reserved cpu configured
    fn get_reserved_cpu(&self) -> Ratio;
}

#[derive(Debug)]
pub struct NodeSituationHashSetImpl {
    database: NodeSituationData,
}

impl NodeSituationHashSetImpl {
    pub fn new(situation: NodeSituationData) -> Self {
        Self { database: situation }
    }
}

impl NodeSituation for NodeSituationHashSetImpl {
    fn register(&self, id: NodeId, description: NodeDescription) {
        self.database.children.insert(id, description);
    }

    fn get_fog_node_neighbor(&self, id: &NodeId) -> Option<NodeDescription> {
        match self.database.children.get(id).map(|x| x.clone()) {
            Some(x) => Some(x),
            None => match &self.database.situation {
                NodeConnected {
                    parent_node_ip,
                    parent_node_port_http,
                    parent_id,
                    ..
                } => {
                    if parent_id == id {
                        Some(NodeDescription {
                            ip:        *parent_node_ip,
                            port_http: parent_node_port_http.clone(),
                        })
                    } else {
                        None
                    }
                }
                MarketConnected { .. } => None,
            },
        }
    }

    fn get_my_id(&self) -> NodeId { self.database.my_id.clone() }

    fn get_parent_id(&self) -> Option<NodeId> {
        match &self.database.situation {
            NodeConnected { parent_id, .. } => Some(parent_id.clone()),
            _ => None,
        }
    }

    fn get_my_tags(&self) -> Vec<String> { self.database.tags.clone() }

    fn is_market(&self) -> bool {
        matches!(self.database.situation, MarketConnected { .. })
    }

    fn get_parent_node_address(&self) -> Option<(IpAddr, FogNodeHTTPPort)> {
        match &self.database.situation {
            NodeConnected {
                parent_node_ip, parent_node_port_http, ..
            } => Some((*parent_node_ip, parent_node_port_http.clone())),
            _ => None,
        }
    }

    fn get_market_node_address(&self) -> Option<(IpAddr, MarketHTTPPort)> {
        match &self.database.situation {
            MarketConnected { market_ip, market_port, .. } => {
                Some((*market_ip, market_port.clone()))
            }
            _ => None,
        }
    }

    fn get_neighbors(&self) -> Vec<NodeId> {
        let mut ret: Vec<NodeId> =
            self.database.children.iter().map(|x| x.key().clone()).collect();

        match &self.database.situation {
            NodeConnected { parent_id, .. } => {
                ret.push(parent_id.clone());
            }
            MarketConnected { .. } => (),
        }

        ret
    }

    fn get_my_public_ip(&self) -> IpAddr { self.database.my_public_ip }

    fn get_my_public_port_http(&self) -> FogNodeHTTPPort {
        self.database.my_public_port_http.clone()
    }

    fn get_my_public_port_faas(&self) -> FogNodeFaaSPortExternal {
        self.database.my_public_port_faas.clone()
    }

    fn get_reserved_memory(&self) -> Information {
        self.database.reserved_memory
    }

    fn get_reserved_cpu(&self) -> Ratio { self.database.reserved_cpu }
}
