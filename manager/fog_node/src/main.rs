#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate tracing;

use actix_web::web::Data;
use bytes::Bytes;
use futures::future::join_all;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use actix_web::{middleware, web, App, HttpServer};

#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use model::{BidId, FogNodeFaaSPortExternal, FogNodeFaaSPortInternal};
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry::sdk::propagation::TraceContextPropagator;
#[cfg(feature = "jaeger")]
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;

use service::function_life::FunctionLife;
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
use crate::service::neighbor_monitor::{NeighborMonitor, NeighborMonitorImpl};
use crate::service::node_life::{NodeLife, NodeLifeImpl};

use model::dto::node::{NodeSituationData, NodeSituationDisk};
use openfaas::{Configuration, DefaultApiClient};
use prometheus::TextEncoder;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;

#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;

use tracing_forest::ForestLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};

mod controller;
mod handler_http;
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

#[cfg(feature = "cloud_only")]
use crate::service::function_life::FunctionLifeCloudOnlyImpl as FunctionLifeService;
#[cfg(feature = "edge_first")]
use crate::service::function_life::FunctionLifeEdgeFirstImpl as FunctionLifeService;
#[cfg(feature = "edge_ward")]
use crate::service::function_life::FunctionLifeEdgeWardImpl as FunctionLifeService;
#[cfg(not(any(
    feature = "edge_first",
    feature = "cloud_only",
    feature = "edge_ward"
)))]
use crate::service::function_life::FunctionLifeImpl as FunctionLifeService;

fn function_life_factory(
    faas_service: Arc<dyn FaaSBackend>,
    auction_service: Arc<dyn Auction>,
    node_situation: Arc<dyn NodeSituation>,
    neighbor_monitor_service: Arc<dyn NeighborMonitor>,
    node_query: Arc<dyn NodeQuery>,
    resource_tracking: Arc<dyn ResourceTracking>,
) -> FunctionLifeService {
    #[cfg(feature = "edge_first")]
    {
        info!("Using bottom-up placement");
    }
    #[cfg(feature = "cloud_only")]
    {
        info!("Using cloud only placement");
    }
    #[cfg(feature = "edge_ward")]
    {
        info!("Using edge ward placement");
    }
    #[cfg(not(any(
        feature = "edge_first",
        feature = "cloud_only",
        feature = "edge_ward"
    )))]
    {
        info!("Using default placement");
    }

    FunctionLifeService::new(
        faas_service,
        auction_service,
        node_situation,
        neighbor_monitor_service,
        node_query,
        resource_tracking,
    )
}

async fn get_other_metrics(function: BidId, faas: &dyn FaaSBackend) -> Bytes {
    match faas.get_metrics(&function).await {
        Ok(Some(metrics)) => metrics,
        Ok(None) => {
            trace!("No metrics returned for {}", function);
            Bytes::default()
        }
        Err(err) => {
            warn!("Could not get metrics for function {}: {}", function, err);
            Bytes::default()
        }
    }
}

