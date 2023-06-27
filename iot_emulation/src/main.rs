extern crate core;
#[macro_use]
extern crate tracing;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use anyhow::Context;
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use helper::monitoring::{
    InfluxAddress, InfluxBucket, InfluxOrg, InfluxToken, InstanceName,
    MetricsExporter,
};
use helper::{env_load, env_var};
use helper_derive::influx_observation;
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry::sdk::propagation::TraceContextPropagator;
#[cfg(feature = "jaeger")]
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::task::yield_now;
use tokio::time::{self, sleep};
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::EnvFilter;
use uuid::Uuid;

#[cfg(feature = "jaeger")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "jaeger"))]
type HttpClient = reqwest::Client;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    #[serde(with = "ts_microseconds")]
    sent_at: DateTime<Utc>,
    tag:     String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CronConfig {
    pub function_id:     Uuid,
    pub iot_url:         String,
    pub node_url:        String,
    pub tag:             String,
    pub initial_wait_ms: u64,
    pub interval_ms:     u64,
    pub duration_ms:     u64,
}

/// Counter of number of failed send
#[influx_observation]
struct SendFails {
    #[influxdb(field)]
    n:   u64,
    #[influxdb(tag)]
    tag: String,
}

env_var!(INFLUX_ADDRESS);
env_var!(INFLUX_TOKEN);
env_var!(INFLUX_ORG);
env_var!(INFLUX_BUCKET);
env_var!(INSTANCE_NAME);

#[instrument(
    level = "trace",
    skip(config),
    fields(tag=%config.tag)
)]
async fn put_cron(
    config: Json<CronConfig>,
    cron_jobs: Data<dashmap::DashMap<String, Arc<CronConfig>>>,
    client: Data<HttpClient>,
    metrics: Data<MetricsExporter>,
) -> HttpResponse {
    put_cron_content(config.0, &cron_jobs, &client, &metrics).await;

    HttpResponse::Ok().finish()
}

async fn put_cron_content(
    config: CronConfig,
    cron_jobs: &Arc<dashmap::DashMap<String, Arc<CronConfig>>>,
    client: &Arc<HttpClient>,
    metrics: &Arc<MetricsExporter>,
) -> HttpResponse {
    let config = Arc::new(config);
    let tag = config.tag.clone();
    info!(
        "Created the CRON to send to {:?} on tag {:?}; then directly to {:?}.",
        config.node_url, config.tag, config.iot_url
    );

    cron_jobs.insert(tag.clone(), config);

    tokio::spawn(loop_send_requests(
        tag,
        cron_jobs.clone(),
        client.clone(),
        metrics.clone(),
    ));

    HttpResponse::Ok().finish()
}

// #[instrument(
//     level = "trace",
//     skip(config),
//     fields(tag=%config.tag)
// )]
/// Returns if the function should be removed from cron
async fn ping(
    config: &CronConfig,
    client: &HttpClient,
    metrics: &MetricsExporter,
) -> bool {
    let tag = config.tag.clone();

    let res = client
        .post(&config.node_url)
        .json(&Payload { tag: tag.clone(), sent_at: Utc::now() });

    let res = res.send().await;

    match res {
        Err(err) => {
            warn!(
                "Something went wrong sending a message using config {:?}, \
                 error is {:?}",
                config, err
            );
            if let Err(err) = metrics
                .observe(SendFails { n: 1, tag, timestamp: Utc::now() })
                .await
            {
                warn!("Something went wrong saving the metrics {:?}", err);
            }
        }
        Ok(response) => {
            let code = response.status();
            if code.is_client_error() {
                error!(
                    "Client error on {}: {:?}",
                    config.tag,
                    code.canonical_reason()
                );
                return true;
            } else if code.is_server_error() {
                warn!(
                    "Server error on {}: {:?}",
                    config.tag,
                    code.canonical_reason()
                );
                if let Err(err) = metrics
                    .observe(SendFails { n: 1, tag, timestamp: Utc::now() })
                    .await
                {
                    warn!("Something went wrong saving the metrics {:?}", err);
                }
            }
        }
    };
    false
}

async fn loop_send_requests(
    id: String,
    jobs: Arc<dashmap::DashMap<String, Arc<CronConfig>>>,
    client: Arc<HttpClient>,
    metrics: Arc<MetricsExporter>,
) {
    let Some(config) = jobs.get(&id).map(|x| x.value().clone()) else {
        error!("Config was empty at initizalization for function {}", id);
        return;
    };

    let mut send_interval =
        time::interval(Duration::from_millis(config.interval_ms));

    sleep(Duration::from_millis(config.initial_wait_ms)).await;

    let start = SystemTime::now();
    let until = Duration::from_millis(config.duration_ms);
    loop {
        send_interval.tick().await;
        if !jobs.contains_key(&id) {
            info!("Ended loop for {}", id);
            break;
        }

        if ping(&config, &client, &metrics).await {
            info!("Removed cron for function {}", config.tag);
            break;
        };

        yield_now().await;

        let Ok(elapsed) = start.elapsed() else {
            continue;
        };
        if elapsed > until {
            break;
        }
    }

    jobs.remove(&id);
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    _name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
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
                "http://{}:{}/api/traces",
                collector_ip, collector_port
            ))
            .with_reqwest()
            .with_service_name(_name)
            .install_batch(opentelemetry::runtime::Tokio)
            .unwrap(),
    );

    let reg = tracing_subscriber::Registry::default().with(env_filter);

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
    std::env::set_var("RUST_LOG", "warn,iot_emulation=trace");

    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("iot_emulation".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let my_port = std::env::var("SERVER_PORT")
        .expect("Please specfify SERVER_PORT env variable");
    // Id of the request; Histogram that started w/ that request

    let jobs = Arc::new(dashmap::DashMap::<String, Arc<CronConfig>>::new());

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(reqwest::Client::new());

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

    let http_client = web::Data::from(http_client);
    let metrics = web::Data::from(metrics);
    let jobs = web::Data::from(jobs);

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(web::JsonConfig::default().limit(4096))
            .app_data(web::Data::clone(&http_client))
            .app_data(web::Data::clone(&jobs))
            .app_data(web::Data::clone(&metrics))
            .service(
                web::scope("/api").route("/cron", web::put().to(put_cron)),
            )
    })
    .bind(("0.0.0.0", my_port.parse().unwrap()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    #[cfg(feature = "jaeger")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
