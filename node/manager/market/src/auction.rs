use shared_models::{domain::sla::Sla, view::auction::BidProposal};

pub type Price = f64;

pub fn second_price(_sla: &Sla, bids: &[BidProposal]) -> Option<(BidProposal, Price)> {
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
            Some((bid, price))
        }
        None => None,
    }
}
