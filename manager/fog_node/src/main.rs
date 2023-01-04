#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate tracing;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use actix_web::{middleware, web, App, HttpServer};
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_log::LogTracer;

use crate::handler_http::*;
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
use crate::service::function_life::FunctionLife;
use crate::service::neighbor_monitor::{NeighborMonitor, NeighborMonitorImpl};
use crate::service::node_life::{NodeLife, NodeLifeImpl};
use crate::service::routing::{Router, RouterImpl};

use actix_web_opentelemetry::RequestTracing;
use model::dto::node::{NodeSituationData, NodeSituationDisk};
use openfaas::{Configuration, DefaultApiClient};
use prometheus::TextEncoder;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};

mod controller;
mod handler_http;
mod handler_rpc;
mod prom_metrics;
mod repeated_tasks;
mod repository;
mod service;

/*
ID=1 KUBECONFIG="../../../kubeconfig-master-${ID}" OPENFAAS_USERNAME="admin" OPENFAAS_PASSWORD=$(kubectl get secret -n openfaas --kubeconfig=${KUBECONFIG} basic-auth -o jsonpath="{.data.basic-auth-password}" | base64 --decode; echo) r="300${ID}" OPENFAAS_PORT="808${ID}" NODE_SITUATION_PATH="node-situation-${ID}.ron" cargo run --package manager --bin fog_node

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

#[cfg(feature = "edge_first")]
use crate::service::function_life::FunctionLifeBottomUpImpl as FunctionLifeService;
#[cfg(feature = "cloud_only")]
use crate::service::function_life::FunctionLifeCloudOnlyImpl as FunctionLifeService;
#[cfg(not(any(feature = "edge_first", feature = "cloud_only")))]
use crate::service::function_life::FunctionLifeImpl as FunctionLifeService;

fn function_life_factory(
    faas_service: Arc<dyn FaaSBackend>,
    auction_service: Arc<dyn Auction>,
    node_situation: Arc<dyn NodeSituation>,
    neighbor_monitor_service: Arc<dyn NeighborMonitor>,
    node_query: Arc<dyn NodeQuery>,
) -> FunctionLifeService {
    #[cfg(feature = "edge_first")]
    {
        debug!("Using bottom-up placement");
    }
    #[cfg(not(feature = "edge_first"))]
    {
        info!("Using default placement");
    }
    #[cfg(not(feature = "cloud_only"))]
    {
        info!("Using cloud only placement");
    }

    FunctionLifeService::new(
        faas_service,
        auction_service,
        node_situation,
        neighbor_monitor_service,
        node_query,
    )
}

pub async fn metrics() -> actix_web::HttpResponse {
    let encoder = TextEncoder::new();
    let mut buffer: String = "".to_string();
    if encoder.encode_utf8(&prometheus::gather(), &mut buffer).is_err() {
        return actix_web::HttpResponse::InternalServerError()
            .body("Failed to encode prometheus metrics");
    }

    actix_web::HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::plaintext())
        .body(buffer)
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    // Env variable LOG_CONFIG_PATH points at the path where
    // LOG_CONFIG_FILENAME is located
    let log_config_path =
        env::var("LOG_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
    // Env variable LOG_CONFIG_FILENAME names the log file
    let log_config_filename = env::var("LOG_CONFIG_FILENAME")
        .unwrap_or_else(|_| "fog_node.log".to_string());
    let collector_ip =
        env::var("COLLECTOR_IP").unwrap_or_else(|_| "localhost".to_string());
    let collector_port =
        env::var("COLLECTOR_PORT").unwrap_or_else(|_| "14268".to_string());

    let file_appender =
        tracing_appender::rolling::never(log_config_path, log_config_filename);
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    let tracing_leyer = tracing_opentelemetry::OpenTelemetryLayer::new(
        opentelemetry_jaeger::new_collector_pipeline()
            .with_endpoint(format!(
                "http://{}:{}/api/traces",
                collector_ip, collector_port
            ))
            .with_reqwest()
            .with_service_name(name)
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap(),
    );

    Registry::default()
        .with(env_filter)
        .with(fmt::Layer::default().with_writer(non_blocking_file))
        .with(tracing_leyer)
        .with(ForestLayer::default())
}

/// Register a subscriber as global default to process span data.
///
/// It should only be called once!
pub fn init_subscriber(subscriber: impl Subscriber + Send + Sync) {
    LogTracer::init().expect("Failed to set logger");
    set_global_default(subscriber).expect("Failed to set subscriber");
}

// TODO: Use https://crates.io/crates/rnp instead of a HTTP ping as it is currently the case
#[tokio::main]
async fn main() -> std::io::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber(
        env::var("LOG_CONFIG_FILENAME")
            .unwrap_or_else(|_| "fog_node.log".to_string()),
        "trace".into(),
    );
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

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

    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(
        Configuration {
            base_path:  format!("http://{}:{}", ip_openfaas, port_openfaas),
            basic_auth: auth,
        },
        http_client.clone(),
    ));

    let disk_data = NodeSituationDisk::new(config);
    let node_situation = Arc::new(NodeSituationHashSetImpl::new(
        NodeSituationData::from(disk_data.unwrap()),
    ));

    info!("Current node ID is {}", node_situation.get_my_id());
    info!("Current node has been tagged {:?}", node_situation.get_my_tags());
    let node_query = Arc::new(NodeQueryRESTImpl::new(
        node_situation.clone(),
        http_client.clone(),
    ));
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
        Arc::new(crate::repository::routing::RoutingImpl::new(http_client.clone())),
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

    let my_port_http = node_situation.get_my_public_port_http();
    env::set_var("ROCKET_PORT", my_port_http.to_string());

    tokio::spawn(register_to_market(
        node_life_service.clone(),
        node_situation.clone(),
    ));
    tokio::spawn(loop_jobs(
        init(neighbor_monitor_service.clone(), k8s_repo).await,
    ));
    // tokio::spawn(serve_rpc(
    //     node_situation.get_my_public_port_rpc(),
    //     router_service.clone(),
    // ));

    info!("Starting HHTP server on 0.0.0.0:{}", my_port_http);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(TracingLogger::default())
            .wrap(RequestTracing::new())
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(web::Data::new(
                auction_service.clone() as Arc<dyn Auction>
            ))
            .app_data(web::Data::new(
                faas_service.clone() as Arc<dyn FaaSBackend>
            ))
            .app_data(web::Data::new(
                function_life_service.clone() as Arc<dyn FunctionLife>
            ))
            .app_data(
                web::Data::new(router_service.clone() as Arc<dyn Router>),
            )
            .app_data(web::Data::new(
                node_life_service.clone() as Arc<dyn NodeLife>
            ))
            .app_data(web::Data::new(
                neighbor_monitor_service.clone() as Arc<dyn NeighborMonitor>
            ))
            .route("/metrics", web::get().to(metrics))
            .service(
                web::scope("/api")
                    .route("/bid", web::post().to(post_bid))
                    .route("/bid/{id}", web::post().to(post_bid_accept))
                    .route("/routing", web::post().to(post_routing))
                    .route("/sync-routing", web::post().to(post_sync_routing))
                    .route(
                        "/register_route",
                        web::post().to(post_register_route),
                    )
                    .route(
                        "/route_linking",
                        web::post().to(post_route_linking),
                    )
                    .route(
                        "/register",
                        web::post().to(post_register_child_node),
                    )
                    .route("/health", web::head().to(health)),
            )
    })
    .bind(("0.0.0.0", my_port_http.into()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}

async fn register_to_market(
    node_life: Arc<dyn NodeLife>,
    node_situation: Arc<dyn NodeSituation>,
) {
    info!("Registering to market and parent...");
    let mut interval = time::interval(Duration::from_secs(5));
    let my_ip = node_situation.get_my_public_ip();
    let my_port_http = node_situation.get_my_public_port_http();
    let my_port_rpc = node_situation.get_my_public_port_rpc();

    while let Err(err) = node_life
        .init_registration(my_ip, my_port_http.clone(), my_port_rpc.clone())
        .await
    {
        warn!("Failed to register to market: {}", err.to_string());
        interval.tick().await;
    }
    info!("Registered to market and parent.");
}

async fn loop_jobs(jobs: Vec<repeated_tasks::CronFn>) {
    let mut interval = time::interval(Duration::from_secs(15));

    loop {
        for value in jobs.iter() {
            tokio::spawn(value());
        }
        interval.tick().await;
    }
}
