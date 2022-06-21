use std::collections::HashMap;
use std::convert::Infallible;
use std::sync::Arc;

use anyhow::Result;

use manager::model::domain::auction::AuctionResult;
use manager::model::view::auction::AcceptedBid;
use manager::model::view::node::RegisterNode;
use manager::model::view::sla::PutSla;
use manager::model::NodeId;

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    FogNodeNetwork(#[from] crate::service::fog_node_network::Error),
    #[error(transparent)]
    FaaS(#[from] crate::service::faas::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Register a SLA and starts the auctioning process, can take a while.
// TODO define "a while"; set a timeout
pub async fn start_auction(
    payload: PutSla,
    auction_service: &Arc<dyn crate::service::auction::Auction>,
    faas_service: &Arc<dyn crate::service::faas::FogNodeFaaS>,
) -> Result<AcceptedBid, ControllerError> {
    trace!("put sla: {:?}", payload);

    let proposals = auction_service
        .call_for_bids(payload.target_node, payload.sla)
        .await?;

    let AuctionResult { chosen_bid } = auction_service.do_auction(&proposals).await?;

    let accepted = AcceptedBid {
        chosen: chosen_bid,
        proposals,
    };

    faas_service.provision_function(accepted.clone()).await?;

    Ok(accepted)
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

/// Get all the provisioned functions from the database
pub async fn get_functions(
    faas_service: &Arc<dyn crate::service::faas::FogNodeFaaS>,
) -> Result<HashMap<NodeId, Vec<AcceptedBid>>, Infallible> {
    trace!("getting functions");
    Ok(faas_service.get_functions().await)
}
