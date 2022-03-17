use std::collections::HashMap;

use uuid::Uuid;

use crate::models::{BidRecord, ProvisionedRecord};

pub type ClientId = Uuid;
pub type BidId = Uuid;

pub struct BidDataBase {
    database: HashMap<ClientId, BidRecord>,
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

    pub fn remove(&mut self, id: &ClientId) {
        self.database.remove(id);
    }
}
