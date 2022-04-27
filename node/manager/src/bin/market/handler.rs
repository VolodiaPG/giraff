use std::sync::Arc;

use okapi::openapi3::Responses;
use rocket::{post, serde::json::Json};
use rocket::{put, response::Responder, Request, State};
use rocket_okapi::{openapi, response::OpenApiResponderInner, util::ensure_status_code_exists};
use tokio::sync::Mutex;

use manager::model::domain::auction::AuctionResult;
use manager::model::{
    view::{
        node::{PostNode, PostNodeResponse},
        sla::PutSla,
    },
    NodeId,
};

use crate::controller;

pub type Result<T = ()> = std::result::Result<T, Error>;

// implements responder for anyhow::error
pub struct Error(pub anyhow::Error);

impl<E> From<E> for Error
where
    E: Into<anyhow::Error>,
{
    fn from(error: E) -> Self {
        Error(error.into())
    }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, request: &Request<'_>) -> rocket::response::Result<'static> {
        rocket::response::Debug(self.0).respond_to(request)
    }
}

impl OpenApiResponderInner for Error {
    fn responses(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<okapi::openapi3::Responses> {
        let mut responses = Responses::default();
        ensure_status_code_exists(&mut responses, 500);
        Ok(responses)
    }
}

/// Register a SLA and starts the auctioning process, as well as establishing the routing once the auction is completed
#[openapi]
#[put("/function/<leaf_node>", data = "<payload>")]
pub async fn put_function(
    leaf_node: NodeId,
    payload: Json<PutSla>,
    auction_service: &State<Arc<dyn crate::service::auction::Auction>>,
) -> Result<Json<AuctionResult>> {
    let res =
        controller::process_function_host(leaf_node, payload.0, auction_service.inner()).await?;
    Ok(Json(res))
}
