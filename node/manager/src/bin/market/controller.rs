use anyhow::Result;
use manager::model::view::node::RegisterNode;
use manager::model::view::sla::PutSla;
use std::sync::Arc;

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    FogNodeNetwork(#[from] crate::service::fog_node_network::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Register a SLA and starts the auctioning process, can take a while.
// TODO define "a while"; set a timeout
pub async fn start_auction(
    payload: PutSla,
    auction_service: &Arc<dyn crate::service::auction::Auction>,
) -> Result<(), ControllerError> {
    trace!("put sla: {:?}", payload);

    auction_service
        .call_for_bids(payload.target_node, &payload.sla)
        .await?;
    // Ok(auction_service.do_auction(bids).await?)
    // Todo: start cron job to then do the auction + end the auction when all bids have been studied : level 2
    Ok(())
}

/// Register a new node in the network
pub async fn register_node(
    payload: RegisterNode,
    fog_net: &Arc<dyn crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<(), ControllerError> {
    trace!("registering new node: {:?}", payload);
    fog_net.register_node(payload).await?;
    Ok(())
}
