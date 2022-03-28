use std::{collections::HashMap, fs};

use if_chain::if_chain;
use serde::{Deserialize, Serialize};
type NodeId = uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct Node {
    pub url: String,
    pub id: NodeId,
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
pub struct NodeSitutation {
    pub next_node: Option<Node>,
    pub lower_nodes: HashMap<NodeId, Node>,
}

impl NodeSitutation {
    pub fn new(path: String) -> Self {
        if_chain! {
            if let Ok(content) = fs::read_to_string(path.clone());
            if let Ok(situation) = ron::from_str::<NodeSitutation>(&content);
            then
            {
                info!("Loading nodes from disk, path: {}", path);
                situation
            }
            else
            {
                debug!("No nodes found on disk, path: {}", path);
                NodeSitutation {
                    ..Default::default()
                }
            }
        }
    }
}
