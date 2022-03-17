#[macro_use]
extern crate log;

mod handlers;
mod live_store;
mod models;

use http_api_problem::{HttpApiProblem, StatusCode};
use serde::de::DeserializeOwned;
use std::{convert::Infallible, env, sync::Arc};
use tokio::sync::Mutex;
use validator::{Validate, ValidationErrors};
use warp::{http::Response, Filter, Rejection, Reply, path};

use crate::live_store::BidDataBase;
/*
OPENFAAS_USERNAME=admin OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) cargo run
*/

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info, market=trace");
    env_logger::init();

    let debug = !env::var("DEBUG").is_err();

    let db_bid = Arc::new(Mutex::new(BidDataBase::new()));

    let path_api_prefix = path!("api" / ..);

    let path_node_api_prefix = path_api_prefix.and(path!("node" / ..));
    let path_client_api_prefix = path_api_prefix.and(path!("client" / ..));

    let routes = path_client_api_prefix
        .and(path!("sla"))
        .and(warp::put())
        .and(with_validated_json())
        .and_then(handlers::put_sla);

    let routes = routes.recover(handle_rejection);

    let app = warp::serve(routes);

    if debug {
        app.run(([0, 0, 0, 0], 3000)).await;
    } else {
        app.run(([127, 0, 0, 1], 3000)).await;
    }
}

fn with_database<T>(
    db: Arc<Mutex<T>>,
) -> impl Filter<Extract = (Arc<Mutex<T>>,), Error = Infallible> + Clone
where
    T: Send + Sync,
{
    warp::any().map(move || db.clone())
}

fn with_validated_json<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: DeserializeOwned + Validate + Send,
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .and_then(|value| async move { validate(value).map_err(warp::reject::custom) })
}

fn validate<T>(value: T) -> Result<T, Error>
where
    T: Validate,
{
    value.validate().map_err(Error::Validation)?;

    Ok(value)
}

#[derive(Debug)]
enum Error {
    Validation(ValidationErrors),
    SerializationError(serde_json::error::Error),
}

impl warp::reject::Reject for Error {}

async fn handle_rejection(err: Rejection) -> Result<impl Reply, Infallible> {
    trace!("{:?}", err);
    let response = if let Some(e) = err.find::<Error>() {
        handle_crate_error(e)
    } else {
        HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
    };

    Ok(Response::builder()
        .status(response.status.unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .body(serde_json::to_string(&response).unwrap()))
}

fn handle_crate_error(err: &Error) -> HttpApiProblem {
    match err {
        Error::Validation(errors) => {
            let mut problem = HttpApiProblem::with_title_and_type(StatusCode::BAD_REQUEST)
                .title("One or more validation errors occurred")
                .detail("Please refer to the errors property for additional details");

            problem.set_value("errors", errors.errors());

            problem
        }
        Error::SerializationError(err) => {
            let mut problem =
                HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
                    .title("An error occurred while serializing the response")
                    .detail("Please refer to the error property for additional details");

            problem.set_value("error", &err.to_string());

            problem
        }
    }
}
