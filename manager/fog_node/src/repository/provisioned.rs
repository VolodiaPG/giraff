use std::fmt::Debug;

use model::dto::faas::ProvisionedRecord;
use model::BidId;

pub trait Provisioned: Debug + Sync + Send {
    fn insert(&self, id: BidId, record: ProvisionedRecord);
    fn get(&self, id: &BidId) -> Option<ProvisionedRecord>;
}

#[derive(Debug)]
pub struct ProvisionedHashMapImpl {
    database: dashmap::DashMap<BidId, ProvisionedRecord>,
}

impl ProvisionedHashMapImpl {
    pub fn new() -> Self { Self { database: dashmap::DashMap::new() } }
}

impl Provisioned for ProvisionedHashMapImpl {
    fn insert(&self, id: BidId, bid: ProvisionedRecord) {
        self.database.insert(id, bid);
    }

    fn get(&self, id: &BidId) -> Option<ProvisionedRecord> {
        self.database.get(id).map(|x| x.clone())
    }
}
