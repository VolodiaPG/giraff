use crate::controller::ControllerError;
use crate::service::function_life::FunctionLife;
use crate::service::routing::Router;
use crate::{controller, NodeLife};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use model::domain::routing::Packet;
use model::view::auction::BidRequestOwned;
use model::view::node::RegisterNode;
use model::BidId;
use serde::Deserialize;
use std::sync::Arc;

impl actix_web::error::ResponseError for ControllerError {}

/// Return a bid for the SLA.
pub async fn post_bid(
    payload: web::Json<BidRequestOwned>,
    function: Data<Arc<dyn FunctionLife>>,
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
    function: Data<Arc<dyn FunctionLife>>,
) -> Result<HttpResponse, ControllerError> {
    let res =
        controller::auction::provision_from_bid(params.id.clone(), &function)
            .await?;
    Ok(HttpResponse::Ok().json(res))
}

/// Routes the request to the correct URL and node.
pub async fn post_routing(
    packet: web::Json<Packet>,
    router: Data<Arc<dyn Router>>,
) -> Result<HttpResponse, ControllerError> {
    #[cfg(feature = "async_routes")]
    {
        controller::routing::post_async_forward_function_routing(
            packet.0,
            router.get_ref().clone(),
        )
        .await;
        Ok(HttpResponse::Ok().finish())
    }
    #[cfg(not(feature = "async_routes"))]
    {
        post_sync_routing(packet, router).await
    }
}

/// Routes the request to the correct URL and node.
pub async fn post_sync_routing(
    packet: web::Json<Packet>,
    router: Data<Arc<dyn Router>>,
) -> Result<HttpResponse, ControllerError> {
    let res = controller::routing::post_sync_forward_function_routing(
        packet.0,
        router.get_ref().clone(),
    )
    .await?;
    Ok(HttpResponse::Ok().json(res))
}

/// Register a child node to this one
pub async fn post_register_child_node(
    payload: web::Json<RegisterNode>,
    router: Data<Arc<dyn NodeLife>>,
) -> Result<HttpResponse, ControllerError> {
    controller::node::register_child_node(payload.0, &router).await?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn health() -> HttpResponse { HttpResponse::Ok().finish() }
