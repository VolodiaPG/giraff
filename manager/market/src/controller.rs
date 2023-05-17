use anyhow::{Context, Result};
use model::view::auction::AcceptedBid;
use model::view::node::{GetFogNodes, RegisterNode};
use model::view::sla::PutSla;
use model::NodeId;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::trace;

/// Register a SLA and starts the auctioning process, can take a while.
// TODO define "a while"; set a timeout
pub async fn start_auction(
    payload: PutSla,
    auction_service: &Arc<crate::service::auction::Auction>,
) -> Result<AcceptedBid> {
    trace!("put sla: {:?}", payload);

    auction_service
        .start_auction(payload.target_node, payload.sla)
        .await
        .context("Failed the auctioning and provisionning process")
}

/// Register a new node in the network
pub async fn register_node(
    payload: RegisterNode,
    fog_net: &Arc<crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<()> {
    trace!("registering new node: {:?}", payload);
    fog_net.register_node(payload).await.context("Failed to register node")
}

/// Get all the provisioned functions from the database
pub async fn get_functions(
    faas_service: &Arc<crate::service::faas::FogNodeFaaS>,
) -> HashMap<NodeId, Vec<AcceptedBid>> {
    trace!("getting functions");
    faas_service.get_functions().await
}

/// Get all the connected nodes that have registered here
pub async fn get_fog(
    fog_node_network: &Arc<crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<Vec<GetFogNodes>> {
    Ok(fog_node_network
        .get_nodes()
        .await
        .into_iter()
        .map(|val| val.into())
        .collect())
}
