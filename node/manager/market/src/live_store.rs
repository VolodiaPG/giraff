use std::collections::HashMap;

use chrono::{Duration, Utc};
use sla::Sla;
use uuid::Uuid;

use crate::models::{BidRecord, ClientId, NodeId, NodeRecord};

pub struct BidDataBase {
    database: HashMap<ClientId, BidRecord>,
}

pub struct NodesDataBase {
    database: HashMap<ClientId, NodeRecord>,
}

impl BidDataBase {
    pub fn new() -> Self {
        BidDataBase {
            database: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bid: BidRecord) -> ClientId {
        let uuid = Uuid::new_v4();
        self.database.insert(uuid, bid);
        uuid
    }

    pub fn get(&self, id: &ClientId) -> Option<&BidRecord> {
        self.database.get(id)
    }

    pub fn get_mut(&mut self, id: &ClientId) -> Option<&mut BidRecord> {
        self.database.get_mut(id)
    }

    pub fn remove(&mut self, id: &ClientId) {
        self.database.remove(id);
    }
}

impl NodesDataBase {
    pub fn new() -> Self {
        NodesDataBase {
            database: HashMap::new(),
        }
    }

    pub fn insert(&mut self, node: NodeRecord) -> NodeId {
        let uuid = Uuid::new_v4();
        self.database.insert(uuid, node);
        uuid
    }

    pub fn get(&mut self, id: &ClientId) -> Option<&mut NodeRecord> {
        self.database.get_mut(id)
    }

    pub fn remove(&mut self, id: &NodeId) {
        self.database.remove(id);
    }

    pub fn get_bid_candidates(&self, sla: &Sla) -> HashMap<NodeId, NodeRecord> {
        self.database
            .iter()
            .filter(|&(id, node)| {
                node.latency.get_avg() < sla.latency_max
                    && Utc::now() - node.latency.get_last_update() > Duration::seconds(10)
            })
            .map(|(id, node)| (id.clone(), node.clone()))
            .collect()
    }
}
