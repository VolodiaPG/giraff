use async_trait::async_trait;
use uuid::Uuid;

use model::dto::auction::BidRecord;
use model::BidId;

#[async_trait]
pub trait Auction: Sync + Send {
    async fn insert(&self, auction: BidRecord) -> BidId;
    async fn get(&self, id: &BidId) -> Option<BidRecord>;
    async fn remove(&self, id: &BidId);
}

pub struct AuctionImpl {
    database: flurry::HashMap<BidId, BidRecord>,
}

impl AuctionImpl {
    pub fn new() -> AuctionImpl {
        AuctionImpl { database: flurry::HashMap::new() }
    }
}

#[async_trait]
impl Auction for AuctionImpl {
    async fn insert(&self, auction: BidRecord) -> BidId {
        let id = BidId::from(Uuid::new_v4());
        self.database.pin().insert(id.to_owned(), auction);
        id
    }

    async fn get(&self, id: &BidId) -> Option<BidRecord> {
        self.database.pin().get(id).cloned()
    }

    async fn remove(&self, id: &BidId) { self.database.pin().remove(id); }
}
