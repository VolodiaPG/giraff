extern crate core;
#[macro_use]
extern crate log;

use std::io::Read;
use std::{env, io, sync::Arc};

use reqwest::Client;
use rocket::fairing::AdHoc;
use rocket::launch;
use rocket_okapi::{openapi_get_routes, swagger_ui::*};
use rocket_prometheus::PrometheusMetrics;

use manager::model::dto::node::{NodeSituationData, NodeSituationDisk};
use manager::openfaas::{Configuration, DefaultApiClient};

use crate::handler::*;
use crate::repository::k8s::{K8sFakeImpl, K8sImpl};
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
    let ip_openfaas = env::var("OPENFAAS_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    debug!("OpenFaaS uri: {}:{}", ip_openfaas, port_openfaas);

    let username = env::var("OPENFAAS_USERNAME").ok();
    let password = env::var("OPENFAAS_PASSWORD").ok();
    debug!("username: {:?}", username);
    debug!("password?: {:?}", password.is_some());

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(Configuration {
        base_path: format!("http://{}:{}", ip_openfaas, port_openfaas),
        client: Client::new(),
        basic_auth: auth,
    }));
    let mut buffer = String::new();
    if let Err(err) = io::stdin().read_to_string(&mut buffer) {
        error!("Error reading stdin: {}", err);
        std::process::exit(1);
    }
    let disk_data = NodeSituationDisk::new(buffer);
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
    let k8s_repo = Arc::new(K8sFakeImpl::new());
    let auction_repo = Arc::new(crate::repository::auction::AuctionImpl::new());
    let latency_estimation_repo = Arc::new(LatencyEstimationImpl::new(node_situation.clone()));

    // Services
    let auction_service = Arc::new(AuctionImpl::new(k8s_repo.clone(), auction_repo.clone()));
    let faas_service = Arc::new(OpenFaaSBackend::new(
        client.clone(),
        provisioned_repo.clone(),
    ));
    let router_service = Arc::new(RouterImpl::new(
        Arc::new(crate::repository::faas_routing::FaaSRoutingTableHashMap::new()),
        node_situation.clone(),
        Arc::new(crate::repository::routing::RoutingImpl),
        faas_service.clone(),
        client.clone(),
    ));
    let node_life_service = Arc::new(NodeLifeImpl::new(
        router_service.clone(),
        node_situation.clone(),
        node_query.clone(),
    ));
    let neighbor_monitor_service = Arc::new(NeighborMonitorImpl::new(latency_estimation_repo));
    let function_life_service = Arc::new(FunctionLifeImpl::new(
        faas_service.clone(),
        auction_service.clone(),
        node_situation.clone(),
        neighbor_monitor_service.clone(),
        node_query.clone(),
    ));

    if node_situation.is_market().await {
        info!("This node is a provider node located at the market node");
    } else {
        info!("This node is a provider node");
    }

    let prometheus = PrometheusMetrics::new();

    rocket::build()
        .attach(prometheus.clone())
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .manage(faas_service as Arc<dyn crate::service::faas::FaaSBackend>)
        .manage(function_life_service as Arc<dyn crate::service::function_life::FunctionLife>)
        .manage(router_service as Arc<dyn crate::service::routing::Router>)
        .manage(node_life_service.clone() as Arc<dyn crate::service::node_life::NodeLife>)
        .manage(neighbor_monitor_service.clone()
            as Arc<dyn crate::service::neighbor_monitor::NeighborMonitor>)
        .mount(
            "/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "/api/openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .mount("/metrics", prometheus)
        .mount(
            "/api/",
            openapi_get_routes![
                post_bid,
                post_bid_accept,
                post_routing,
                put_routing,
                post_register_child_node,
                post_ping,
                health
            ],
        )
        .attach(AdHoc::on_liftoff(
            "Registration to the parent & market",
            |_rocket| {
                Box::pin(async {
                    info!("Registering to market and parent...");
                    // let address = address.clone();
                    // trace!("Register using address: {}:{}", address, port);
                    register_to_market(node_life_service, node_situation).await;
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

async fn register_to_market(node_life: Arc<dyn NodeLife>, node_situation: Arc<dyn NodeSituation>) {
    let my_ip = node_situation.get_my_public_ip().await;
    let my_port = node_situation.get_my_public_port().await;
    if let Err(err) = node_life.init_registration(my_ip, my_port).await {
        error!("Failed to register to market: {}", err.to_string());
        std::process::exit(1);
    }
}
