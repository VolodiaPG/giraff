#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate tracing;

use crate::handler::*;
use crate::repeated_tasks::init;
use crate::repository::latency_estimation::LatencyEstimationImpl;
use crate::repository::node_query::{NodeQuery, NodeQueryRESTImpl};
use crate::repository::node_situation::{
    NodeSituation, NodeSituationHashSetImpl,
};
use crate::repository::provisioned::ProvisionedHashMapImpl;
use crate::repository::resource_tracking::ResourceTracking;
use crate::service::auction::{Auction, AuctionImpl};
use crate::service::faas::{FaaSBackend, OpenFaaSBackend};
use crate::service::neighbor_monitor::{NeighborMonitor, NeighborMonitorImpl};
use crate::service::node_life::{NodeLife, NodeLifeImpl};
use crate::service::routing::{Router, RouterImpl};

use model::dto::node::{NodeSituationData, NodeSituationDisk};
use openfaas::{Configuration, DefaultApiClient};
use rocket_okapi::openapi_get_routes;
use rocket_okapi::swagger_ui::*;
use rocket_prometheus::prometheus::GaugeVec;
use rocket_prometheus::PrometheusMetrics;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time;
use tracing_forest::ForestLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

mod controller;
mod handler;
mod prom_metrics;
mod repeated_tasks;
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
    let config = base64::decode_config(
        config,
        base64::STANDARD.decode_allow_trailing_bits(true),
    )?;
    let config = String::from_utf8(config)?;

    Ok(config)
}

#[cfg(feature = "fake_k8s")]
use crate::repository::k8s::K8sFakeImpl as k8s;
#[cfg(not(feature = "fake_k8s"))]
use crate::repository::k8s::K8sImpl as k8s;
fn k8s_factory() -> k8s {
    #[cfg(feature = "fake_k8s")]
    {
        info!("Using Fake k8s impl");
    }
    #[cfg(not(feature = "fake_k8s"))]
    {
        debug!("Using default k8s impl");
    }
    k8s::new()
}

#[cfg(feature = "bottom_up_placement")]
use crate::service::function_life::FunctionLifeBottomUpImpl as FunctionLifeService;
#[cfg(not(feature = "bottom_up_placement"))]
use crate::service::function_life::FunctionLifeImpl as FunctionLifeService;

fn function_life_factory(
    faas_service: Arc<dyn FaaSBackend>,
    auction_service: Arc<dyn Auction>,
    node_situation: Arc<dyn NodeSituation>,
    neighbor_monitor_service: Arc<dyn NeighborMonitor>,
    node_query: Arc<dyn NodeQuery>,
) -> FunctionLifeService {
    #[cfg(feature = "bottom_up_placement")]
    {
        debug!("Using bottom-up placement");
    }

    #[cfg(not(feature = "bottom_up_placement"))]
    {
        info!("Using default placement");
    }

    FunctionLifeService::new(
        faas_service,
        auction_service,
        node_situation,
        neighbor_monitor_service,
        node_query,
    )
}

// TODO: Use https://crates.io/crates/rnp instead of a HTTP ping as it is currently the case

