extern crate core;
#[macro_use]
extern crate log;

use std::net::IpAddr;
use std::{env, sync::Arc};

use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::launch;
use rocket_okapi::{openapi_get_routes, swagger_ui::*};

use manager::model::dto::node::{NodeSituationData, NodeSituationDisk};
use manager::openfaas::{Configuration, DefaultApiClient};

use crate::handler::*;
use crate::repository::k8s::K8sImpl;
use crate::repository::latency_estimation::LatencyEstimationImpl;
use crate::repository::node_query::{NodeQuery, NodeQueryRESTImpl};
use crate::repository::node_situation::{NodeSituation, NodeSituationHashSetImpl};
use crate::repository::provisioned::ProvisionedHashMapImpl;
use crate::service::auction::AuctionImpl;
use crate::service::faas::OpenFaaSBackend;
use crate::service::function_life::FunctionLifeImpl;
use crate::service::neighbor_monitor::NeighborMonitorImpl;
use crate::service::node_life::{NodeLife, NodeLifeImpl};
use crate::service::routing::{Router, RouterImpl};

mod controller;
mod cron;
mod handler;
mod repository;
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

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(Configuration {
        base_path: format!("http://localhost:{}", port_openfaas),
        client: Client::new(),
        basic_auth: auth,
    }));
    let disk_data = NodeSituationDisk::new(path_node_situation);
    if let Err(e) = disk_data {
        error!("Error loading node situation from disk: {}", e);
        std::process::exit(1);
    }
    let node_situation = Arc::new(NodeSituationHashSetImpl::new(NodeSituationData::from(
        disk_data.unwrap(),
    )));

    info!("Current node ID is {}", node_situation.get_my_id().await);
    let node_query = Arc::new(NodeQueryRESTImpl::new(node_situation.clone()));
    let provisioned_repo = Arc::new(ProvisionedHashMapImpl::new());
    let k8s_repo = Arc::new(K8sImpl::new());
    let auction_repo = Arc::new(crate::repository::auction::AuctionImpl::new());
    let latency_estimation_repo = Arc::new(LatencyEstimationImpl::new(node_situation.clone()));

    // Services
    let auction_service = Arc::new(AuctionImpl::new(
        k8s_repo.to_owned(),
        auction_repo.to_owned(),
    ));
    let faas_service = Arc::new(OpenFaaSBackend::new(
        client.to_owned(),
        provisioned_repo.to_owned(),
    ));
    let router_service = Arc::new(RouterImpl::new(
        Arc::new(crate::repository::faas_routing::FaaSRoutingTableHashMap::new()),
        node_situation.to_owned(),
        Arc::new(crate::repository::routing::RoutingImpl),
        faas_service.to_owned(),
        client.to_owned(),
    ));
    let node_life_service = Arc::new(NodeLifeImpl::new(
        router_service.to_owned(),
        node_situation.to_owned(),
        node_query.to_owned(),
    ));
    let neighbor_monitor_service = Arc::new(NeighborMonitorImpl::new(latency_estimation_repo));
    let function_life_service = Arc::new(FunctionLifeImpl::new(
        faas_service.to_owned(),
        auction_service.to_owned(),
        node_situation.to_owned(),
        neighbor_monitor_service.to_owned(),
        node_query.to_owned(),
    ));

    if node_situation.is_market().await {
        info!("This node is a provider node located at the market node");
    } else {
        info!("This node is a provider node");
    }

    let node_life_service_clone = node_life_service.clone();
    rocket::build()
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .manage(faas_service as Arc<dyn crate::service::faas::FaaSBackend>)
        .manage(function_life_service as Arc<dyn crate::service::function_life::FunctionLife>)
        .manage(router_service as Arc<dyn crate::service::routing::Router>)
        .manage(node_life_service as Arc<dyn crate::service::node_life::NodeLife>)
        .manage(neighbor_monitor_service.clone()
            as Arc<dyn crate::service::neighbor_monitor::NeighborMonitor>)
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount(
            "/api/",
            openapi_get_routes![
                post_bid,
                post_bid_accept,
                post_routing,
                put_routing,
                post_register_child_node,
                post_ping,
            ],
        )
        .attach(AdHoc::on_liftoff(
            "Registration to the parent & market",
            |rocket| {
                Box::pin(async {
                    info!("Registering to market and parent...");
                    let address = rocket.config().address;
                    let port = rocket.config().port;
                    trace!("Register using address: {}:{}", address, port);
                    register_to_market(node_life_service_clone, address, port).await;
                    info!("Registered to market and parent.");
                })
            },
        ))
        .attach(AdHoc::on_liftoff("Starting CRON jobs", |_rocket| {
            Box::pin(async {
                cron::init(neighbor_monitor_service);
                info!("Initialized CRON jobs.");
            })
        }))
}

async fn register_to_market(node_life: Arc<dyn NodeLife>, my_ip: IpAddr, my_port: u16) {
    if let Err(err) = node_life.init_registration(my_ip, my_port).await {
        error!("Failed to register to market: {}", err.to_string());
        std::process::exit(1);
    }
}
