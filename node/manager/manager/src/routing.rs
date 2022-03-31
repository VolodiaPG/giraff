use std::{collections::HashMap, fs, str::FromStr};

use if_chain::if_chain;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use lazy_static::lazy_static;

use crate::models::BidId;
type NodeId = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
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

#[derive(Debug, Clone, Default)]
pub struct NodeSituation{
    nodes: HashMap<NodeId, Node>,
    pub to_market: Option<Node>,
    pub is_market: bool,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeSituationDisk {
    pub nodes: Vec<Node>,
}

impl From<NodeSituationDisk> for NodeSituation {
    fn from(disk: NodeSituationDisk) -> Self {
        let nodes: HashMap<NodeId, Node> = disk.nodes.into_iter().map(|node| (node.id, node)).collect();
        let to_market = nodes.clone().into_iter().find(|(_id, node)| node.category == NodeCategory::Parent).map(|(_id, node)| node);
        let is_market = to_market.is_none();
        NodeSituation {
            nodes,
            to_market,
            is_market,
        }
    }
}

impl NodeSituationDisk {
    pub fn new(path: String) -> Self {
        if_chain! {
            if let Ok(content) = fs::read_to_string(path.clone());
            if let Ok(situation) = ron::from_str::<NodeSituationDisk>(&content);
            then
            {
                info!("Loading nodes from disk, path: {}", path);
                situation
            }
            else
            {
                warn!("No node situation config found on disk, tried path: {}", path);
                NodeSituationDisk {
                    ..Default::default()
                }
            }
        }
    }
}
impl NodeSituation{
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