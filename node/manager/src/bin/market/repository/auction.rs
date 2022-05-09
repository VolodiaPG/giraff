use manager::model::dto::auction::ChosenBid;
use manager::model::view::auction::BidProposal;

pub trait Auction: Sync + Send {
    fn auction(&self, bids: &[BidProposal]) -> Option<ChosenBid>;
}

pub struct SecondPriceAuction;

impl SecondPriceAuction {
    pub fn new() -> Self {
        Self {}
    }
}

impl Auction for SecondPriceAuction {
    fn auction(&self, bids: &[BidProposal]) -> Option<ChosenBid> {
        let mut bids = bids.iter().collect::<Vec<_>>();
        bids.sort_unstable_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap());
        bids.reverse();
        let first = bids.get(0);
        let second = bids.get(1);
        match (first.cloned().cloned(), second) {
            (Some(first), Some(second)) => Some(ChosenBid {
                price: second.bid,
                bid: first,
            }),
            (Some(first), None) => Some(ChosenBid {
                price: first.bid,
                bid: first,
            }),
            _ => None,
        }
    }
}
