use crate::controller::ControllerError;
use crate::service::function_life::FunctionLife;
use crate::{controller, NodeLife};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use model::view::auction::BidRequestOwned;
use model::view::node::RegisterNode;
use model::BidId;
use serde::Deserialize;

impl actix_web::error::ResponseError for ControllerError {}

/// Return a bid for the SLA.
pub async fn post_bid(
    payload: web::Json<BidRequestOwned>,
    function: Data<FunctionLife>,
) -> Result<HttpResponse, ControllerError> {
    let res = controller::auction::bid_on(payload.0, &function).await?;
    Ok(HttpResponse::Ok().json(res))
}

#[derive(Debug, Deserialize)]
pub struct PostBidAcceptParams {
    id: BidId,
}

/// Second function called after [post_bid] if the bid is accepted and the
/// transaction starts. Will then proceed to provision the SLA and thus, the
/// function.
pub async fn post_bid_accept(
    params: web::Path<PostBidAcceptParams>,
    function: Data<FunctionLife>,
) -> Result<HttpResponse, ControllerError> {
    let res =
        controller::auction::provision_from_bid(params.id.clone(), &function)
            .await?;
    Ok(HttpResponse::Ok().json(res))
}

/// Register a child node to this one
pub async fn post_register_child_node(
    payload: web::Json<RegisterNode>,
    router: Data<NodeLife>,
) -> Result<HttpResponse, ControllerError> {
    controller::node::register_child_node(payload.0, &router).await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn health() -> HttpResponse { HttpResponse::Ok().finish() }
