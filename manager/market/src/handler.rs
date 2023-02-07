use actix_web::web::{Data, Json};
use actix_web::HttpResponse;
use model::view::node::RegisterNode;
use model::view::sla::PutSla;

use crate::controller::{self, ControllerError};

impl actix_web::error::ResponseError for ControllerError {}

/// Register a SLA and starts the auctioning process, as well as establishing
/// the routing once the auction is completed
pub async fn put_function(
    payload: Json<PutSla>,
    auction_service: Data<dyn crate::service::auction::Auction>,
    faas_service: Data<dyn crate::service::faas::FogNodeFaaS>,
    fog_node_network: Data<
        dyn crate::service::fog_node_network::FogNodeNetwork,
    >,
) -> Result<HttpResponse, ControllerError> {
    let res = controller::start_auction(
        payload.0,
        &auction_service,
        &faas_service,
        &fog_node_network,
    )
    .await?;
    Ok(HttpResponse::Ok().json(res))
}

/// Register a new node in the network
pub async fn post_register_node(
    payload: Json<RegisterNode>,
    node_net: Data<dyn crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<HttpResponse, ControllerError> {
    controller::register_node(payload.0, &node_net).await?;
    Ok(HttpResponse::Ok().finish())
}

/// Get all the successfull transactions (function provisioned) done by the
/// market since its boot.
pub async fn get_functions(
    faas_service: Data<dyn crate::service::faas::FogNodeFaaS>,
) -> Result<HttpResponse, ControllerError> {
    let res = controller::get_functions(&faas_service).await;
    Ok(HttpResponse::Ok().json(res))
}

/// Get all the connected nodes that have registered here
pub async fn get_fog(
    fog_node_network: Data<
        dyn crate::service::fog_node_network::FogNodeNetwork,
    >,
) -> Result<HttpResponse, ControllerError> {
    let res = controller::get_fog(&fog_node_network).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn health() -> HttpResponse { HttpResponse::Ok().finish() }
