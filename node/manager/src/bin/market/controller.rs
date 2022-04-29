use anyhow::Result;
use manager::model::{view::sla::PutSla, NodeId};
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Register a SLA and starts the auctionning process
pub async fn start_auction(
    leaf_node: NodeId,
    payload: PutSla,
    auction_service: &Arc<dyn crate::service::auction::Auction>,
) -> Result<(), ControllerError> {
    trace!("put sla: {:?}", payload);

    auction_service
        .call_for_bids(leaf_node, &payload.sla)
        .await?;
    // Ok(auction_service.do_auction(bids).await?)
    // Todo: start cron job to then do the auction + end the auction when all bids have been studied : level 2
    Ok(())
}
