#![feature(async_closure)]

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use actix_web::dev::Service;
use actix_web::{
    middleware, web, App, HttpMessage, HttpRequest, HttpResponse, HttpServer,
};
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use anyhow::{Context, Result};
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use helper::monitoring::{
    InfluxAddress, InfluxBucket, InfluxOrg, InfluxToken, InstanceName,
    MetricsExporter,
};
use helper::{env_load, env_var};
use helper_derive::influx_observation;
use model::domain::sla::Sla;
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry_sdk::propagation::TraceContextPropagator;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tracing::subscriber::set_global_default;
use tracing::{debug, error, info, instrument, warn, Subscriber};
#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

const VAR_SLA: &str = "SLA";
env_var!(INFLUX_ADDRESS);
env_var!(INFLUX_TOKEN);
env_var!(INFLUX_ORG);
env_var!(INFLUX_BUCKET);
env_var!(ID);

/// timestamp at which the function is fully operational
#[influx_observation]
struct FinishedBooting {
    #[influxdb(field)]
    value:  f64,
    #[influxdb(tag)]
    sla_id: String,
}
/// timestamp at which the request is sent to the next node
#[influx_observation]
struct ProcessingTime {
    #[influxdb(field)]
    value:  f64,
    #[influxdb(tag)]
    sla_id: String,
    #[influxdb(tag)]
    to:     String,
}
/// SLA that passed here
#[influx_observation]
struct Latency {
    #[influxdb(field)]
    value:  f64,
    #[influxdb(tag)]
    sla_id: String,
    #[influxdb(tag)]
    tag:    String,
}
/// SLA that passed here
#[influx_observation]
struct LatencyHeader {
    #[influxdb(field)]
    value:  f64,
    #[influxdb(tag)]
    sla_id: String,
    #[influxdb(tag)]
    tag:    String,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct IncomingPayload {
    #[serde(with = "ts_microseconds")]
    sent_at: DateTime<Utc>,
    tag:     String,
    from:    String,
    to:      String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Reconfigure {
    next_function_url: String,
}
struct Configuration {
    next_function_url: Option<String>,
    sla:               Sla,
}

impl Configuration {
    fn new(sla: Sla) -> Self { Self { sla, next_function_url: None } }
}

#[instrument(level = "trace")]
async fn handle() -> HttpResponse { 
    info!("Got it");
    HttpResponse::Ok().finish()
 }

#[instrument(level = "trace", skip(config), fields(next_function_url=payload.0.next_function_url))]
async fn reconfigure(
    payload: web::Json<Reconfigure>,
    config: web::Data<RwLock<Configuration>>,
) -> HttpResponse {
    let data = payload.0;
    info!("Configured next jump url to be {}", data.next_function_url);
    config.write().await.next_function_url = Some(data.next_function_url);
    HttpResponse::Ok().finish()
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    _name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    // Env variable LOG_CONFIG_PATH points at the path where

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(env_filter));

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
                "http://{}:{}/api/traces",
                collector_ip, collector_port
            ))
            .with_reqwest()
            .with_service_name(_name)
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .unwrap(),
    );

    let reg = Registry::default().with(env_filter);

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
async fn main() -> Result<()> {
    let Ok(id) = std::env::var(ID) else {
        panic!("{} env variable not found", ID);
    };

    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber(id, "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let Ok(sla_raw) = std::env::var(VAR_SLA) else {
        panic!("{} env variable not found", VAR_SLA);
    };

    let Ok(sla) = serde_json::from_str::<Sla>(&sla_raw) else {
        panic!("Cannot read and deserialize {} env variable", VAR_SLA);
    };

    let reqwest_client = reqwest::Client::builder()
        .pool_max_idle_per_host(0)
        .pool_idle_timeout(Duration::new(20, 0))
        .build()
        .expect("Failed to build http client");

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest_client)
            .with(TracingMiddleware::default())
            .build(),
    );
    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(ClientBuilder::new(reqwest_client).build());

    let metrics = Arc::new(
        MetricsExporter::new(
            env_load!(InfluxAddress, INFLUX_ADDRESS),
            env_load!(InfluxOrg, INFLUX_ORG),
            env_load!(InfluxToken, INFLUX_TOKEN),
            env_load!(InfluxBucket, INFLUX_BUCKET),
            InstanceName::new(sla.id.to_string())?,
        )
        .await
        .expect("Cannot build the InfluxDB2 database connection"),
    );

    let sla_id_raw = sla.id.clone();
    let metrics_raw = metrics.clone();

    let config =
        web::Data::from(Arc::new(RwLock::new(Configuration::new(sla))));
    let http_client = web::Data::from(http_client);
    let metrics = web::Data::from(metrics);
    let sla_id = web::Data::from(Arc::new(sla_id_raw.to_string()));

    let server = HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::default());

        app.wrap_fn(|req, srv| {
            // Store the instant when the first byte was received in
            // request extensions
            req.extensions_mut().insert(Utc::now());

            // Call the next middleware or handler
            srv.call(req)
        })
        .app_data(web::Data::clone(&config))
        .app_data(web::Data::clone(&http_client))
        .app_data(web::Data::clone(&metrics))
        .app_data(web::Data::clone(&sla_id))
        .service(
            web::scope("")
                .route("/", web::post().to(handle))
                .route("/reconfigure", web::post().to(reconfigure)),
        )
    });

    metrics_raw
        .observe(FinishedBooting {
            value:     0.0,
            sla_id:    sla_id_raw.to_string(),
            timestamp: Utc::now(),
        })
        .await?;

    server.bind(("0.0.0.0", 3000))?.run().await?;

    // Ensure all spans have been reported
    #[cfg(feature = "jaeger")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
