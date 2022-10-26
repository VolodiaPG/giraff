use crate::service::function_life::FunctionLife;
use crate::service::routing::Router;
use crate::{controller, NodeLife};
use helper::handler::{BytesResponse, Resp};
use helper::respond;
use model::domain::routing::Packet;
use model::view::auction::{BidProposals, BidRequestOwned};
use model::view::node::RegisterNode;
use model::view::routing::{Route, RouteLinking};
use model::BidId;
use rocket::serde::json::Json;
use rocket::{head, post, State};
use rocket_okapi::openapi;
use std::sync::Arc;

/// Return a bid for the SLA.
#[openapi]
#[post("/bid", data = "<payload>")]
pub async fn post_bid(
    payload: Json<BidRequestOwned>,
    function: &State<Arc<dyn FunctionLife>>,
) -> Resp<BidProposals> {
    respond!(controller::auction::bid_on(payload.0, function.inner()).await)
}

/// Second function called after [post_bid] if the bid is accepted and the
/// transaction starts. Will then proceed to provision the SLA and thus, the
/// function.
#[openapi]
#[post("/bid/<id>")]
pub async fn post_bid_accept(
    id: BidId,
    function: &State<Arc<dyn FunctionLife>>,
) -> Resp {
    respond!(
        controller::auction::provision_from_bid(id, function.inner()).await
    )
}

/// Routes the request to the correct URL and node.
#[openapi]
#[post("/routing", data = "<packet>")]
pub async fn post_routing(
    packet: Json<Packet<'_>>,
    router: &State<Arc<dyn Router>>,
) -> Result<BytesResponse, helper::handler::Error> {
    Ok(BytesResponse::from(
        controller::routing::post_forward_function_routing(
            &packet.0,
            router.inner(),
        )
        .await?,
    ))
}

/// Register a route.
#[openapi]
#[post("/register_route", data = "<route>")]
pub async fn post_register_route(
    router: &State<Arc<dyn Router>>,
    route: Json<Route>,
) -> Resp {
    respond!(
        controller::routing::register_route(router.inner(), route.0).await
    )
}

#[openapi]
#[post("/route_linking", data = "<linking>")]
pub async fn post_route_linking(
    router: &State<Arc<dyn Router>>,
    linking: Json<RouteLinking>,
) -> Resp {
    respond!(
        controller::routing::route_linking(router.inner(), linking.0).await
    )
}

/// Register a child node to this one
#[openapi]
#[post("/register", data = "<payload>")]
pub async fn post_register_child_node(
    payload: Json<RegisterNode>,
    router: &State<Arc<dyn NodeLife>>,
) -> Resp {
    respond!(
        controller::node::register_child_node(payload.0, router.inner()).await
    )
}

#[openapi]
#[head("/health")]
pub fn health() {}
