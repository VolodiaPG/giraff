#![feature(async_closure)]

extern crate core;
#[macro_use]
extern crate tracing;
use helper::pool::Pool;
use helper::prom_metrics::PooledMetrics;

use lazy_static::lazy_static;
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use crate::handler_http::*;
use crate::repeated_tasks::init;
use crate::repository::cron::Cron;
use crate::repository::faas::FaaSBackend;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::k8s::K8s;
use crate::repository::latency_estimation::LatencyEstimation;
use crate::repository::node_query::NodeQuery;
use crate::repository::node_situation::NodeSituation;
use crate::service::auction::Auction;
use crate::service::function::Function;
use crate::service::function_life::FunctionLife;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::service::node_life::NodeLife;
use actix_web::web::Data;
use actix_web::{middleware, web, App, HttpServer};
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use anyhow::Result;
use bytes::Bytes;
use model::dto::node::{NodeSituationData, NodeSituationDisk};
use model::{BidId, FogNodeFaaSPortExternal, FogNodeFaaSPortInternal};
use openfaas::{Configuration, DefaultApiClient};
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry::sdk::propagation::TraceContextPropagator;
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use std::env;
use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter, Registry};
use uom::si::f64::Time;
use uom::si::time::second;

mod controller;
mod handler_http;
mod prom_metrics;
mod repeated_tasks;
mod repository;
mod service;

lazy_static! {
    static ref BUFFER_POOL: Pool<PooledMetrics> = Pool::new(25);
}

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

async fn get_other_metrics(
    function: BidId,
    functions: Arc<FunctionTracking>,
    faas: Arc<FaaSBackend>,
) -> Result<Bytes> {
    let Some(record) = &functions.get_provisioned(&function) else {
        warn!("No records returned for {}", function);
        return Ok(Bytes::default());
    };
    match faas.get_metrics(record).await {
        Ok(Some(metrics)) => Ok(metrics),
        Ok(None) => {
            warn!("No metrics returned for {}", function);
            Ok(Bytes::default())
        }
        Err(err) => {
            warn!("Could not get metrics for function {}: {}", function, err);
            Ok(Bytes::default())
        }
    }
}

// pub async fn metrics(
//     faas: Data<FaaSBackend>,
//     functions: Data<FunctionTracking>,
// ) -> HttpResponse {
//     let faas = faas.into_inner();
//     let functions = functions.into_inner();

//     let stream_local = stream::iter(prometheus::gather())
//         .map(async move |mf| -> Result<Bytes> {
//             let mut buffer = BUFFER_POOL.get();
//             buffer.buffer.clear();
//             buffer.encoder.encode(&[mf.clone()], &mut buffer.buffer)?;
//             let res = Bytes::copy_from_slice(&buffer.buffer);
//             BUFFER_POOL.put(buffer);
//             Ok(res)
//         })
//         .buffer_unordered(25);

//     let stream_functions = stream::iter(functions.get_all_provisioned())
//         .map(move |function| {
//             let functions = functions.clone();
//             let faas = faas.clone();
//             async move {
//                 get_other_metrics(function, functions.clone(), faas.clone())
//                     .await
//             }
//         })
//         .buffer_unordered(25);

//     let stream = select(stream_local, stream_functions);
//     HttpResponse::Ok()
//         .content_type(actix_web::http::header::ContentType::plaintext())
//         .streaming(stream)
// }

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
async fn main() -> anyhow::Result<()> {
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
    let http_client =
        Arc::new(ClientBuilder::new(reqwest::Client::new()).build());

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
    let node_situation = Arc::new(NodeSituation::new(NodeSituationData::new(
        disk_data.unwrap(),
        port_openfaas_external,
    )));

    info!("Current node ID is {}", node_situation.get_my_id());
    info!("Current node has been tagged {:?}", node_situation.get_my_tags());
    let node_query =
        Arc::new(NodeQuery::new(node_situation.clone(), http_client.clone()));
    let function_tracking_repo = Arc::new(FunctionTracking::default());
    let k8s_repo = Arc::new(K8s::new());
    let resource_tracking_repo = Arc::new(
        crate::repository::resource_tracking::ResourceTracking::new(
            k8s_repo.clone(),
            node_situation.get_reserved_cpu(),
            node_situation.get_reserved_memory(),
        )
        .await
        .expect("Failed to instanciate the ResourceTrackingRepo"),
    );
    let latency_estimation_repo = Arc::new(LatencyEstimation::new(
        node_situation.clone(),
        model::domain::exp_average::Alpha::new(0.3).unwrap(),
        model::domain::moving_median::MovingMedianSize::new(50).unwrap(),
    ));
    let cron_repo = Arc::new(
        Cron::new(Time::new::<second>(15.0))
            .await
            .expect("Failed to start Cron repository"),
    );

    // Services
    let auction_service = Arc::new(Auction::new(
        resource_tracking_repo.clone(),
        function_tracking_repo.clone(),
    ));
    let faas_service =
        Arc::new(FaaSBackend::new(client.clone(), node_situation.clone()));
    let node_life_service =
        Arc::new(NodeLife::new(node_situation.clone(), node_query.clone()));
    let neighbor_monitor_service =
        Arc::new(NeighborMonitor::new(latency_estimation_repo));
    let function = Arc::new(Function::new(
        faas_service.clone(),
        node_situation.clone(),
        neighbor_monitor_service.clone(),
        node_query.clone(),
        resource_tracking_repo.clone(),
        function_tracking_repo.clone(),
    ));
    let function_life_service = Arc::new(FunctionLife::new(
        function.clone(),
        auction_service.clone(),
        node_situation.clone(),
        #[cfg(any(
            feature = "auction",
            feature = "edge_first",
            feature = "edge_first_v2",
            feature = "edge_ward_v2",
            feature = "edge_ward_v3",
        ))]
        neighbor_monitor_service.clone(),
        node_query.clone(),
        function_tracking_repo.clone(),
        cron_repo.clone(),
    ));

    if node_situation.is_market() {
        info!("This node is a provider node located at the market node");
    } else {
        info!("This node is a provider node");
    }

    let my_port_http = node_situation.get_my_public_port_http();

    tokio::spawn(register_to_market(
        node_life_service.clone(),
        node_situation.clone(),
    ));

    init(
        cron_repo,
        neighbor_monitor_service.to_owned(),
        k8s_repo,
        node_situation.to_owned(),
        http_client.to_owned(),
    )
    .await
    .expect("Failed to register periodic actions");

    info!("Starting HHTP server on 0.0.0.0:{}", my_port_http);

    let auction_service = Data::from(auction_service);
    let faas_service = Data::from(faas_service);
    let function_life_service = Data::from(function_life_service);
    let node_life_service = Data::from(node_life_service);
    let neighbor_monitor_service = Data::from(neighbor_monitor_service);

    // For metrics gathering
    let function_tracking_repository = Data::from(function_tracking_repo);

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
            // For metrics gathering
            .app_data(Data::clone(&function_tracking_repository))
            // .route("/metrics", web::get().to(metrics))
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
    node_life: Arc<NodeLife>,
    node_situation: Arc<NodeSituation>,
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
        warn!("Failed to register to market: {:?}", err);
        interval.tick().await;
    }
    info!("Registered to market and parent.");
}
