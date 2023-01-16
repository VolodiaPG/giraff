use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Result;

use model::domain::auction::AuctionResult;
use model::dto::node::NodeRecord;
use model::view::auction::{AcceptedBid, InstanciatedBid};
use model::view::node::{GetFogNodes, RegisterNode};
use model::view::sla::PutSla;
use model::NodeId;

#[derive(thiserror::Error, Debug)]
pub enum ControllerError {
    #[error("Failed to retrieve the record of the node {0}")]
    RecordOfNodeNotFound(NodeId),
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
    fog_node_network: &Arc<
        dyn crate::service::fog_node_network::FogNodeNetwork,
    >,
) -> Result<AcceptedBid, ControllerError> {
    trace!("put sla: {:?}", payload);

    let proposals = auction_service
        .call_for_bids(payload.target_node, &payload.sla)
        .await?;

    let AuctionResult { chosen_bid } =
        auction_service.do_auction(&proposals).await?;

    let Some(NodeRecord {ip, port_http, ..}) = fog_node_network.get_node(&chosen_bid.bid.node_id).await else {
        return Err(ControllerError::RecordOfNodeNotFound(chosen_bid.bid.node_id));
    };

    let accepted = AcceptedBid {
        chosen: InstanciatedBid {
            bid: chosen_bid.bid,
            price: chosen_bid.price,
            ip,
            port: port_http,
        },
        proposals,
        sla: payload.sla,
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
) -> HashMap<NodeId, Vec<AcceptedBid>> {
    trace!("getting functions");
    faas_service.get_functions().await
}

/// Get all the connected nodes that have registered here
pub async fn get_fog(
    fog_node_network: &Arc<
        dyn crate::service::fog_node_network::FogNodeNetwork,
    >,
) -> Result<Vec<GetFogNodes>> {
    Ok(fog_node_network
        .get_nodes()
        .await
        .into_iter()
        .map(|val| val.into())
        .collect())
}
