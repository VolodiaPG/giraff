use std::{collections::HashMap, sync::Arc};

use rocket::{get, post, put, serde::json::Json, State};
use rocket_okapi::openapi;

use manager::{
    helper::handler::Resp,
    model::{
        view::{
            auction::AcceptedBid,
            node::{GetFogNodes, RegisterNode},
            sla::PutSla,
        },
        NodeId,
    },
    respond,
};

use crate::controller;

/// Register a SLA and starts the auctioning process, as well as establishing the routing once the
/// auction is completed
#[openapi]
#[put("/function", data = "<payload>")]
pub async fn put_function(
    payload: Json<PutSla>,
    auction_service: &State<Arc<dyn crate::service::auction::Auction>>,
    faas_service: &State<Arc<dyn crate::service::faas::FogNodeFaaS>>,
) -> Resp<AcceptedBid> {
    respond!(
        controller::start_auction(payload.0, auction_service.inner(), faas_service.inner()).await
    )
}

/// Register a new node in the network
#[openapi]
#[post("/register", data = "<payload>")]
pub async fn post_register_node(
    payload: Json<RegisterNode>,
    node_net: &State<Arc<dyn crate::service::fog_node_network::FogNodeNetwork>>,
) -> Resp {
    respond!(controller::register_node(payload.0, node_net.inner()).await)
}

/// Get all the successfull transactions (function provisioned) done by the market since its boot.
#[openapi]
#[get("/functions")]
pub async fn get_functions(
    faas_service: &State<Arc<dyn crate::service::faas::FogNodeFaaS>>,
) -> Resp<HashMap<NodeId, Vec<AcceptedBid>>> {
    respond!(controller::get_functions(faas_service.inner()).await)
}

/// Get all the connected nodes that have registered here
#[openapi]
#[get("/fog")]
pub async fn get_fog(
    fog_node_network: &State<Arc<dyn crate::service::fog_node_network::FogNodeNetwork>>,
) -> Resp<Vec<GetFogNodes>> {
    respond!(controller::get_fog(fog_node_network.inner()).await)
}

#[openapi]
#[get("/health")]
pub async fn health() {}
