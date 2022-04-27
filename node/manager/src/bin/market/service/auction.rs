use std::borrow::Cow;
use std::future::join;
use std::{convert::Infallible, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use futures::future::join_all;
use if_chain::if_chain;

use manager::model::domain::auction::{AuctionResult, AuctionStatus, AuctionSummary};
use manager::model::dto::auction::BidRecord;
use manager::model::{
    domain::sla::Sla,
    view::auction::{Bid, BidProposal},
    BidId, NodeId,
};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Node Id was not found: {0}.")]
    NodeIdNotFound(NodeId),
    #[error("Auction status doesn't allow that action to take place.")]
    AuctionNotInRightStatus,
    #[error("No winner were selected after the auction took place")]
    NoWinner,
}

#[async_trait]
pub trait Auction: Send + Sync {
    /// Call fog nodes for the bids. Will get their bid proposals
    async fn call_for_bids(&self, leaf_node: NodeId, sla: &Sla) -> AuctionSummary;

    /// Execute the auction process and find the winner among the bid proposal
    async fn do_auction(&self, summary: AuctionSummary) -> Result<AuctionResult, Error>;
}

pub struct AuctionImpl {
    fog_node: Arc<dyn crate::repository::fog_node::FogNode>,
    auction_process: Arc<dyn crate::repository::auction::Auction>,
    node_communication: Arc<dyn crate::repository::node_communication::NodeCommunication>,
}

impl AuctionImpl {
    pub fn new(
        fog_node: Arc<dyn crate::repository::fog_node::FogNode>,
        auction_process: Arc<dyn crate::repository::auction::Auction>,
        node_communication: Arc<dyn crate::repository::node_communication::NodeCommunication>,
    ) -> Self {
        AuctionImpl {
            fog_node,
            auction_process,
            node_communication,
        }
    }
}
#[async_trait]
impl Auction for AuctionImpl {
    async fn call_for_bids(&self, leaf_node: NodeId, sla: &Sla) -> AuctionSummary {
        trace!("call for bids: {:?}", sla);

        let nodes = self.fog_node.get_bid_candidates(&sla, leaf_node).await;

        let mut handles = Vec::new();
        for (node_id, node_record) in nodes.iter() {
            handles.push(self.node_communication.request_bid_from_node(
                node_id.to_owned(),
                &node_record,
                sla,
            ));
        }

        let bids = join_all(handles)
            .await
            .into_iter()
            .filter(|x| x.is_ok())
            .map(|x| x.unwrap())
            .collect::<Vec<_>>();

        AuctionSummary {
            status: AuctionStatus::Active(bids),
        }
    }

    async fn do_auction(&self, summary: AuctionSummary) -> Result<AuctionResult, Error> {
        let bids = match summary.status {
            AuctionStatus::Active(bids) => bids,
            _ => return Err(Error::AuctionNotInRightStatus),
        };
        let auction_result = self.auction_process.auction(&bids).ok_or(Error::NoWinner)?;
        Ok(AuctionResult {
            bids,
            chosen_bid: auction_result,
        })
    }
}
