use std::sync::Arc;

use anyhow::Result;
use chrono::Utc;
use if_chain::if_chain;
use tokio::sync::Mutex;

use manager::model::domain::auction::AuctionResult;
use manager::model::{
    view::{
        node::{PostNode, PostNodeResponse},
        sla::PutSla,
    },
    NodeId,
};

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Register a SLA and starts the auctionning process
pub async fn process_function_host(
    leaf_node: NodeId,
    payload: PutSla,
    auction_service: &Arc<dyn crate::service::auction::Auction>,
) -> Result<AuctionResult, ControllerError> {
    trace!("put sla: {:?}", payload);

    let bids = auction_service.call_for_bids(leaf_node, &payload.sla).await;
    Ok(auction_service.do_auction(bids).await?)
}
