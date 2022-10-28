use std::fmt::Debug;

use async_trait::async_trait;

use model::dto::faas::ProvisionedRecord;
use model::BidId;

#[async_trait]
pub trait Provisioned: Debug + Sync + Send {
    async fn insert(&self, id: BidId, record: ProvisionedRecord);
    async fn get(&self, id: &BidId) -> Option<ProvisionedRecord>;
}

#[derive(Debug)]
pub struct ProvisionedHashMapImpl {
    database: flurry::HashMap<BidId, ProvisionedRecord>,
}

impl ProvisionedHashMapImpl {
    pub fn new() -> Self { Self { database: flurry::HashMap::new() } }
}

#[async_trait]
impl Provisioned for ProvisionedHashMapImpl {
    async fn insert(&self, id: BidId, bid: ProvisionedRecord) {
        self.database.pin().insert(id, bid);
    }

    async fn get(&self, id: &BidId) -> Option<ProvisionedRecord> {
        self.database.pin().get(id).map(|x| x.clone())
    }
}
