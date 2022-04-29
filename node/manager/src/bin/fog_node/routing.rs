use std::{collections::HashMap, fs};

use if_chain::if_chain;
use serde::{Deserialize, Serialize};

use manager::model::NodeId;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum NodeCategory {
    Parent,
    Child,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Node {
    pub uri: String,
    pub id: NodeId,
    pub category: NodeCategory,
}

#[derive(Debug, Clone, Default)]
pub struct NodeSituation {
    nodes: HashMap<NodeId, Node>,
    pub to_market: Option<Node>,
    pub is_market: bool,
    pub market_url: Option<String>,
    pub my_id: NodeId,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NodeSituationDisk {
    pub nodes: Vec<Node>,
    pub market_url: Option<String>,
    pub my_id: NodeId,
}

impl From<NodeSituationDisk> for NodeSituation {
    fn from(disk: NodeSituationDisk) -> Self {
        let nodes: HashMap<NodeId, Node> = disk
            .nodes
            .into_iter()
            .map(|node| (node.id.clone(), node))
            .collect();
        let to_market = nodes
            .clone()
            .into_iter()
            .find(|(_id, node)| node.category == NodeCategory::Parent)
            .map(|(_id, node)| node);
        let is_market = to_market.is_none();
        let my_id = disk.my_id;
        NodeSituation {
            nodes,
            to_market,
            is_market,
            market_url: if is_market { disk.market_url } else { None },
            my_id,
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

impl NodeSituation {
    pub fn get(&self, id: &NodeId) -> Option<&Node> {
        self.nodes.get(id)
    }
}
