use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use manager::model::{domain::{auction::AuctionResult, sla::Sla},
                     view::auction::BidProposals,
                     NodeId};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("No winner were selected after the auction took place")]
    NoWinner,
    #[error("Stack to targeted node is empty: {0}.")]
    RequestFailed(#[from] crate::repository::node_communication::Error),
}

#[async_trait]
pub trait Auction: Send + Sync {
    /// Call fog nodes for the bids. Will get their bid proposals
    async fn call_for_bids(&self, leaf_node: NodeId, sla: Sla) -> Result<BidProposals, Error>;

    /// Execute the auction process and find the winner among the bid proposal
    async fn do_auction(&self, proposals: &BidProposals) -> Result<AuctionResult, Error>;
}

pub struct AuctionImpl {
    auction_process:    Arc<dyn crate::repository::auction::Auction>,
    node_communication: Arc<dyn crate::repository::node_communication::NodeCommunication>,
}

impl AuctionImpl {
    pub fn new(auction_process: Arc<dyn crate::repository::auction::Auction>,
               node_communication: Arc<dyn crate::repository::node_communication::NodeCommunication>)
               -> Self {
        AuctionImpl { auction_process, node_communication }
    }
}
#[async_trait]
impl Auction for AuctionImpl {
    async fn call_for_bids(&self, leaf_node: NodeId, sla: Sla) -> Result<BidProposals, Error> {
        trace!("call for bids: {:?}", sla);

        Ok(self.node_communication.request_bids_from_node(leaf_node, sla).await?)
    }

    async fn do_auction(&self, proposals: &BidProposals) -> Result<AuctionResult, Error> {
        trace!("do auction: {:?}", proposals);
        let auction_result =
            self.auction_process.auction(&proposals.bids).ok_or(Error::NoWinner)?;
        Ok(AuctionResult { chosen_bid: auction_result })
    }
}
