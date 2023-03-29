#[macro_use]
extern crate tracing;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
use model::domain::sla::Sla;
use prometheus::{register_histogram_vec, HistogramVec, TextEncoder};
use serde::Deserialize;
use tracing::Subscriber;

#[cfg(feature = "mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

use std::sync::Arc;

use actix_web::{middleware, web, App, HttpResponse, HttpServer};
#[cfg(feature = "jaeger")]
use actix_web_opentelemetry::RequestTracing;
use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
#[cfg(feature = "jaeger")]
use opentelemetry::global;
#[cfg(feature = "jaeger")]
use opentelemetry::sdk::propagation::TraceContextPropagator;
#[cfg(feature = "jaeger")]
use reqwest_middleware::ClientBuilder;
#[cfg(feature = "jaeger")]
use reqwest_tracing::TracingMiddleware;
use tracing::subscriber::set_global_default;
#[cfg(feature = "jaeger")]
use tracing_actix_web::TracingLogger;
use tracing_forest::ForestLayer;
use tracing_log::LogTracer;
use tracing_subscriber::prelude::__tracing_subscriber_SubscriberExt;
use tracing_subscriber::{EnvFilter, Registry};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingPayload {
    #[serde(with = "ts_microseconds")]
    sent_at: DateTime<Utc>,
    tag:     String,
    period:  u64,
}

async fn handle(
    payload: web::Json<IncomingPayload>,
    histogram: web::Data<Arc<HistogramVec>>,
    sla_id: web::Data<Arc<String>>,
) -> HttpResponse {
    let now = chrono::offset::Utc::now();

    let data = &payload.0;

    let elapsed = now - data.sent_at;

    histogram
        .with_label_values(&[&data.tag, &data.period.to_string(), &sla_id])
        .observe(elapsed.num_milliseconds().abs() as f64 / 1000.0);

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
            .install_batch(opentelemetry::runtime::Tokio)
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

#[tokio::main]
async fn main() -> std::io::Result<()> {
    #[cfg(feature = "jaeger")]
    global::set_text_map_propagator(TraceContextPropagator::new());

    let subscriber = get_subscriber("market".into(), "trace".into());
    init_subscriber(subscriber);

    debug!("Tracing initialized.");

    #[cfg(feature = "jaeger")]
    let http_client = Arc::new(
        ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::default())
            .build(),
    );

    let Ok(sla_raw)= std::env::var("SLA") else {
        panic!("SLA env variable not found");
    };

    let Ok(sla) = serde_json::from_str::<Sla>(&sla_raw) else{
        panic!("Cannot read and deserialize SLA env variable");
    };

    let latency_constraint_sec =
        sla.latency_max.get::<uom::si::time::second>();

    let mut buckets = vec![
        latency_constraint_sec * 0.25,
        latency_constraint_sec * 0.5,
        latency_constraint_sec * 0.75,
        latency_constraint_sec - 3.0 / 1000.0,
        latency_constraint_sec - 2.0 / 1000.0,
        latency_constraint_sec - 1.0 / 1000.0,
        latency_constraint_sec,
        latency_constraint_sec + 1.0 / 1000.0,
        latency_constraint_sec + 2.0 / 1000.0,
        latency_constraint_sec + 3.0 / 1000.0,
        latency_constraint_sec * 1.5,
        latency_constraint_sec * 1.75,
        latency_constraint_sec * 2.0,
    ];

    buckets.sort_by(|a, b| a.partial_cmp(b).unwrap());
    buckets.dedup();

    let histogram = Arc::new(
        register_histogram_vec!(
        "echo_function_http_request_to_processing_echo_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, time to \
         first process the request by the echo node, tagged with the content \
         `tag`.",
        &["tag", "period", "sla_id"],
        buckets
    )
        .unwrap(),
    );

    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(reqwest::Client::new());

    let http_client = web::Data::new(http_client);
    let histogram = web::Data::new(histogram);
    let sla_id = web::Data::new(Arc::new(sla.id.to_string()));

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(web::Data::clone(&http_client))
            .app_data(web::Data::clone(&histogram))
            .app_data(web::Data::clone(&sla_id))
            .route("/metrics", web::get().to(metrics))
            .service(web::scope("/").route("", web::post().to(handle)))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await?;

    // Ensure all spans have been reported
    #[cfg(feature = "jaeger")]
    opentelemetry::global::shutdown_tracer_provider();

    Ok(())
}
