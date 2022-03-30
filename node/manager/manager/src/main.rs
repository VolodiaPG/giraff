#[macro_use]
extern crate log;

mod handlers;
mod live_store;
mod models;
mod openfaas;
mod routing;

use http_api_problem::{HttpApiProblem, StatusCode};
use openfaas::{configuration::BasicAuth, Configuration, DefaultApiClient};
use reqwest::Client;
use serde::de::DeserializeOwned;
use std::{convert::Infallible, env, sync::Arc};
use tokio::sync::RwLock;
use validator::{Validate, ValidationErrors};
use warp::{http::Response, path, Filter, Rejection, Reply};

use crate::{
    live_store::{BidDataBase, ProvisionedDataBase},
    models::{BidId, NodeId},
    routing::{NodeSituation, RoutingTable},
};

/*
ID=1 KUBECONFIG="../../../kubeconfig-master-${ID}" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) PORT="300${ID}" OPENFAAS_PORT="808${ID}" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run
*/

#[tokio::main]
async fn main() {
    std::env::set_var("RUST_LOG", "info, manager=trace, node_logic=trace");
    env_logger::init();

    let debug = env::var("DEBUG").is_ok();
    let port = env::var("PORT")
        .unwrap_or_else(|_| "3000".to_string())
        .parse::<u16>()
        .unwrap_or(3000);
    let port_openfaas = env::var("OPENFAAS_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    debug!("OpenFaaS port: {}", port_openfaas);
    let path_node_situation =
        env::var("NODE_SITUATION_PATH").unwrap_or_else(|_| "node_situation.ron".to_string());
    debug!("Loading node situation from: {}", path_node_situation);

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
        base_path: format!("http://localhost:{}", port_openfaas),
        client: Client::new(),
        basic_auth: auth,
    });

    let db_bid = Arc::new(RwLock::new(BidDataBase::new()));
    let provisioned_db = Arc::new(RwLock::new(ProvisionedDataBase::new()));
    let node_situation = Arc::new(NodeSituation::new(path_node_situation));
    let routing_table = Arc::new(RwLock::new(RoutingTable::default()));

    let path_api_prefix = path!("api" / ..);

    let path_functions = path!("functions");
    let routes = path_api_prefix
        .and(path_functions)
        .and(warp::get())
        .and(with_client(client.clone()))
        .and_then(handlers::list_functions);

    let path_sla = path!("sla");
    let routes = routes.or(path_api_prefix
        .and(path_sla)
        .and(warp::post())
        .and(with_client(client.clone()))
        .and(with_validated_json())
        .and_then(handlers::post_sla));

    let path_bid = path!("bid" / ..);
    let routes = routes.or(path_api_prefix
        .and(path_bid)
        .and(path::end())
        .and(warp::post())
        .and(with_client(client.clone()))
        .and(with_database(db_bid.clone()))
        .and(with_validated_json())
        .and_then(handlers::post_bid));

    let routes = routes.or(path_api_prefix
        .and(path_bid)
        .and(path!(BidId))
        .and(warp::post())
        .and(with_client(client.clone()))
        .and(with_database(db_bid.clone()))
        .and(with_database(provisioned_db.clone()))
        .and_then(handlers::post_bid_accept));

    let routes = routes.or(path_api_prefix
        .and(path!("routing" / BidId / NodeId))
        .and(warp::put())
        .and(with_routing_table(routing_table.clone()))
        .and_then(handlers::put_routing));
            
    let routes = routes.or(path_api_prefix
        .and(path!("routing" / BidId))
        .and(warp::post())
        .and(with_node_situation(node_situation.clone()))
        .and(with_routing_table(routing_table.clone()))
        .and(with_database(provisioned_db.clone()))
        .and(with_client(client.clone()))
        .and(with_raw_body())
        .and_then(handlers::post_forward_routing));

    let routes = routes.recover(handle_rejection);

    let app = warp::serve(routes);

    if debug {
        app.run(([0, 0, 0, 0], port)).await;
    } else {
        app.run(([127, 0, 0, 1], port)).await;
    }
}

fn with_client(
    client: DefaultApiClient,
) -> impl Filter<Extract = (DefaultApiClient,), Error = Infallible> + Clone {
    warp::any().map(move || client.clone())
}

fn with_database<T>(
    db: Arc<RwLock<T>>,
) -> impl Filter<Extract = (Arc<RwLock<T>>,), Error = Infallible> + Clone
where
    T: Send + Sync,
{
    warp::any().map(move || db.clone())
}

fn with_node_situation(
    node_situation: Arc<NodeSituation>,
) -> impl Filter<Extract = (Arc<NodeSituation>,), Error = Infallible> + Clone {
    warp::any().map(move || node_situation.to_owned())
}

fn with_routing_table(
    routing_table: Arc<RwLock<RoutingTable>>,
) -> impl Filter<Extract = (Arc<RwLock<RoutingTable>>,), Error = Infallible> + Clone {
    warp::any().map(move || routing_table.to_owned())
}

fn with_validated_json<T>() -> impl Filter<Extract = (T,), Error = Rejection> + Clone
where
    T: DeserializeOwned + Validate + Send,
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::json())
        .and_then(|value| async move { validate(value).map_err(warp::reject::custom) })
}


fn with_raw_body() -> impl Filter<Extract = (warp::hyper::body::Bytes,), Error = Rejection> + Clone
{
    warp::body::content_length_limit(1024 * 16)
        .and(warp::body::bytes())
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
    NodeLogic(node_logic::error::Error),
    Serialization(serde_json::error::Error),
    BidIdUnvalid(String, Option<uuid::Error>),
    OpenFaas(String),
}

impl warp::reject::Reject for Error {}

impl<T> From<openfaas::Error<T>> for Error where T: std::fmt::Display{
    fn from(error: openfaas::Error<T>) -> Self {
        Error::OpenFaas(error.to_string())
    }
}

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
        Error::NodeLogic(err) => {
            let mut problem =
                HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
                    .title("An error occurred while executing the node logic's code")
                    .detail("Please refer to the error property for additional details");

            problem.set_value("error", &err.to_string());

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
        Error::BidIdUnvalid(id, err) => {
            let mut problem =
                HttpApiProblem::with_title_and_type(StatusCode::NOT_FOUND)
                    .title("An error occurred while looking for a bid id")
                    .detail("Something went wrong processing the id. Please refer to the detail property for additional details");

            problem.set_value("uuid", &id);

            let detail = match err {
                Some(err) => err.to_string(),
                None => "No additional details provided".to_string(),
            };
            problem.set_value("error", &detail);

            problem
        }
        Error::OpenFaas(details) => HttpApiProblem::with_title_and_type(StatusCode::INTERNAL_SERVER_ERROR)
            .title(
                "An error occurred while contacting the OpenFaaS backend through the gateway API",
            )
            .detail(details),
    }
}
