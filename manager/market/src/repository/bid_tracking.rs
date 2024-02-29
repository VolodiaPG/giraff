use dashmap::DashMap;
use model::{NodeId, SlaId};

#[derive(Debug)]
pub struct BidTracking {
    node_association: dashmap::DashMap<SlaId, NodeId>,
}

impl BidTracking {
    pub fn new() -> Self { Self { node_association: DashMap::new() } }

    pub fn save(&self, id: SlaId, node: NodeId) {
        self.node_association.insert(id, node);
    }

    pub fn get(&self, id: &SlaId) -> Option<NodeId> {
        match self.node_association.get(id) {
            Some(node) => Some(node.clone()),
            None => None,
        }
    }
}
