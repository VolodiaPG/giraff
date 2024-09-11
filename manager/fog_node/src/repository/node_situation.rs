use model::dto::node::NodeCategory::{MarketConnected, NodeConnected};
use model::dto::node::{MaxInFlight, NodeDescription, NodeSituationData};
use model::{
    FogNodeFaaSPortExternal, FogNodeHTTPPort, MarketHTTPPort, NodeId,
};
use std::fmt::Debug;
use std::net::IpAddr;
#[cfg(feature = "offline")]
use uom::si::f64::Time;
use uom::si::rational64::{Information, InformationRate, Ratio};

#[derive(Debug)]
pub struct NodeSituation {
    database: NodeSituationData,
}

impl NodeSituation {
    pub fn new(situation: NodeSituationData) -> Self {
        Self { database: situation }
    }

    pub fn register(&self, id: NodeId, description: NodeDescription) {
        self.database.children.insert(id, description);
    }

    pub fn get_fog_node_neighbor(
        &self,
        id: &NodeId,
    ) -> Option<NodeDescription> {
        match self.database.children.get(id).map(|x| x.clone()) {
            Some(x) => Some(x),
            None => match &self.database.situation {
                NodeConnected {
                    parent_node_ip,
                    parent_node_port_http,
                    parent_id,
                    #[cfg(feature = "offline")]
                    parent_latency,
                    ..
                } => {
                    if parent_id == id {
                        Some(NodeDescription {
                            ip: *parent_node_ip,
                            port_http: parent_node_port_http.clone(),
                            advertised_bandwidth: self
                                .get_my_advertised_bandwidth(),
                            #[cfg(feature = "offline")]
                            latency: *parent_latency,
                        })
                    } else {
                        None
                    }
                }
                MarketConnected { .. } => None,
            },
        }
    }

    pub fn get_my_id(&self) -> NodeId { self.database.my_id.clone() }

    pub fn get_my_advertised_bandwidth(&self) -> InformationRate {
        self.database.my_advertised_bandwidth.clone()
    }

    pub fn get_max_in_flight_functions_proposals(&self) -> MaxInFlight {
        self.database.max_in_flight_functions_proposals.clone()
    }

    pub fn get_parent_id(&self) -> Option<NodeId> {
        match &self.database.situation {
            NodeConnected { parent_id, .. } => Some(parent_id.clone()),
            _ => None,
        }
    }

    pub fn get_my_tags(&self) -> Vec<String> { self.database.tags.clone() }

    pub fn is_market(&self) -> bool {
        matches!(self.database.situation, MarketConnected { .. })
    }

    pub fn get_parent_node_address(
        &self,
    ) -> Option<(IpAddr, FogNodeHTTPPort)> {
        match &self.database.situation {
            NodeConnected {
                parent_node_ip, parent_node_port_http, ..
            } => Some((*parent_node_ip, parent_node_port_http.clone())),
            _ => None,
        }
    }

    pub fn get_market_node_address(&self) -> Option<(IpAddr, MarketHTTPPort)> {
        match &self.database.situation {
            MarketConnected { market_ip, market_port, .. } => {
                Some((*market_ip, market_port.clone()))
            }
            _ => None,
        }
    }

    pub fn get_neighbors(&self) -> Vec<NodeId> {
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

    pub fn get_my_public_ip(&self) -> IpAddr { self.database.my_public_ip }

    pub fn get_my_public_port_http(&self) -> FogNodeHTTPPort {
        self.database.my_public_port_http.clone()
    }

    pub fn get_my_public_port_faas(&self) -> FogNodeFaaSPortExternal {
        self.database.my_public_port_faas.clone()
    }

    pub fn get_reserved_memory(&self) -> Information {
        self.database.reserved_memory
    }

    pub fn get_reserved_cpu(&self) -> Ratio { self.database.reserved_cpu }

    #[cfg(feature = "offline")]
    pub fn get_my_offline_latency(&self) -> Time {
        match &self.database.situation {
            NodeConnected { parent_latency, .. } => *parent_latency,
            MarketConnected { .. } => Time::new::<uom::si::time::second>(0.0),
        }
    }
}
