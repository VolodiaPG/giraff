use anyhow::{anyhow, Context, Result};
use model::domain::auction::AuctionResult;
use model::dto::node::NodeRecord;
use model::view::auction::{AcceptedBid, InstanciatedBid};
use model::view::node::{GetFogNodes, RegisterNode};
use model::view::sla::PutSla;
use model::NodeId;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::time::Instant;

/// Register a SLA and starts the auctioning process, can take a while.
// TODO define "a while"; set a timeout
pub async fn start_auction(
    payload: PutSla,
    auction_service: &Arc<crate::service::auction::Auction>,
    faas_service: &Arc<crate::service::faas::FogNodeFaaS>,
    fog_node_network: &Arc<crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<AcceptedBid> {
    trace!("put sla: {:?}", payload);

    let started = Instant::now();

    let proposals = auction_service
        .call_for_bids(payload.target_node.clone(), &payload.sla)
        .await
        .with_context(|| {
            format!(
                "Failed to call the network for bids with node {} as the \
                 starting point",
                payload.target_node,
            )
        })?;

    let AuctionResult { chosen_bid } = auction_service
        .do_auction(&proposals)
        .await
        .context("Auction failed")?;

    let NodeRecord { ip, port_faas, .. } = fog_node_network
        .get_node(&chosen_bid.bid.node_id)
        .await
        .ok_or_else(|| {
            anyhow!(
                "Node record of {} is not present in my database",
                chosen_bid.bid.node_id
            )
        })?;

    let accepted = AcceptedBid {
        chosen: InstanciatedBid {
            bid: chosen_bid.bid,
            price: chosen_bid.price,
            ip,
            port: port_faas,
        },
        proposals,
        sla: payload.sla,
    };

    faas_service
        .provision_function(accepted.clone())
        .await
        .context("Failed to provision function")?;

    let finished = Instant::now();

    let duration = finished - started;

    crate::prom_metrics::FUNCTION_DEPLOYMENT_TIME_GAUGE
        .with_label_values(&[
            &accepted.sla.function_live_name,
            &accepted.chosen.bid.id.to_string(),
        ])
        .set(duration.as_millis() as f64 / 1000.0);

    Ok(accepted)
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
