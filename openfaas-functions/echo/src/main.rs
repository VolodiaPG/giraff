#[macro_use]
extern crate tracing;

#[cfg(feature = "mimalloc")]
use mimalloc::MiMalloc;
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

lazy_static::lazy_static! {
    static ref HTTP_TIME_TO_PROCESS_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "echo_function_http_request_to_processing_echo_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, time to first process the request by the echo node, tagged \
         with the content `tag`.",
        &["tag", "period"]
    )
    .unwrap();
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct IncomingPayload {
    #[serde(with = "ts_microseconds")]
    sent_at: DateTime<Utc>,
    tag:     String,
    period:  u64,
}

async fn handle(payload: web::Json<IncomingPayload>) -> HttpResponse {
    let now = chrono::offset::Utc::now();

    let data = &payload.0;

    let elapsed = now - data.sent_at;

    HTTP_TIME_TO_PROCESS_HISTOGRAM
        .with_label_values(&[&data.tag, &data.period.to_string()])
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

    #[cfg(not(feature = "jaeger"))]
    let http_client = Arc::new(reqwest::Client::new());

    HttpServer::new(move || {
        let app = App::new().wrap(middleware::Compress::default());

        #[cfg(feature = "jaeger")]
        let app =
            app.wrap(TracingLogger::default()).wrap(RequestTracing::new());

        app.app_data(web::Data::new(http_client.clone()))
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
