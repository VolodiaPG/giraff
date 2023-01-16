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
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry::sdk::propagation::TraceContextPropagator;
use prometheus::{register_histogram_vec, HistogramVec, TextEncoder};
#[cfg(feature = "jaeger")]
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use tokio::sync::RwLock;
use tokio::task::yield_now;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
#[cfg(feature = "jaeger")]
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
use tokio::time::{self};
use uuid::Uuid;

#[cfg(feature = "jaeger")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "jaeger"))]
type HttpClient = reqwest::Client;

type CronFn =
    Box<dyn Fn(u64) -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

type PromTimer = Arc<dashmap::DashMap<Uuid, DateTime<Utc>>>;

lazy_static! {
    static ref HTTP_PRINT_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "iot_emulation_http_request_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, tagged \
         with the content `tag`.",
        &["tag", "period"]
    )
    .unwrap();

    static ref HTTP_TIME_TO_PROCESS_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "iot_emulation_http_request_to_processing_echo_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, time to first process the request by the echo node, tagged \
         with the content `tag`.",
        &["tag", "period"]
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
    tag:    String,
    id:     Uuid,
    period: u64,
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
    pub function_id: Uuid,
    pub iot_url:     String,
    pub node_url:    String,
    pub tag:         String,
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
    let now = chrono::offset::Utc::now();

    yield_now().await; // Yield because other requests need to be timestamped before pursuing

    trace!("{:?}", payload);

    let Some(sent_at) = prom_timers.get(&payload.data.id).map(|x|*x.value()) else {
        warn!("Received a print that was discarded");
        return HttpResponse::BadRequest().finish();
    };

    let elapsed_http = now - sent_at;
    debug!(
        "Elapsed (after being received): {}ms",
        elapsed_http.num_milliseconds()
    );
    let elapsed_function = payload.timestamp - sent_at;

    debug!(
        "Elapsed (start of processing by echo): {}ms",
        elapsed_function.num_milliseconds()
    );

    tokio::spawn(async move {
        HTTP_PRINT_HISTOGRAM
            .with_label_values(&[
                &payload.data.tag,
                &payload.data.period.to_string(),
            ])
            .observe(elapsed_http.num_milliseconds().abs() as f64 / 1000.0);

        HTTP_TIME_TO_PROCESS_HISTOGRAM
            .with_label_values(&[
                &payload.data.tag,
                &payload.data.period.to_string(),
            ])
            .observe(
                elapsed_function.num_milliseconds().abs() as f64 / 1000.0,
            );
    });

    HttpResponse::Ok().finish()
}

async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: Data<Arc<dashmap::DashMap<String, Arc<CronFn>>>>,
    prom_timers: Data<PromTimer>,
    client: Data<Arc<HttpClient>>,
) -> HttpResponse {
    let config = Arc::new(config.0);
    let prom_timers = prom_timers.get_ref().clone();
    let client = client.get_ref().clone();
    let tag = config.tag.clone();
    info!(
        "Created the CRON to send to {:?} on tag {:?}; then directly to {:?}.",
        config.node_url, config.tag, config.iot_url
    );

    let job: CronFn = Box::new(move |period| {
        let prom_timers = prom_timers.clone();
        let config = config.clone();
        let client = client.clone();
        Box::pin(ping(prom_timers, config, client, period))
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
    client: Arc<HttpClient>,
    period: u64,
) {
    let id = Uuid::new_v4();
    let tag = config.tag.clone();
    info!("Sending a ping to {:?}...", tag.clone());

    let res = client.post(&config.node_url).json(&CronPayload {
        address_to_call: config.iot_url.clone(),
        data:            Payload { tag: tag.clone(), id, period },
    });

    prom_timers.insert(id, chrono::offset::Utc::now());
    let res = res.send().await;

    info!("Ping sent to {:?}.", tag);
    if let Err(err) = res {
        warn!(
            "Discarded measure because something went wrong sending a \
             message using config {:?}, error is {:?}",
            config, err
        );
        prom_timers.remove(&id);
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

#[derive(Deserialize)]
pub struct IntervalQuery {
    interval_ms: u64,
}

pub async fn post_interval(
    query: web::Query<IntervalQuery>,
    request_interval: Data<Arc<RwLock<u64>>>,
) -> HttpResponse {
    let mut request_interval = request_interval.write().await;
    *request_interval = query.interval_ms;

    debug!("Request interval set to {}", request_interval);

    HttpResponse::Ok().finish()
}

async fn forever(
    jobs: Arc<dashmap::DashMap<String, Arc<CronFn>>>,
    request_interval: Arc<RwLock<u64>>,
) {
    let mut last_period = 0;

    loop {
        if !jobs.is_empty() {
            let mut send_interval = time::interval(Duration::from_micros(
                last_period * 1000 / (jobs.len() as u64),
            ));
            send_interval.tick().await;

            let jobs_collected: Vec<Arc<CronFn>> =
                jobs.iter().map(|x| x.value().clone()).collect();

            for value in jobs_collected {
                tokio::spawn(async move { value(last_period).await });
                send_interval.tick().await;
            }
        }

        {
            last_period = *request_interval.read().await;
        }
    }
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
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "warn,iot_emulation=trace");

    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("iot_emulation".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    let my_port = std::env::var("SERVER_PORT")
        .expect("Please specfify SERVER_PORT env variable");
    // Id of the request; Histogram that started w/ that request
    let prom_timers = Arc::new(dashmap::DashMap::<Uuid, DateTime<Utc>>::new());

    let jobs = Arc::new(dashmap::DashMap::<String, Arc<CronFn>>::new());

    let request_interval = Arc::new(RwLock::new(5000));

    tokio::spawn(forever(jobs.clone(), request_interval.clone()));

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(reqwest::Client::new());

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(web::JsonConfig::default().limit(4096))
            .app_data(web::Data::new(jobs.clone()))
            .app_data(web::Data::new(prom_timers.clone()))
            .app_data(web::Data::new(http_client.clone()))
            .app_data(web::Data::new(request_interval.clone()))
            .route("/metrics", web::get().to(metrics))
            .service(
                web::scope("/api")
                    .route("/print", web::post().to(print))
                    .route("/cron", web::put().to(put_cron))
                    .route("/interval", web::post().to(post_interval)),
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
