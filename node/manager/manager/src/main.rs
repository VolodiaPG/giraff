mod handlers;
mod openfaas;

use openfaas::{configuration::BasicAuth, Configuration, DefaultApiClient};
use reqwest::Client;
use std::{convert::Infallible, env};
use warp::Filter;

/*
OPENFAAS_USERNAME=admin OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) cargo run
*/

#[tokio::main]
async fn main() {
    let debug = !env::var("DEBUG").is_err();
    let username = env::var("OPENFAAS_USERNAME").ok();
    let password = env::var("OPENFAAS_PASSWORD").ok();

    let auth: Option<BasicAuth>;
    if let Some(username) = username {
        println!("Using username: {}", username);
        auth = Some((username, password));
    } else {
        println!("No auth");
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
