#![feature(future_join)]
#[macro_use]
extern crate log;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use actix_web::{middleware, web, App, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{fmt, EnvFilter};

use std::env;
use std::sync::Arc;

use crate::handler::*;
use crate::repository::fog_node::FogNodeImpl;

mod controller;
mod handler;
mod repository;
mod service;

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
    let file_appender = tracing_appender::rolling::never(
        log_config_path,
        log_config_filename.clone(),
    );
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    let tracing_leyer = tracing_opentelemetry::OpenTelemetryLayer::new(
        opentelemetry_jaeger::new_agent_pipeline()
            .with_service_name(name)
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap(),
    );

    tracing_subscriber::Registry::default()
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    env::set_var("RUST_LOG", "info, market=trace, model=trace");
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("market".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let my_port_http = env::var("SERVER_PORT")
        .expect("Please specify SERVER_PORT env variable");

    let fog_node = Arc::new(FogNodeImpl::new());
    let fog_node_communication =
        Arc::new(crate::repository::node_communication::NodeCommunicationThroughRoutingImpl::new(
            fog_node.clone(),
        ));
    let auction_process =
        Arc::new(crate::repository::auction::SecondPriceAuction::new());

    // Services
    let auction_service = Arc::new(service::auction::AuctionImpl::new(
        auction_process,
        fog_node_communication.clone(),
    ));
    let fog_node_network_service =
        Arc::new(service::fog_node_network::FogNodeNetworkHashTreeImpl::new(
            fog_node.clone(),
        ));
    let faas_service = Arc::new(service::faas::FogNodeFaaSImpl::new(
        fog_node,
        fog_node_network_service.clone(),
        fog_node_communication,
    ));

    info!("Starting HHTP server on 0.0.0.0:{}", my_port_http);

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(TracingLogger::default())
            .wrap(RequestTracing::new())
            .app_data(web::Data::new(faas_service.clone()
                as Arc<dyn crate::service::faas::FogNodeFaaS>))
            .app_data(web::Data::new(auction_service.clone()
                as Arc<dyn crate::service::auction::Auction>))
            .app_data(web::Data::new(fog_node_network_service.clone()
                as Arc<dyn crate::service::fog_node_network::FogNodeNetwork>))
            .service(
                web::scope("/api")
                    .route("/function", web::put().to(put_function))
                    .route("/register", web::post().to(post_register_node))
                    .route("/functions", web::get().to(get_functions))
                    .route("/fog", web::get().to(get_fog))
                    .route("/health", web::head().to(health)),
            )
    })
    .bind(("0.0.0.0", my_port_http.parse().unwrap()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
