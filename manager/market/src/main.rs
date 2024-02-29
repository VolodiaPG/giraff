#![feature(async_closure)]
#![feature(future_join)]
use actix_web::web::Data;
use helper::monitoring::{
    InfluxAddress, InfluxBucket, InfluxOrg, InfluxToken, InstanceName,
    MetricsExporter,
};
use helper::{env_load, env_var};
#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;
use crate::handler::*;
use actix_web::{middleware, web, App, HttpServer};
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use anyhow::Context;
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry_sdk::propagation::TraceContextPropagator;
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use std::env::var;
use std::sync::Arc;
use tracing::subscriber::set_global_default;
use tracing::{debug, info, Subscriber};
#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter, Registry};

mod controller;
mod handler;
mod monitoring;
mod repository;
mod service;

env_var!(INFLUX_ADDRESS);
env_var!(INFLUX_TOKEN);
env_var!(INFLUX_ORG);
env_var!(INFLUX_BUCKET);
env_var!(INSTANCE_NAME);

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    _name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    // Env variable LOG_CONFIG_PATH points at the path where
    // LOG_CONFIG_FILENAME is located
    let log_config_path =
        var("LOG_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
    // Env variable LOG_CONFIG_FILENAME names the log file
    let log_config_filename = var("LOG_CONFIG_FILENAME")
        .unwrap_or_else(|_| "marketplace.log".to_string());

    let file_appender =
        tracing_appender::rolling::never(log_config_path, log_config_filename);
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    #[cfg(feature = "jaeger")]
    let collector_ip = std::env::var("COLLECTOR_IP")
        .unwrap_or_else(|_| "localhost".to_string());
    #[cfg(feature = "jaeger")]
    let collector_port = std::env::var("COLLECTOR_PORT")
        .unwrap_or_else(|_| "14268".to_string());
    #[cfg(feature = "jaeger")]
    let tracing_leyer = tracing_opentelemetry::OpenTelemetryLayer::new(
        opentelemetry_jaeger::new_collector_pipeline()
            .with_endpoint(format!(
                "http://{collector_ip}:{collector_port}/api/traces"
            ))
            .with_reqwest()
            .with_service_name(_name)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("market".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let my_port_http =
        var("SERVER_PORT").expect("Please specify SERVER_PORT env variable");

    let metrics = Arc::new(
        MetricsExporter::new(
            env_load!(InfluxAddress, INFLUX_ADDRESS),
            env_load!(InfluxOrg, INFLUX_ORG),
            env_load!(InfluxToken, INFLUX_TOKEN),
            env_load!(InfluxBucket, INFLUX_BUCKET),
            env_load!(InstanceName, INSTANCE_NAME),
        )
        .await
        .expect("Cannot build the InfluxDB2 database connection"),
    );

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    #[cfg(not(feature = "jaeger"))]
    let http_client =
        Arc::new(ClientBuilder::new(reqwest::Client::new()).build());

    let fog_node = Arc::new(repository::fog_node::FogNode::new());
    let fog_node_network_service = Arc::new(
        service::fog_node_network::FogNodeNetwork::new(fog_node.clone()),
    );

    let fog_node_communication = Arc::new(
        crate::repository::node_communication::NodeCommunication::new(
            fog_node_network_service.clone(),
            http_client,
        ),
    );
    let auction_process = Arc::new(crate::repository::auction::Auction::new());
    let bid_tracking =
        Arc::new(crate::repository::bid_tracking::BidTracking::new());

    // Services
    let faas_service = Arc::new(service::faas::FogNodeFaaS::new(
        fog_node.clone(),
        fog_node_communication.clone(),
    ));
    let auction_service = Arc::new(service::auction::Auction::new(
        auction_process,
        fog_node_communication,
        fog_node_network_service.clone(),
        faas_service.clone(),
        metrics.clone(),
        bid_tracking.clone(),
    ));

    info!("Starting HHTP server on 0.0.0.0:{}", my_port_http);

    let fog_node_network_service = Data::from(fog_node_network_service);
    let auction_service = Data::from(auction_service);
    let faas_service = Data::from(faas_service);
    let metrics = Data::from(metrics);

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(Data::clone(&faas_service))
            .app_data(Data::clone(&auction_service))
            .app_data(Data::clone(&fog_node_network_service))
            .app_data(Data::clone(&metrics))
            .service(
                web::scope("/api")
                    .route("/function", web::put().to(put_function))
                    .route(
                        "/function/{id}",
                        web::post().to(post_provision_function),
                    )
                    .route("/register", web::post().to(post_register_node))
                    .route("/functions", web::get().to(get_functions))
                    .route("/fog", web::get().to(get_fog))
                    .route("/health", web::get().to(health)),
            )
    })
    .bind(("0.0.0.0", my_port_http.parse().unwrap()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    #[cfg(feature = "jaeger")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
