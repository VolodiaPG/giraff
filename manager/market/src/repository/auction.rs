use model::dto::function::ChosenBid;
use model::view::auction::BidProposal;

pub struct Auction;

impl Auction {
    pub fn new() -> Self { Self {} }
}

impl Auction {
    #[cfg(feature = "powerrandom")]
    pub fn auction(&self, bids: &[BidProposal]) -> Option<ChosenBid> {
        use rand::Rng;

        let bids = bids.iter().collect::<Vec<_>>();
        let first = bids.get(0).cloned().cloned();
        let second = bids.get(1).cloned().cloned();
        match (first, second) {
            (Some(first), Some(second)) => {
                let rand_choice = rand::thread_rng().gen_range(0..2);
                if rand_choice == 0 {
                    Some(ChosenBid { price: first.bid, bid: first })
                } else {
                    Some(ChosenBid { price: second.bid, bid: second })
                }
            }
            (Some(first), None) => {
                Some(ChosenBid { price: first.bid, bid: first })
            }
            _ => None,
        }
    }

    #[cfg(not(feature = "powerrandom"))]
    pub fn auction(&self, bids: &[BidProposal]) -> Option<ChosenBid> {
        let mut bids = bids.iter().collect::<Vec<_>>();
        bids.sort_unstable_by(|a, b| a.bid.partial_cmp(&b.bid).unwrap()); // Sort asc
        let first = bids.get(0).cloned().cloned();
        let second = bids.get(1);
        match (first, second) {
            (Some(first), Some(second)) => {
                Some(ChosenBid { price: second.bid, bid: first })
            }
            (Some(first), None) => {
                Some(ChosenBid { price: first.bid, bid: first })
            }
            _ => None,
        }
    }
}
