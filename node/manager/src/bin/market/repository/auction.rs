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
        match bids.get(0).cloned().cloned() {
            Some(bid) => {
                // TODO: this is not right !!!!
                let price = if let Some(second_bid) = bids.get(1) {
                    second_bid.bid
                } else {
                    bid.bid
                };
                Some(ChosenBid { bid, price })
            }
            None => None,
        }
    }
}
