use bytes::Bytes;
use log::error;
use okapi::openapi3::Responses;
use rocket::{http::Status, response::Responder, serde::json::Json, Request, Response};
use rocket_okapi::{gen::OpenApiGenerator, response::OpenApiResponderInner};
use std::io::Cursor;

/// Shortcut type for the responses of this handler.
pub type Resp<T = ()> = std::result::Result<Json<T>, Error>;

/// Expand the response by mapping the [Result] passed to [Json]
/// and mapping the error channel with `Into::into`.
#[macro_export]
macro_rules! respond {
    ($call:expr) => {
        $call.map(Json).map_err(Into::into)
    };
}

// implements responder for anyhow::error
pub struct Error(pub anyhow::Error);

impl<E> From<E> for Error where E: Into<anyhow::Error>
{
    fn from(error: E) -> Self { Error(error.into()) }
}

impl<'r> Responder<'r, 'static> for Error {
    fn respond_to(self, _request: &Request<'_>) -> rocket::response::Result<'static> {
        let body = self.0.to_string();
        error!("Responder will answer: {}", body);
        Ok(Response::build().header(rocket::http::ContentType::Text)
                            .sized_body(body.len(), Cursor::new(body))
                            .status(Status::InternalServerError)
                            .finalize())
    }
}

impl OpenApiResponderInner for Error {
    fn responses(_gen: &mut rocket_okapi::gen::OpenApiGenerator)
                 -> rocket_okapi::Result<okapi::openapi3::Responses> {
        let mut responses = Responses::default();
        rocket_okapi::util::ensure_status_code_exists(&mut responses, 500);
        Ok(responses)
    }
}

pub struct BytesResponse(pub Bytes);

impl From<Bytes> for BytesResponse {
    fn from(bytes: Bytes) -> Self { Self(bytes) }
}

impl<'r> Responder<'r, 'static> for BytesResponse {
    fn respond_to(self, _request: &Request<'_>) -> rocket::response::Result<'static> {
        let body = self.0;
        Ok(Response::build().header(rocket::http::ContentType::JSON)
                            .sized_body(body.len(), Cursor::new(body))
                            .status(Status::Ok)
                            .finalize())
    }
}

impl OpenApiResponderInner for BytesResponse {
    fn responses(_gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let mut responses = Responses::default();
        rocket_okapi::util::add_content_response(&mut responses,
                                                 200,
                                                 "application/json",
                                                 okapi::openapi3::MediaType::default())?;
        Ok(responses)
    }
}
