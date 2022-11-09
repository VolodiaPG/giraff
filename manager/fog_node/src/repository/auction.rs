use uuid::Uuid;

use model::dto::auction::BidRecord;
use model::BidId;

pub trait Auction: Sync + Send {
    fn insert(&self, auction: BidRecord) -> BidId;
    fn get(&self, id: &BidId) -> Option<BidRecord>;
    fn remove(&self, id: &BidId);
}

pub struct AuctionImpl {
    database: dashmap::DashMap<BidId, BidRecord>,
}

impl AuctionImpl {
    pub fn new() -> AuctionImpl {
        AuctionImpl { database: dashmap::DashMap::new() }
    }
}

impl Auction for AuctionImpl {
    fn insert(&self, auction: BidRecord) -> BidId {
        let id = BidId::from(Uuid::new_v4());
        self.database.insert(id.to_owned(), auction);
        id
    }

    fn get(&self, id: &BidId) -> Option<BidRecord> {
        self.database.get(id).map(|x| x.clone())
    }

    fn remove(&self, id: &BidId) { self.database.remove(id); }
}
