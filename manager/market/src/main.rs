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
use actix_web_opentelemetry::RequestTracing;
use anyhow::Context;
use opentelemetry::global;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use reqwest_middleware::ClientBuilder;
use reqwest_tracing::TracingMiddleware;
use std::env::var;
use std::sync::Arc;
use tracing::subscriber::set_global_default;
use tracing::{debug, info};
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
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
pub fn init_subscriber(name: String, env_filter: String) {
    // Env variable LOG_CONFIG_PATH points at the path where
    // LOG_CONFIG_FILENAME is located
    let log_config_path =
        var("LOG_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
    // Env variable LOG_CONFIG_FILENAME names the log file
    let log_config_filename = var("LOG_CONFIG_FILENAME")
        .unwrap_or_else(|_| "market.log".to_string());

    let file_appender =
        tracing_appender::rolling::never(log_config_path, log_config_filename);
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    let reg = Registry::default()
        .with(env_filter)
        .with(fmt::Layer::default().with_writer(non_blocking_file));

    let collector_ip = std::env::var("COLLECTOR_IP")
        .unwrap_or_else(|_| "localhost".to_string());
    let collector_port = std::env::var("COLLECTOR_PORT")
        .unwrap_or_else(|_| "14268".to_string());

    // let provider = TracerProvider::builder()
    //     .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
    //     .build();
    //let tracer = provider.tracer(name.clone());
    let tracer = opentelemetry_jaeger::new_collector_pipeline()
        .with_endpoint(format!(
            "http://{collector_ip}:{collector_port}/api/traces"
        ))
        .with_reqwest()
        .with_service_name(name)
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracing_layer = tracing_opentelemetry::OpenTelemetryLayer::new(tracer);
    //  let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    //let reg = reg.with(telemetry);
    if std::env::var("ENABLE_COLLECTOR").unwrap_or("".to_string())
        == "true".to_string()
    {
        let reg = reg.with(tracing_layer);
        let reg = reg.with(ForestLayer::default());

        set_global_default(reg).expect("Failed to set subscriber");
    } else {
        let reg = reg.with(ForestLayer::default());

        set_global_default(reg).expect("Failed to set subscriber");
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    init_subscriber("marketplace".into(), "info".into());

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

    let mut client_builder = ClientBuilder::new(
        reqwest::Client::builder()
            .pool_idle_timeout(Some(std::time::Duration::from_secs(90)))
            .build()?, // keep-alive
    );
    client_builder = client_builder.with(TracingMiddleware::default());
    let http_client = Arc::new(client_builder.build());

    //#[cfg(not(feature = "jaeger"))]
    //let http_client =
    //    Arc::new(ClientBuilder::new(reqwest::Client::new()).build());

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
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
