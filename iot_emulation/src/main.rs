extern crate core;
#[macro_use]
extern crate tracing;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use actix_web::web::{Data, Json};
use actix_web::{middleware, web, App, HttpResponse, HttpServer};
use actix_web_opentelemetry::RequestTracing;
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use opentelemetry::global;
use opentelemetry::sdk::propagation::TraceContextPropagator;
use prometheus::{
    register_gauge_vec, register_histogram_vec, GaugeVec, HistogramVec,
    TextEncoder,
};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::EnvFilter;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use tokio::time;
use uuid::Uuid;

type CronFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

type PromTimer = Arc<dashmap::DashMap<Uuid, DateTime<Utc>>>;

lazy_static! {
    static ref HTTP_PRINT_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "iot_emulation_http_request_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, tagged \
         with the content `tag`.",
        &["tag"]
    )
    .unwrap();

    static ref HTTP_TIME_TO_PROCESS_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "iot_emulation_http_request_to_processing_echo_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, time to first process the request by the echo node, tagged \
         with the content `tag`.",
        &["tag"]
    )
    .unwrap();

    static ref HTTP_TIME_TO_PROCESS_GAUGE: GaugeVec = register_gauge_vec!(
        "iot_emulation_http_request_to_processing_echo_duration_seconds_print_gauge",
        "The HTTP request latencies in seconds for the /print route, time to first process the request by the echo node, tagged \
         with the content `tag`.",
        &["tag"]
    )
    .unwrap();
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Packet<'a> {
    #[serde(rename = "faasFunction")]
    FaaSFunction {
        to:   Uuid,
        #[serde(borrow)]
        data: &'a RawValue,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    tag: String,
    id:  Uuid,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResponseFromEcho {
    #[serde(with = "ts_microseconds")]
    timestamp: DateTime<Utc>,
    data:      Payload,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CronPayload {
    address_to_call: String,
    data:            Payload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaaSPacket {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCron {
    pub iot_url:        String,
    pub first_node_url: String,
    pub function_id:    Uuid,
    pub tag:            String,
}

#[instrument(
    level = "trace",
    skip(prom_timers),
    fields(tag=%payload.data.tag)
)]
pub async fn print(
    payload: Json<ResponseFromEcho>,
    prom_timers: Data<PromTimer>,
) -> HttpResponse {
    let Some(start) = prom_timers.get(&payload.data.id).map(|x|*x.value()) else {
        warn!("Received a print that was discarded");
        return HttpResponse::BadRequest().finish();
    };

    info!("{:?}", payload);
    let now = chrono::offset::Utc::now();

    let elapsed_http = now - start;
    debug!(
        "Elapsed (after being received): {}ms",
        elapsed_http.num_milliseconds()
    );
    let elapsed_function = now - payload.timestamp;

    debug!(
        "Elapsed (start of processing by echo): {}ms",
        elapsed_function.num_milliseconds()
    );

    info!("Measured {:?} ({:?})", payload.data.id, payload.data.tag);

    tokio::spawn(async move {
        HTTP_PRINT_HISTOGRAM
            .with_label_values(&[&payload.data.tag])
            .observe(elapsed_http.num_milliseconds().abs() as f64 / 100.0);

        HTTP_TIME_TO_PROCESS_HISTOGRAM
            .with_label_values(&[&payload.data.tag])
            .observe(elapsed_function.num_milliseconds().abs() as f64 / 100.0);

        HTTP_TIME_TO_PROCESS_GAUGE
            .with_label_values(&[&payload.data.tag])
            .set(elapsed_function.num_milliseconds().abs() as f64 / 100.0);
    });

    HttpResponse::Ok().finish()
}

async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: Data<Arc<dashmap::DashMap<String, Arc<CronFn>>>>,
    prom_timers: Data<PromTimer>,
) -> HttpResponse {
    let config = Arc::new(config.0);
    let prom_timers = prom_timers.get_ref().clone();
    let tag = config.tag.clone();
    info!(
        "Created the CRON to send to {:?} on tag {:?}; then directly to {:?}.",
        config.first_node_url, config.tag, config.iot_url
    );

    let job: CronFn = Box::new(move || {
        let prom_timers = prom_timers.clone();
        let config = config.clone();
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build();
        Box::pin(ping(prom_timers, config, client))
    });

    cron_jobs.insert(tag, Arc::new(job));

    HttpResponse::Ok().finish()
}

#[instrument(
    level = "trace",
    skip(prom_timers, config),
    fields(tag=%config.tag)
)]
async fn ping(
    prom_timers: PromTimer,
    config: Arc<StartCron>,
    client: ClientWithMiddleware,
) {
    let id = Uuid::new_v4();
    let tag = config.tag.clone();
    info!("Sending a ping to {:?}...", tag.clone());

    let res = client.post(config.first_node_url.clone()).json(
        &Packet::FaaSFunction {
            to:   config.function_id.clone(),
            data: &serde_json::value::to_raw_value(&CronPayload {
                address_to_call: config.iot_url.clone(),
                data:            Payload { tag: tag.clone(), id: id.clone() },
            })
            .unwrap(),
        },
    );

    prom_timers.insert(id, chrono::offset::Utc::now());
    let res = res.send();

    info!("Ping sent to {:?}.", tag);
    if let Err(err) = res.await {
        warn!(
            "Discarded measure because something went wrong sending a \
             message using config {:?}, error is {:?}",
            config, err
        );
        prom_timers.remove(&id);
        return;
    }
}

