use std::io::Cursor;
use std::sync::Arc;

use okapi::openapi3::Responses;
use rocket::http::Status;
use rocket::{post, put, response::Responder, serde::json::Json, Request, Response, State};
use rocket_okapi::{openapi, response::OpenApiResponderInner, util::ensure_status_code_exists};
use serde_json::value::Value as JsonValue;

use crate::controller;
use manager::model::domain::routing::RoutingStack;
use manager::model::domain::sla::Sla;
use manager::model::view::auction::Bid;
use manager::model::BidId;

use crate::service::function_life::FunctionLife;
use crate::service::routing::Router;

/// Shortcut type for the responses of this handler.
pub type Resp<T = ()> = std::result::Result<Json<T>, Error>;

/// Expand the response by mapping the [Result] passed to [Json]
/// and mapping the error channel with `Into::into`.
macro_rules! respond {
    ($call: expr) => {
        $call.map(Json).map_err(Into::into)
    };
}

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
    fn respond_to(self, _request: &Request<'_>) -> rocket::response::Result<'static> {
        let body = self.0.to_string();
        trace!("Responder will answer: {}", body);
        Ok(Response::build()
            .header(rocket::http::ContentType::Text)
            .sized_body(body.len(), Cursor::new(body))
            .status(Status::InternalServerError)
            .finalize())
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

#[openapi]
#[post("/routing/<function_id>", format = "json", data = "<payload>")]
pub async fn post_routing(
    function_id: BidId,
    router: &State<Arc<dyn Router>>,
    payload: Json<JsonValue>,
) -> Resp {
    respond!(
        controller::routing::post_forward_function_routing(
            function_id,
            router.inner(),
            serde_json::to_vec(&payload.0).unwrap() // TODO Find a better way to pass the raw data without compromising the format of the body
        )
        .await
    )
}

#[openapi]
#[put("/routing", data = "<stack>")]
pub async fn put_routing(router: &State<Arc<dyn Router>>, stack: Json<RoutingStack>) -> Resp {
    respond!(controller::routing::register_route(router.inner(), stack.0).await)
}
