use crate::controller;
use crate::monitoring::{ProvisionedFunctionGauge, RefusedFunctionGauge};
use actix_web::web::{self, Data, Json};
use actix_web::HttpResponse;
use anyhow::Context;
use chrono::Utc;
use helper::log_err;
use helper::monitoring::MetricsExporter;
use model::view::node::RegisterNode;
use model::view::sla::{PutSla, PutSlaRequest};
use model::SlaId;
use serde::Deserialize;
use tracing::error;

#[derive(Debug)]
pub struct AnyhowErrorWrapper {
    err: anyhow::Error,
}

impl std::fmt::Display for AnyhowErrorWrapper {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //error!("{:?}", self.err);
        write!(f, "{:?}", self.err)
    }
}

impl actix_web::error::ResponseError for AnyhowErrorWrapper {}

impl From<anyhow::Error> for AnyhowErrorWrapper {
    fn from(err: anyhow::Error) -> AnyhowErrorWrapper {
        error!("{}", err);
        AnyhowErrorWrapper { err }
    }
}

/// Register a SLA and starts the auctioning process, as well as establishing
/// the routing once the auction is completed
pub async fn put_function(
    payload: Json<PutSlaRequest>,
    auction_service: Data<crate::service::auction::Auction>,
    metrics: Data<MetricsExporter>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let payload: PutSla = payload.0.into();
    let res =
        controller::start_auction(payload.clone(), &auction_service).await;
    match res {
        Ok(_) => {
            metrics
                .observe(ProvisionedFunctionGauge {
                    value:         1,
                    sla_id:        payload.sla.id.to_string(),
                    function_name: payload.sla.function_live_name,
                    timestamp:     Utc::now(),
                })
                .await
                .context("Failed to save metrics")?;
        }
        Err(_) => {
            metrics
                .observe(RefusedFunctionGauge {
                    value:         1,
                    sla_id:        payload.sla.id.to_string(),
                    function_name: payload.sla.function_live_name,
                    timestamp:     Utc::now(),
                })
                .await
                .context("Failed to save metrics")?;
        }
    }
    log_err!(res);
    Ok(HttpResponse::Ok().json(res?))
}

#[derive(Debug, Deserialize)]
pub struct PostProvisionParams {
    id: SlaId,
}
/// Provision the previously paid function
pub async fn post_provision_function(
    params: web::Path<PostProvisionParams>,
    auction_service: Data<crate::service::auction::Auction>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res =
        controller::provision_function(params.id.clone(), &auction_service)
            .await;
    log_err!(res);
    Ok(HttpResponse::Ok().finish())
}

/// Register a new node in the network
pub async fn post_register_node(
    payload: Json<RegisterNode>,
    node_net: Data<crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res = controller::register_node(payload.0, &node_net).await;
    log_err!(res);
    let _ = res?;
    Ok(HttpResponse::Ok().finish())
}

/// Get all the successfull transactions (function provisioned) done by the
/// market since its boot.
pub async fn get_functions(
    faas_service: Data<crate::service::faas::FogNodeFaaS>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res = controller::get_functions(&faas_service).await;
    Ok(HttpResponse::Ok().json(res))
}

/// Get all the connected nodes that have registered here
pub async fn get_fog(
    fog_node_network: Data<crate::service::fog_node_network::FogNodeNetwork>,
) -> Result<HttpResponse, AnyhowErrorWrapper> {
    let res = controller::get_fog(&fog_node_network).await?;
    Ok(HttpResponse::Ok().json(res))
}

pub async fn health(metrics: Data<MetricsExporter>) -> HttpResponse {
    if let Err(err) = metrics
        .observe(crate::monitoring::Toto {
            value:     1.0,
            toto:      "health".to_string(),
            timestamp: Utc::now(),
        })
        .await
        .context("Failed to save metrics")
    {
        error!("Failed health: {:?}", err);
        return HttpResponse::InternalServerError().finish();
    }

    HttpResponse::Ok().finish()
}
