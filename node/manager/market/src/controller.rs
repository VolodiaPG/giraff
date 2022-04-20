use std::sync::Arc;

use chrono::Utc;

use tokio::sync::Mutex;

use crate::model::AuctionStatus;
use crate::{auction, service};
use if_chain::if_chain;
use shared_models::{
    view::{
        auction::MarketBidProposal,
        node::{PostNode, PostNodeResponse},
        sla::PutSla,
    },
    NodeId,
};

use anyhow::Result;

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error("Node {0} was not found")]
    NodeIdNotFound(NodeId),
    #[error("The auction was not in the right status to still accept bids")]
    AuctionNotInRightStatus,
    #[error("The auction failed and did not return a proposal")]
    AuctionFailed,
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

// impl From<anyhow::Error> for ControllerError {
//     fn from(err: anyhow::Error) -> Self {
//         ControllerError::Failed(err)
//     }
// }

/// Register a SLA and starts the auctionning process
pub async fn process_function_host(
    leaf_node: NodeId,
    bid_db: Arc<Mutex<service::live_store::BidDataBase>>,
    nodes_db: Arc<Mutex<service::live_store::NodesDataBase>>,
    payload: PutSla,
) -> Result<MarketBidProposal, ControllerError> {
    trace!("put sla: {:?}", payload);

    let id = service::auction::call_for_bids(
        payload.sla.clone(),
        bid_db.clone(),
        nodes_db.clone(),
        leaf_node,
    )
    .await
    .unwrap();

    let res;
    {
        res = bid_db.lock().await.get(&id).unwrap().clone();
    }

    let (bids, auction_result) = match &res.auction {
        AuctionStatus::Active(bids) => (bids, auction::second_price(&payload.sla, bids)),
        _ => return Err(ControllerError::AuctionNotInRightStatus),
    };

    let (bid, proposed_price) = auction_result.ok_or(ControllerError::AuctionFailed)?;

    let node = nodes_db
        .lock()
        .await
        .get(&bid.node_id)
        .map(|node| node.data.clone())
        .ok_or_else(|| ControllerError::NodeIdNotFound(bid.node_id.to_owned()))?;

    service::auction::take_offer(&node, &bid).await?;

    {
        bid_db
            .lock()
            .await
            .update_auction(&id, AuctionStatus::Finished(bid.clone()));
    }

    Ok(MarketBidProposal {
        bids: bids.to_owned(),
        chosen_bid: Some(bid),
        price: Some(proposed_price),
    })
}

/// Patch part of the node data
pub async fn post_nodes(
    nodes_db: Arc<Mutex<service::live_store::NodesDataBase>>,
    payload: PostNode,
) -> Result<PostNodeResponse, ControllerError> {
    trace!("patch the node @{}", &payload.from);
    nodes_db
        .lock()
        .await
        .get_mut(&payload.from)
        .ok_or_else(|| ControllerError::NodeIdNotFound(payload.from.clone()))?
        .latency_to_market
        .update(Utc::now(), payload.created_at);

    if_chain! {
        if let Some(last_answered_at) = payload.last_answered_at;
        if let Some(last_answer_received_at) = payload.last_answer_received_at;
        then {
            nodes_db
                .lock()
                .await
                .get_mut(&payload.from)
                .ok_or_else(|| ControllerError::NodeIdNotFound(payload.from.clone()))?
                .latency_to_node
                .update(last_answer_received_at, last_answered_at);
        }
    }

    Ok(PostNodeResponse {
        answered_at: Utc::now(),
    })
}
