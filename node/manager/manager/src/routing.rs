use std::{collections::HashMap, fs, str::FromStr};

use if_chain::if_chain;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::models::BidId;
type NodeId = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum NodeCategory {
    Parent,
    Child
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub uri: String,
    pub id: NodeId,
    pub category: NodeCategory,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeSituation {
    nodes: HashMap<NodeId, Node>,
    pub market_uri: String,
    pub is_market: bool,
}

impl NodeSituation {
    pub fn new(path: String) -> Self {
        if_chain! {
            if let Ok(content) = fs::read_to_string(path.clone());
            if let Ok(situation) = ron::from_str::<NodeSituation>(&content);
            then
            {
                info!("Loading nodes from disk, path: {}", path);
                situation
            }
            else
            {
                warn!("No node situation config found on disk, tried path: {}", path);
                NodeSituation {
                    ..Default::default()
                }
            }
        }
    }

    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }
}

#[derive(Debug, Default)]
pub struct RoutingTable{
    pub routes: HashMap<BidId, NodeId>,
}

#[derive(Debug)]
pub enum Forward {
    Outside(BidId, NodeId),
    Inside(BidId),
    ToMarket(BidId),
}

impl RoutingTable{
    pub async fn update_route(&mut self, source: BidId, target: NodeId) {
        self.routes.insert(source, target);
    }

    pub async fn route(&self, source: BidId) -> Forward {
        lazy_static! {
            static ref DEFAULT_NODE: NodeId = Uuid::from_str("00000000-0000-0000-0000-000000000000").unwrap();
        }
        match self.routes.get(&source){
            Some(node) => {
                if node.eq(&DEFAULT_NODE) {
                    Forward::Inside(source)
                } else {
                    Forward::Outside(source, *node)
                }
            },
            None => Forward::ToMarket(source),
        }
    }
}