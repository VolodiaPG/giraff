#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

use crate::{
    handler::*,
    repository::{
        latency_estimation::LatencyEstimationImpl,
        node_query::{NodeQuery, NodeQueryRESTImpl},
        node_situation::{NodeSituation, NodeSituationHashSetImpl},
        provisioned::ProvisionedHashMapImpl,
        resource_tracking::ResourceTracking,
    },
    service::{
        auction::AuctionImpl,
        faas::OpenFaaSBackend,
        function_life::FunctionLifeImpl,
        neighbor_monitor::NeighborMonitorImpl,
        node_life::{NodeLife, NodeLifeImpl},
        routing::{Router, RouterImpl},
    },
};
use manager::{
    model::dto::node::{NodeSituationData, NodeSituationDisk},
    openfaas::{Configuration, DefaultApiClient},
};
use reqwest::Client;
use rocket::{fairing::AdHoc, launch};
use rocket_okapi::{openapi_get_routes, swagger_ui::*};
use rocket_prometheus::{prometheus::GaugeVec, PrometheusMetrics};
use std::{env, sync::Arc};

mod controller;
mod cron;
mod handler;
mod prom_metrics;
mod repository;
mod service;

/*
ID=1 KUBECONFIG="../../../kubeconfig-master-${ID}" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) ROCKET_PORT="300${ID}" OPENFAAS_PORT="808${ID}" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run --package manager --bin fog_node

Simpler config only using kubeconfig-1
ID=1 KUBECONFIG="../../kubeconfig-master-1" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) ROCKET_PORT="300${ID}" OPENFAAS_PORT="8081" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run --package manager --bin fog_node
*/

/// Load the CONFIG env variable
fn load_config_from_env() -> anyhow::Result<String> {
    let config = env::var("CONFIG")?;
    let config = base64::decode_config(config, base64::STANDARD.decode_allow_trailing_bits(true))?;
    let config = String::from_utf8(config)?;

    Ok(config)
}

cfg_if! {
    if #[cfg(fake_k8s)]{
        use crate::repository::k8s::K8sFakeImpl;
        fn k8s_factory() -> K8sFakeImpl {
            info!("Using Fake k8s impl");
            K8sFakeImpl::new()
        }
    } else{
        use crate::repository::k8s::K8sImpl;
        fn k8s_factory() -> K8sImpl {
            debug!("Using default k8s impl");
            K8sImpl::new()
        }
    }
}

// TODO: Use https://crates.io/crates/rnp instead of a HTTP ping as it is currently the case

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

    let config = load_config_from_env()
        .map_err(|err| {
            error!("Error looking for the based64 CONFIG env variable: {}", err);
            std::process::exit(1);
        })
        .unwrap();
    info!("Loaded config from CONFIG env variable.");

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(Configuration {
        base_path:  format!("http://{}:{}", ip_openfaas, port_openfaas),
        client:     Client::new(),
        basic_auth: auth,
    }));

    let disk_data = NodeSituationDisk::new(config);
    let node_situation =
        Arc::new(NodeSituationHashSetImpl::new(NodeSituationData::from(disk_data.unwrap())));

    info!("Current node ID is {}", node_situation.get_my_id().await);
    info!("Current node has been tagged {:?}", node_situation.get_my_tags().await);
    let node_query = Arc::new(NodeQueryRESTImpl::new(node_situation.clone()));
    let provisioned_repo = Arc::new(ProvisionedHashMapImpl::new());
    let k8s_repo = Arc::new(k8s_factory());
    let resource_tracking_repo = Arc::new(
        crate::repository::resource_tracking::ResourceTrackingImpl::new(k8s_repo.clone())
            .await
            .expect("Failed to instanciate the ResourceTrackingRepo"),
    );
    let auction_repo = Arc::new(crate::repository::auction::AuctionImpl::new());
    let latency_estimation_repo = Arc::new(LatencyEstimationImpl::new(node_situation.clone()));

    // Services
    let auction_service = Arc::new(
        AuctionImpl::new(
            resource_tracking_repo.clone() as Arc<dyn ResourceTracking>,
            auction_repo.clone(),
        )
        .await,
    );
    let faas_service = Arc::new(OpenFaaSBackend::new(client.clone(), provisioned_repo.clone()));
    let router_service = Arc::new(RouterImpl::new(
        Arc::new(crate::repository::faas_routing_table::FaaSRoutingTableHashMap::new()),
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

    let metrics: [&GaugeVec; 11] = [
        &prom_metrics::BID_GAUGE,
        &prom_metrics::MEMORY_USAGE_GAUGE,
        &prom_metrics::MEMORY_ALLOCATABLE_GAUGE,
        &prom_metrics::CPU_USAGE_GAUGE,
        &prom_metrics::CPU_ALLOCATABLE_GAUGE,
        &prom_metrics::MEMORY_USED_GAUGE,
        &prom_metrics::MEMORY_AVAILABLE_GAUGE,
        &prom_metrics::CPU_USED_GAUGE,
        &prom_metrics::CPU_AVAILABLE_GAUGE,
        &prom_metrics::LATENCY_NEIGHBORS_GAUGE,
        &prom_metrics::LATENCY_NEIGHBORS_AVG_GAUGE,
    ];
    for metric in metrics {
        prometheus.registry().register(Box::new(metric.clone())).unwrap();
    }

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
        .attach(AdHoc::on_liftoff("Registration to the parent & market", |_rocket| {
            Box::pin(async {
                info!("Registering to market and parent...");
                // let address = address.clone();
                // trace!("Register using address: {}:{}", address, port);
                register_to_market(node_life_service, node_situation).await;
                info!("Registered to market and parent.");
            })
        }))
        .attach(AdHoc::on_liftoff("Starting CRON jobs", |_rocket| {
            Box::pin(async {
                cron::init(neighbor_monitor_service, k8s_repo);
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
