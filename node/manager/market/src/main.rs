#[macro_use]
extern crate log;

mod auction;
mod handlers;
mod live_store;
mod models;
mod tasks;

use crate::live_store::BidDataBase;
use crate::models::NodeId;
use http_api_problem::{HttpApiProblem, StatusCode};
use live_store::NodesDataBase;
use serde::de::DeserializeOwned;
use std::{convert::Infallible, env, sync::Arc};
use tokio::sync::Mutex;
use validator::{Validate, ValidationErrors};
use warp::{http::Response, path, Filter, Rejection, Reply};

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info, market=trace");
    env_logger::init();

    let debug = env::var("DEBUG").is_ok();
    let predefined_clients_path =
        env::var("PREDEFINED_NODES_PATH").unwrap_or_else(|_| "predefined_nodes.ron".to_string());

    let db_bid = Arc::new(Mutex::new(BidDataBase::new()));
    let db_nodes = Arc::new(Mutex::new(NodesDataBase::new(predefined_clients_path)));

    let path_api_prefix = path!("api" / ..);

    let path_node_api_prefix = path_api_prefix.and(path!("node" / ..));
    let path_client_api_prefix = path_api_prefix.and(path!("client" / ..));

    let routes = path_client_api_prefix
        .and(path!("sla"))
        .and(warp::put())
        .and(with_database(db_bid.clone()))
        .and(with_database(db_nodes.clone()))
        .and(with_validated_json())
        .and_then(handlers::clients::put_sla);

    let routes = routes.or(path_node_api_prefix
        .and(path!(NodeId))
        .and(warp::patch())
        .and(with_database(db_nodes.clone()))
        .and(with_validated_json())
        .and_then(handlers::nodes::patch_nodes));

    let routes = routes.or(path_node_api_prefix
        .and(warp::put())
        .and(with_database(db_nodes.clone()))
        .and(with_validated_json())
        .and_then(handlers::nodes::put_nodes));

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
    Serialization(serde_json::error::Error),
    NodeIdNotFound(NodeId),
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
        Error::Serialization(err) => {
            let mut problem =
                HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
                    .title("An error occurred while serializing the response")
                    .detail("Please refer to the error property for additional details");

            problem.set_value("error", &err.to_string());

            problem
        }
        Error::NodeIdNotFound(id) => {
            let mut problem = HttpApiProblem::with_title_and_type(StatusCode::NOT_FOUND)
                .title("Node ID not found")
                .detail("Please refer to the id property to know what node ID was not found");

            problem.set_value("id", &id.to_string());

            problem
        }
    }
}
