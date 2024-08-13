use crate::service::function_life::FunctionLife;
use crate::{controller, NodeLife};
use actix_web::web::{self, Data};
use actix_web::HttpResponse;
use helper::log_err;
use helper::monitoring::MetricsExporter;
use model::view::auction::BidRequestOwned;
use model::view::node::RegisterNode;
use model::SlaId;
use serde::Deserialize;
use tracing::error;

#[derive(Debug)]
pub struct AnyhowErrorWrapper {
    err: anyhow::Error,
}

impl std::fmt::Display for AnyhowErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        error!("{:?}", self.err);
        write!(f, "{:?}", self.err)
    }
}

impl actix_web::error::ResponseError for AnyhowErrorWrapper {}

impl From<anyhow::Error> for AnyhowErrorWrapper {
    fn from(err: anyhow::Error) -> AnyhowErrorWrapper {
        AnyhowErrorWrapper { err }
    }
}

/// Return a bid for the SLA.
pub async fn post_bid(
    payload: web::Json<BidRequestOwned>,
    function: Data<FunctionLife>,
    metrics: Data<MetricsExporter>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res =
        controller::auction::bid_on(payload.0, &function, &metrics).await;
    log_err!(res);
    Ok(HttpResponse::Ok().json(res?))
}

#[derive(Debug, Deserialize)]
pub struct PostBidAcceptParams {
    id: SlaId,
}

/// Second function called after [post_bid] if the bid is accepted and the
/// transaction starts. Will then proceed to reserve the required "space" for
/// the function for the specified duration by the SLA and thus, the function.
pub async fn post_bid_accept(
    params: web::Path<PostBidAcceptParams>,
    function: Data<FunctionLife>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    #[allow(clippy::let_unit_value)]
    let res =
        controller::auction::set_paid_from_sla(params.id.clone(), &function)
            .await;
    log_err!(res);
    Ok(HttpResponse::Ok().json(res?))
}

// Proceeds to provision the paid for SLA ([post_bid_accept]) and thus, the
/// function.
pub async fn post_provision(
    params: web::Path<PostBidAcceptParams>,
    function: Data<FunctionLife>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    #[allow(clippy::let_unit_value)]
    let res =
        controller::auction::provision_from_sla(params.id.clone(), &function)
            .await;
    log_err!(res);
    Ok(HttpResponse::Ok().json(res?))
}

/// Register a child node to this one
pub async fn post_register_child_node(
    payload: web::Json<RegisterNode>,
    router: Data<NodeLife>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res = controller::node::register_child_node(payload.0, &router).await;
    log_err!(res);
    let _ = res?;
    Ok(HttpResponse::Ok().finish())
}

pub async fn health() -> HttpResponse { HttpResponse::Ok().finish() }
