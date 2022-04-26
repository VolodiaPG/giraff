use std::sync::Arc;

use okapi::openapi3::Responses;
use rocket::{post, serde::json::Json};
use rocket::{put, Request, response::Responder, State};
use rocket_okapi::{openapi, response::OpenApiResponderInner, util::ensure_status_code_exists};
use tokio::sync::Mutex;

use manager::model::{
    NodeId,
    view::{
        auction::MarketBidProposal,
        node::{PostNode, PostNodeResponse},
        sla::PutSla,
    },
};

use crate::controller;
use crate::service::live_store::{BidDataBase, NodesDataBase};

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

/// Register a SLA and starts the auctionning process, as well as establishing the routing once the auction is completed
#[openapi]
#[put("/function/<leaf_node>", data = "<payload>")]
pub async fn put_function(
    leaf_node: NodeId,
    payload: Json<PutSla>,
    bid_db: &State<Arc<Mutex<BidDataBase>>>,
    nodes_db: &State<Arc<Mutex<NodesDataBase>>>,
) -> Result<Json<MarketBidProposal>> {
    let res = controller::process_function_host(
        leaf_node,
        bid_db.inner().clone(),
        nodes_db.inner().clone(),
        payload.0,
    )
        .await?;
    Ok(Json(res))
}

/// Modify the [NodesDataBase] with the content of the [PostNode]
#[openapi]
#[post("/nodes", data = "<payload>")]
pub async fn post_nodes(
    payload: Json<PostNode>,
    nodes_db: &State<Arc<Mutex<NodesDataBase>>>,
) -> Result<Json<PostNodeResponse>> {
    let res = controller::post_nodes(nodes_db.inner().clone(), payload.0).await?;
    Ok(Json(res))
}
