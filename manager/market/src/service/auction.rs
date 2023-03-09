use anyhow::{anyhow, Context, Result};
use model::domain::auction::AuctionResult;
use model::domain::sla::Sla;
use model::view::auction::BidProposals;
use model::NodeId;
use std::sync::Arc;

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
    ) -> Result<BidProposals> {
        trace!("call for bids: {:?}", sla);

        self.node_communication
            .request_bids_from_node(to.clone(), sla)
            .await
            .with_context(|| format!("Failed to get bids from {}", to))
    }

    pub async fn do_auction(
        &self,
        proposals: &BidProposals,
    ) -> Result<AuctionResult> {
        trace!("do auction: {:?}", proposals);
        let auction_result =
            self.auction_process.auction(&proposals.bids).ok_or_else(
                || anyhow!("Auction failed, no winners were selected"),
            )?;
        Ok(AuctionResult { chosen_bid: auction_result })
    }
}
