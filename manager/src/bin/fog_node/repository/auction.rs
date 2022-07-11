use std::collections::HashMap;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use manager::model::{dto::auction::BidRecord, BidId};

#[async_trait]
pub trait Auction: Sync + Send {
    async fn insert(&self, auction: BidRecord) -> BidId;
    async fn get(&self, id: &BidId) -> Option<BidRecord>;
    async fn remove(&self, id: &BidId);
}

pub struct AuctionImpl {
    database: RwLock<HashMap<BidId, BidRecord>>,
}

impl AuctionImpl {
    pub fn new() -> AuctionImpl { AuctionImpl { database: RwLock::new(HashMap::new()) } }
}

#[async_trait]
impl Auction for AuctionImpl {
    async fn insert(&self, auction: BidRecord) -> BidId {
        let id = BidId::from(Uuid::new_v4());
        self.database.write().await.insert(id.to_owned(), auction);
        id
    }

    async fn get(&self, id: &BidId) -> Option<BidRecord> {
        self.database.read().await.get(id).cloned()
    }

    async fn remove(&self, id: &BidId) { self.database.write().await.remove(id); }
}