pub async fn metrics(faas: Data<dyn FaaSBackend>) -> actix_web::HttpResponse {
    let encoder = TextEncoder::new();
    let mut buffer: String = "".to_string();
    if encoder.encode_utf8(&prometheus::gather(), &mut buffer).is_err() {
        return actix_web::HttpResponse::InternalServerError()
            .body("Failed to encode prometheus metrics");
    }

    let faas = faas.get_ref();

    let others = faas
        .get_provisioned_functions()
        .into_iter()
        .map(|function| get_other_metrics(function, faas));

    let metrics = join_all(others).await.concat();
    let buffer = [buffer.as_bytes(), &metrics].concat();

    actix_web::HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::plaintext())
        .body(buffer)
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    _name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
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

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    #[cfg(feature = "jaeger")]
    let collector_ip =
        env::var("COLLECTOR_IP").unwrap_or_else(|_| "localhost".to_string());
    #[cfg(feature = "jaeger")]
    let collector_port =
        env::var("COLLECTOR_PORT").unwrap_or_else(|_| "14268".to_string());
    #[cfg(feature = "jaeger")]
    let tracing_leyer = tracing_opentelemetry::OpenTelemetryLayer::new(
        opentelemetry_jaeger::new_collector_pipeline()
            .with_endpoint(format!(
                "http://{collector_ip}:{collector_port}/api/traces"
            ))
            .with_reqwest()
            .with_service_name(_name)
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap(),
    );

    let reg = Registry::default()
        .with(env_filter)
        .with(fmt::Layer::default().with_writer(non_blocking_file));

    #[cfg(feature = "jaeger")]
    let reg = reg.with(tracing_leyer);

    reg.with(ForestLayer::default())
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
    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber(
        env::var("LOG_CONFIG_FILENAME")
            .unwrap_or_else(|_| "fog_node.log".to_string()),
        "trace".into(),
    );
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let port_openfaas_internal = env::var("OPENFAAS_PORT_INTERNAL")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()
        .unwrap_or(8080);
    let port_openfaas_external = env::var("OPENFAAS_PORT_EXTERNAL")
        .unwrap_or_else(|_| "31112".to_string())
        .parse::<u16>()
        .unwrap_or(31112);
    let port_openfaas_internal =
        FogNodeFaaSPortInternal::from(port_openfaas_internal);
    let port_openfaas_external =
        FogNodeFaaSPortExternal::from(port_openfaas_external);
    let ip_openfaas =
        env::var("OPENFAAS_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
    debug!(
        "OpenFaaS uri (internal): {}:{} (external port {})",
        ip_openfaas, port_openfaas_internal, port_openfaas_external
    );

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

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(reqwest::Client::new());

    let auth = username.map(|username| (username, password));

    // Repositories
    let client = Arc::new(DefaultApiClient::new(
        Configuration {
            base_path:  format!(
                "http://{ip_openfaas}:{port_openfaas_internal}"
            ),
            basic_auth: auth,
        },
        http_client.clone(),
    ));

    let disk_data = NodeSituationDisk::new(config);
    let node_situation = Arc::new(NodeSituationHashSetImpl::new(
        NodeSituationData::new(disk_data.unwrap(), port_openfaas_external),
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
            node_situation.get_reserved_cpu(),
            node_situation.get_reserved_memory(),
        )
        .await
        .expect("Failed to instanciate the ResourceTrackingRepo"),
    );
    let auction_repo =
        Arc::new(crate::repository::auction::AuctionImpl::new());
    let latency_estimation_repo = Arc::new(LatencyEstimationImpl::new(
        node_situation.clone(),
        http_client.clone(),
    ));

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
    let node_life_service = Arc::new(NodeLifeImpl::new(
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
        resource_tracking_repo.clone(),
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

    info!("Starting HHTP server on 0.0.0.0:{}", my_port_http);

    let auction_service = Data::from(auction_service as Arc<dyn Auction>);
    let faas_service = Data::from(faas_service as Arc<dyn FaaSBackend>);
    let function_life_service =
        Data::from(function_life_service as Arc<dyn FunctionLife>);
    let node_life_service = Data::from(node_life_service as Arc<dyn NodeLife>);
    let neighbor_monitor_service =
        Data::from(neighbor_monitor_service as Arc<dyn NeighborMonitor>);

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(web::JsonConfig::default().limit(4096))
            .app_data(Data::clone(&auction_service))
            .app_data(Data::clone(&faas_service))
            .app_data(Data::clone(&function_life_service))
            .app_data(Data::clone(&node_life_service))
            .app_data(Data::clone(&neighbor_monitor_service))
            .route("/metrics", web::get().to(metrics))
            .service(
                web::scope("/api")
                    .route("/bid", web::post().to(post_bid))
                    .route("/bid/{id}", web::post().to(post_bid_accept))
                    .route(
                        "/register",
                        web::post().to(post_register_child_node),
                    )
                    .route("/health", web::get().to(health)),
            )
    })
    .bind(("0.0.0.0", my_port_http.into()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    #[cfg(feature = "jaeger")]
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
    let my_port_faas = node_situation.get_my_public_port_faas();

    while let Err(err) = node_life
        .init_registration(my_ip, my_port_http.clone(), my_port_faas.clone())
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
