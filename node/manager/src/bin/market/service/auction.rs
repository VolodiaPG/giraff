use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;

use manager::model::domain::auction::{AuctionResult, AuctionStatus, AuctionSummary};
use manager::model::dto::node::NodeRecord;
use manager::model::{domain::sla::Sla, NodeId};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Node Id was not found: {0}.")]
    NodeIdNotFound(NodeId),
    #[error("Auction status doesn't allow that action to take place.")]
    AuctionNotInRightStatus,
    #[error("No winner were selected after the auction took place")]
    NoWinner,
    #[error("Stack to targeted node is empty: {0}.")]
    StackEmpty(NodeId),
    #[error(transparent)]
    RequestFailed(#[from] crate::repository::node_communication::Error),
    #[error("The first node is the routing stack is not the routing node, and thus not URI can be deduced from a lower node that than the unique root node.")]
    FirstNodeInStackIsNotRootNode,
}

#[async_trait]
pub trait Auction: Send + Sync {
    /// Call fog nodes for the bids. Will get their bid proposals
    async fn call_for_bids(&self, leaf_node: NodeId, sla: &Sla) -> Result<(), Error>;

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
    async fn call_for_bids(&self, leaf_node: NodeId, sla: &Sla) -> Result<(), Error> {
        trace!("call for bids: {:?}", sla);
        let first_node_route_stack = self.fog_node.get_route_to_node(leaf_node.clone()).await;
        trace!(
            "sending the call to first node route stack: {:?}",
            first_node_route_stack
        );

        let (ip, port) = match self
            .fog_node
            .get(
                first_node_route_stack
                    .get(first_node_route_stack.len() - 1)
                    .ok_or(Error::StackEmpty(leaf_node.clone()))?,
            )
            .await
            .ok_or(Error::NodeIdNotFound(leaf_node))?
            .data
        {
            NodeRecord { ip, port, .. } => {
                let ip = ip.ok_or(Error::FirstNodeInStackIsNotRootNode)?.clone();
                let port = port.ok_or(Error::FirstNodeInStackIsNotRootNode)?.clone();
                (ip, port)
            }
        };

        self.node_communication
            .request_bid_from_node(ip, port, first_node_route_stack, sla)
            .await?;

        Ok(())
    }

    async fn do_auction(&self, summary: AuctionSummary) -> Result<AuctionResult, Error> {
        trace!("do auction: {:?}", summary);
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
