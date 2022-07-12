use std::collections::HashMap;
use std::fmt::Debug;

use async_trait::async_trait;
use tokio::sync::RwLock;

use manager::model::dto::faas::ProvisionedRecord;
use manager::model::BidId;

#[async_trait]
pub trait Provisioned: Debug + Sync + Send {
    async fn insert(&self, id: BidId, record: ProvisionedRecord);
    async fn get(&self, id: &BidId) -> Option<ProvisionedRecord>;
}

#[derive(Debug)]
pub struct ProvisionedHashMapImpl {
    database: RwLock<HashMap<BidId, ProvisionedRecord>>,
}

impl ProvisionedHashMapImpl {
    pub fn new() -> Self { Self { database: RwLock::new(HashMap::new()) } }
}

#[async_trait]
impl Provisioned for ProvisionedHashMapImpl {
    async fn insert(&self, id: BidId, bid: ProvisionedRecord) {
        self.database.write().await.insert(id, bid);
    }

    async fn get(&self, id: &BidId) -> Option<ProvisionedRecord> {
        self.database.read().await.get(id).cloned()
    }
}
