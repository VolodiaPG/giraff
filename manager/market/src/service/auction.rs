use anyhow::Result;
use model::domain::auction::AuctionResult;
use model::domain::sla::Sla;
use model::view::auction::BidProposals;
use model::NodeId;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No winner were selected after the auction took place")]
    NoWinner,
    #[error("Failed to send the request to the first node: {0}.")]
    RequestFailed(#[from] crate::repository::node_communication::Error),
}

pub struct Auction {
    auction_process:    Arc<crate::repository::auction::Auction>,
    node_communication:
        Arc<crate::repository::node_communication::NodeCommunication>,
}

impl Auction {
    pub fn new(
        auction_process: Arc<crate::repository::auction::Auction>,
        node_communication: Arc<
            crate::repository::node_communication::NodeCommunication,
        >,
    ) -> Self {
        Self { auction_process, node_communication }
    }

    pub async fn call_for_bids(
        &self,
        to: NodeId,
        sla: &'_ Sla,
    ) -> Result<BidProposals, Error> {
        trace!("call for bids: {:?}", sla);

        Ok(self.node_communication.request_bids_from_node(to, sla).await?)
    }

    pub async fn do_auction(
        &self,
        proposals: &BidProposals,
    ) -> Result<AuctionResult, Error> {
        trace!("do auction: {:?}", proposals);
        let auction_result = self
            .auction_process
            .auction(&proposals.bids)
            .ok_or(Error::NoWinner)?;
        Ok(AuctionResult { chosen_bid: auction_result })
    }
}
