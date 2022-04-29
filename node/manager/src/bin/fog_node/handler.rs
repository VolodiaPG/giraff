use std::sync::Arc;

use manager::helper::handler::Resp;
use rocket::{post, put, serde::json::Json, State};
use rocket_okapi::openapi;

use manager::model::domain::routing::{FunctionRoutingStack, Packet};
use manager::model::domain::sla::Sla;
use manager::model::view::auction::Bid;
use manager::model::BidId;
use manager::respond;

use crate::controller;
use crate::service::function_life::FunctionLife;
use crate::service::routing::Router;

/// Return a bid for the SLA.
#[openapi]
#[post("/bid", data = "<sla>")]
pub async fn post_bid(sla: Json<Sla>, function: &State<Arc<dyn FunctionLife>>) -> Resp<Bid> {
    respond!(controller::auction::bid_on(sla.0, function.inner()).await)
}

/// Second function called after [post_bid] if the bid is accepted and the transaction starts.
/// Will then proceed to provision the SLA and thus, the function.
#[openapi]
#[post("/bid/<id>")]
pub async fn post_bid_accept(id: BidId, function: &State<Arc<dyn FunctionLife>>) -> Resp {
    respond!(controller::auction::provision_from_bid(id, function.inner()).await)
}

/// Routes the request to the correct URL and node.
#[openapi]
#[post("/routing", data = "<packet>")]
pub async fn post_routing(packet: Json<Packet<'_>>, router: &State<Arc<dyn Router>>) -> Resp {
    respond!(controller::routing::post_forward_function_routing(&packet.0, router.inner()).await)
}

/// Register a route.
#[openapi]
#[put("/routing", data = "<stack>")]
pub async fn put_routing(
    router: &State<Arc<dyn Router>>,
    stack: Json<FunctionRoutingStack>,
) -> Resp {
    respond!(controller::routing::register_route(router.inner(), stack.0).await)
}
