use std::fmt::Debug;

use model::dto::routing::Direction;
use model::BidId;

pub trait FaaSRoutingTable: Debug + Sync + Send {
    /// Update the breadcrumb route to the [BidId] passing by the next
    /// [NodeId].
    fn update(&self, source: BidId, target: Direction);

    fn get(&self, bid_id: &BidId) -> Option<Direction>;
}

#[derive(Debug)]
pub struct FaaSRoutingTableHashMap {
    table: dashmap::DashMap<BidId, Direction>,
}

impl FaaSRoutingTableHashMap {
    pub fn new() -> Self { Self { table: dashmap::DashMap::new() } }
}

impl FaaSRoutingTable for FaaSRoutingTableHashMap {
    fn update(&self, source: BidId, target: Direction) {
        trace!("Updating routing table {} -> {:?}", source, target);
        self.table.insert(source, target);
    }

    fn get(&self, bid_id: &BidId) -> Option<Direction> {
        self.table.get(bid_id).map(|x| x.clone())
    }
}
