use std::collections::HashMap;
use std::sync::Arc;

use rocket::serde::json::Json;
use rocket::{get, post, put, State};
use rocket_okapi::openapi;

use manager::helper::handler::Resp;
use manager::model::view::auction::AcceptedBid;
use manager::model::view::node::RegisterNode;
use manager::model::view::sla::PutSla;
use manager::model::NodeId;
use manager::respond;

use crate::controller;

/// Register a SLA and starts the auctioning process, as well as establishing the routing once the auction is completed
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

#[openapi]
#[get("/health")]
pub async fn health() {}