pub async fn metrics() -> HttpResponse {
    let encoder = TextEncoder::new();
    let mut buffer: String = "".to_string();
    if encoder.encode_utf8(&prometheus::gather(), &mut buffer).is_err() {
        return HttpResponse::InternalServerError()
            .body("Failed to encode prometheus metrics");
    }

    HttpResponse::Ok()
        .insert_header(actix_web::http::header::ContentType::plaintext())
        .body(buffer)
}

async fn forever(jobs: Arc<dashmap::DashMap<String, Arc<CronFn>>>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        for value in jobs.iter() {
            let val = value.value().clone();
            tokio::spawn(async move { val().await });
        }
    }
}

/// Compose multiple layers into a `tracing`'s subscriber.
pub fn get_subscriber(
    name: String,
    env_filter: String,
) -> impl Subscriber + Send + Sync {
    let collector_ip = std::env::var("COLLECTOR_IP")
        .unwrap_or_else(|_| "localhost".to_string());
    let collector_port = std::env::var("COLLECTOR_PORT")
        .unwrap_or_else(|_| "14268".to_string());

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

    tracing_subscriber::Registry::default()
        .with(env_filter)
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
    std::env::set_var("RUST_LOG", "warn,iot_emulation=trace");
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("iot_emulation".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let my_port = std::env::var("SERVER_PORT")
        .expect("Please specfify SERVER_PORT env variable");
    // Id of the request; Histogram that started w/ that request
    let prom_timers = Arc::new(dashmap::DashMap::<Uuid, DateTime<Utc>>::new());

    let jobs = Arc::new(dashmap::DashMap::<String, Arc<CronFn>>::new());

    tokio::spawn(forever(jobs.clone()));

    HttpServer::new(move || {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(TracingLogger::default())
            .wrap(RequestTracing::new())
            .app_data(web::JsonConfig::default().limit(4096))
            .app_data(web::Data::new(jobs.clone()))
            .app_data(web::Data::new(prom_timers.clone()))
            .route("/metrics", web::get().to(metrics))
            .service(
                web::scope("/api")
                    .route("/print", web::post().to(print))
                    .route("/cron", web::put().to(put_cron)),
            )
    })
    .bind(("0.0.0.0", my_port.parse().unwrap()))?
    .run()
    .await?;

    // Ensure all spans have been reported
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
