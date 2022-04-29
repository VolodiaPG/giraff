use std::sync::Arc;

use rocket::serde::json::Json;
use rocket::{put, State};
use rocket_okapi::openapi;

use manager::helper::handler::Resp;
use manager::model::{view::sla::PutSla, NodeId};
use manager::respond;

use crate::controller;

/// Register a SLA and starts the auctioning process, as well as establishing the routing once the auction is completed
#[openapi]
#[put("/function/<leaf_node>", data = "<payload>")]
pub async fn put_function(
    leaf_node: NodeId,
    payload: Json<PutSla>,
    auction_service: &State<Arc<dyn crate::service::auction::Auction>>,
) -> Resp {
    respond!(controller::start_auction(leaf_node, payload.0, auction_service.inner()).await)
}