async fn rocket() {
    let port_openfaas = env::var("OPENFAAS_PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    let ip_openfaas =
        env::var("OPENFAAS_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    debug!("OpenFaaS uri: {}:{}", ip_openfaas, port_openfaas);

    let username = env::var("OPENFAAS_USERNAME").ok();
    let password = env::var("OPENFAAS_PASSWORD").ok();
    debug!("username: {:?}", username);
    debug!("password?: {:?}", password.is_some());

    let config = load_config_from_env()
        .map_err(|err| {
            error!(
                "Error looking for the based64 CONFIG env variable: {}",
                err
            );
            std::process::exit(1);
        })
        .unwrap();
    info!("Loaded config from CONFIG env variable.");

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(Configuration {
        base_path:  format!("http://{}:{}", ip_openfaas, port_openfaas),
        basic_auth: auth,
    }));

    let disk_data = NodeSituationDisk::new(config);
    let node_situation = Arc::new(NodeSituationHashSetImpl::new(
        NodeSituationData::from(disk_data.unwrap()),
    ));

    info!("Current node ID is {}", node_situation.get_my_id());
    info!("Current node has been tagged {:?}", node_situation.get_my_tags());
    let node_query = Arc::new(NodeQueryRESTImpl::new(node_situation.clone()));
    let provisioned_repo = Arc::new(ProvisionedHashMapImpl::new());
    let k8s_repo = Arc::new(k8s_factory());
    let resource_tracking_repo = Arc::new(
        crate::repository::resource_tracking::ResourceTrackingImpl::new(
            k8s_repo.clone(),
        )
        .await
        .expect("Failed to instanciate the ResourceTrackingRepo"),
    );
    let auction_repo =
        Arc::new(crate::repository::auction::AuctionImpl::new());
    let latency_estimation_repo =
        Arc::new(LatencyEstimationImpl::new(node_situation.clone()));

    // Services
    let auction_service = Arc::new(
        AuctionImpl::new(
            resource_tracking_repo.clone() as Arc<dyn ResourceTracking>,
            auction_repo.clone(),
        )
        .await,
    );
    let faas_service = Arc::new(OpenFaaSBackend::new(
        client.clone(),
        provisioned_repo.clone(),
    ));
    let router_service = Arc::new(RouterImpl::new(
        Arc::new(
            crate::repository::faas_routing_table::FaaSRoutingTableHashMap::new(
            ),
        ),
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
    let neighbor_monitor_service =
        Arc::new(NeighborMonitorImpl::new(latency_estimation_repo));
    let function_life_service = Arc::new(function_life_factory(
        faas_service.clone(),
        auction_service.clone(),
        node_situation.clone(),
        neighbor_monitor_service.clone(),
        node_query.clone(),
    ));

    if node_situation.is_market() {
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

    let rocket = rocket::build()
        .attach(prometheus.clone())
        .manage(auction_service as Arc<dyn crate::service::auction::Auction>)
        .manage(faas_service as Arc<dyn crate::service::faas::FaaSBackend>)
        .manage(
            function_life_service
                as Arc<dyn crate::service::function_life::FunctionLife>,
        )
        .manage(router_service as Arc<dyn crate::service::routing::Router>)
        .manage(node_life_service.clone()
            as Arc<dyn crate::service::node_life::NodeLife>)
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
                post_register_route,
                post_route_linking,
                post_register_child_node,
                health
            ],
        )
        .ignite()
        .await
        .expect("Rocket failed to ignite")
        .launch();

    let handle = tokio::spawn(rocket);
    tokio::spawn(register_to_market(node_life_service, node_situation));
    tokio::spawn(loop_jobs(init(neighbor_monitor_service, k8s_repo).await));

    let _ = handle
        .await
        .expect("Cannot join tasks concurrently")
        .expect("Rocket launch failed");
}

async fn register_to_market(
    node_life: Arc<dyn NodeLife>,
    node_situation: Arc<dyn NodeSituation>,
) {
    info!("Registering to market and parent...");
    let mut interval = time::interval(Duration::from_secs(5));
    let my_ip = node_situation.get_my_public_ip();
    let my_port = node_situation.get_my_public_port();

    while let Err(err) = node_life.init_registration(my_ip, my_port).await {
        warn!("Failed to register to market: {}", err.to_string());
        interval.tick().await;
    }
    info!("Registered to market and parent.");
}

async fn loop_jobs(jobs: Arc<RwLock<Vec<repeated_tasks::CronFn>>>) {
    let mut interval = time::interval(Duration::from_secs(60));

    loop {
        for value in jobs.read().await.iter() {
            tokio::spawn(value());
        }
        interval.tick().await;
    }
}

fn main() {
    // Env variable LOG_CONFIG_PATH points at the path where
    // LOG_CONFIG_FILENAME is located
    let log_config_path =
        env::var("LOG_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
    // Env variable LOG_CONFIG_FILENAME names the log file
    let log_config_filename = env::var("LOG_CONFIG_FILENAME")
        .unwrap_or_else(|_| "fog_node.log".to_string());
    let file_appender =
        tracing_appender::rolling::never(log_config_path, log_config_filename);
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(ForestLayer::default())
        .with(fmt::Layer::default().with_writer(non_blocking_file))
        .init();

    debug!("Tracing initialized.");

    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(num_cpus::get())
        .enable_all()
        .build()
        .expect("build runtime failed")
        .block_on(rocket());
}
