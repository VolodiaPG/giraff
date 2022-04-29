#[macro_use]
extern crate log;

use std::{env, sync::Arc};

use reqwest::Client;
use rocket::launch;
use rocket_okapi::{openapi_get_routes, swagger_ui::*};

use manager::openfaas::{configuration::BasicAuth, Configuration, DefaultApiClient};

use crate::handler::*;
use crate::repository::k8s::K8sImpl;
use crate::repository::provisioned::ProvisionedHashMapImpl;
use crate::routing::{NodeSituation, NodeSituationDisk};
use crate::service::auction::AuctionImpl;
use crate::service::faas::OpenFaaSBackend;
use crate::service::function_life::FunctionLifeImpl;
use crate::service::routing::RouterImpl;

mod controller;
mod handler;
mod repository;
mod routing;
mod service;

/*
ID=1 KUBECONFIG="../../../kubeconfig-master-${ID}" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) ROCKET_PORT="300${ID}" OPENFAAS_PORT="808${ID}" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run --package manager --bin fog_node

Simpler config only using kubeconfig-1
ID=1 KUBECONFIG="../../kubeconfig-master-1" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) ROCKET_PORT="300${ID}" OPENFAAS_PORT="8081" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run --package manager --bin fog_node
*/

#[launch]
async fn rocket() -> _ {
    std::env::set_var("RUST_LOG", "info, fog_node=trace, node_logic=trace");
    env_logger::init();

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

    // Repositories
    let client = Arc::new(DefaultApiClient::new(Configuration {
        base_path: format!("http://localhost:{}", port_openfaas),
        client: Client::new(),
        basic_auth: auth,
    }));
    let node_situation = Arc::new(NodeSituation::from(NodeSituationDisk::new(
        path_node_situation,
    )));
    let provisioned_repo = Arc::new(ProvisionedHashMapImpl::new());
    let k8s_repo = Arc::new(K8sImpl::new());
    let auction_repo = Arc::new(crate::repository::auction::AuctionImpl::new());

    // Services
    let auction_service = Arc::new(AuctionImpl::new(
        k8s_repo.to_owned(),
        auction_repo.to_owned(),
    ));
    let faas_service = Arc::new(OpenFaaSBackend::new(
        client.to_owned(),
        provisioned_repo.to_owned(),
    ));
    let function_life_service = Arc::new(FunctionLifeImpl::new(
        faas_service.to_owned(),
        auction_service.to_owned(),
    ));
    let router_service = Arc::new(RouterImpl::new(
        Arc::new(crate::repository::faas_routing::FaaSRoutingTableHashMap::new()),
        node_situation.to_owned(),
        Arc::new(crate::repository::routing::RoutingImpl),
        faas_service.to_owned(),
        client.to_owned(),
    ));

    if node_situation.is_market {
        info!("This node is a provider node located at the market node");
    } else {
        info!("This node is a provider node");
    }

    rocket::build()
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .manage(faas_service as Arc<dyn crate::service::faas::FaaSBackend>)
        .manage(function_life_service as Arc<dyn crate::service::function_life::FunctionLife>)
        .manage(router_service as Arc<dyn crate::service::routing::Router>)
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/",
            openapi_get_routes![post_bid, post_bid_accept, post_routing, put_routing],
        )
}
