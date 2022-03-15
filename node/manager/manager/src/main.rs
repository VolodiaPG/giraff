#[macro_use]
extern crate log;

mod handlers;
mod openfaas;

use http_api_problem::{HttpApiProblem, StatusCode};
use openfaas::{configuration::BasicAuth, Configuration, DefaultApiClient};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::{convert::Infallible, env};
use validator::{Validate, ValidationErrors};
use warp::{http::Response, Filter, Rejection, Reply};
/*
OPENFAAS_USERNAME=admin OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) cargo run
*/

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info, manager=trace, node_logic=trace");
    env_logger::init();

    let debug = !env::var("DEBUG").is_err();
    let username = env::var("OPENFAAS_USERNAME").ok();
    let password = env::var("OPENFAAS_PASSWORD").ok();
    debug!("username: {:?}", username);
    debug!("password?: {:?}", password.is_some());

    let auth: Option<BasicAuth>;
    if let Some(username) = username {
        auth = Some((username, password));
    } else {
        auth = None;
    }

    let client = DefaultApiClient::new(Configuration {
        base_path: "http://localhost:8080".to_owned(),
        client: Client::new(),
        basic_auth: auth,
    });

    let path_api_prefix = warp::path("api");

    let path_functions = warp::path("functions");
    let routes = path_api_prefix
        .and(path_functions)
        .and(warp::get())
        .and(with_client(client.clone()))
        .and_then(handlers::list_functions);

    let path_sla = warp::path("sla");
    let routes = routes.or(path_api_prefix
        .and(path_sla)
        .and(warp::post())
        .and(with_client(client.clone()))
        .and(with_validated_json())
        .and_then(handlers::post_sla));

    let path_bid = warp::path("bid");
    let routes = routes.or(path_api_prefix
        .and(path_bid)
        .and(warp::post())
        .and(with_client(client.clone()))
        .and(with_validated_json())
        .and_then(handlers::post_bid));

    let routes = routes.recover(handle_rejection);

    let app = warp::serve(routes);

    if debug {
        app.run(([0, 0, 0, 0], 3000)).await;
    } else {
        app.run(([127, 0, 0, 1], 3000)).await;
    }
}

fn with_client(
    client: DefaultApiClient,
) -> impl Filter<Extract = (DefaultApiClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
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
    NodeLogicError(node_logic::error::Error),
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
        Error::NodeLogicError(err) => {
            let mut problem =
                HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
                    .title("An error occurred while executing the node logic's code")
                    .detail("Please refer to the detail property for additional details");

            problem.set_value("detail", &err.to_string());

            problem
        }
    }
}
