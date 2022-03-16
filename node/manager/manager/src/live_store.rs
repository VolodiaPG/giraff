use std::collections::HashMap;

use uuid::Uuid;

use crate::models::{BidRecord, ProvisionedRecord};

pub struct BidDataBase {
    database: HashMap<Uuid, BidRecord>,
}

pub struct ProvisionedDataBase {
    database: HashMap<Uuid, ProvisionedRecord>,
}

impl BidDataBase {
    pub fn new() -> Self {
        BidDataBase {
            database: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bid: BidRecord) -> Uuid {
        let uuid = Uuid::new_v4();
        self.database.insert(uuid, bid);
        uuid
    }

    pub fn get(&self, id: &Uuid) -> Option<&BidRecord> {
        self.database.get(id)
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.database.remove(id);
    }
}

impl ProvisionedDataBase {
    pub fn new() -> Self {
        ProvisionedDataBase {
            database: HashMap::new(),
        }
    }

    pub fn insert(&mut self, bid: ProvisionedRecord) -> Uuid {
        let uuid = Uuid::new_v4();
        self.database.insert(uuid, bid);
        uuid
    }

    pub fn get(&self, id: &Uuid) -> Option<&ProvisionedRecord> {
        self.database.get(id)
    }

    pub fn remove(&mut self, id: &Uuid) {
        self.database.remove(id);
    }
}
