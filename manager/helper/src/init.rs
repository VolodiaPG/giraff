use async_trait;
use http::{HeaderMap, HeaderName, HeaderValue};
use opentelemetry::global;
use opentelemetry::propagation::TextMapPropagator;
use opentelemetry::trace::{Tracer, TracerProvider, TracerProvider as _};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use std::collections::HashMap;
use std::env::{self, var};
use tracing_forest::ForestLayer;
use tracing_opentelemetry::{OpenTelemetrySpanExt as _, OpenTelemetrySpanExt};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::prelude::*;
use tracing_subscriber::{fmt, EnvFilter};

pub struct TraceThat;
/// Compose multiple layers into a `tracing`'s subscriber.
pub fn init_subscriber(
    name: String,
    env_filter: String,
) -> anyhow::Result<()> {
    //global::set_text_map_propagator(TraceContextPropagator::new());
    let mut layers = Vec::new();
    //global::set_text_map_propagator(TraceContextPropagator::new());
    // Env variable LOG_CONFIG_PATH points at the path where
    // LOG_CONFIG_FILENAME is located
    let log_config_path =
        var("LOG_CONFIG_PATH").unwrap_or_else(|_| "./".to_string());
    // Env variable LOG_CONFIG_FILENAME names the log file
    let log_config_filename = var("LOG_CONFIG_FILENAME")
        .unwrap_or_else(|_| "market.log".to_string());

    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or(EnvFilter::new(env_filter));

    let file_appender =
        tracing_appender::rolling::never(log_config_path, log_config_filename);
    let (non_blocking_file, _guard) =
        tracing_appender::non_blocking(file_appender);

    layers.push(env_filter.boxed());
    layers.push(fmt::Layer::default().with_writer(non_blocking_file).boxed());
    layers.push(ForestLayer::default().boxed());

    if std::env::var("ENABLE_COLLECTOR").unwrap_or("".to_string())
        == "true".to_string()
    {
        let collector_ip = std::env::var("COLLECTOR_IP")
            .unwrap_or_else(|_| "localhost".to_string());
        let collector_port = std::env::var("COLLECTOR_PORT")
            .unwrap_or_else(|_| "4317".to_string());

        //let tracer = opentelemetry_jaeger::new_collector_pipeline()
        //    .with_endpoint(format!(
        //        "http://{collector_ip}:{collector_port}/api/traces"
        //    ))
        //    .with_service_name(name)
        //    .install_batch(opentelemetry_sdk::runtime::Tokio)
        //    .unwrap();

        let pipe = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(format!(
                        "http://{collector_ip}:{collector_port}"
                    ))
                    .with_compression(opentelemetry_otlp::Compression::Gzip),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_resource(Resource::new(vec![
                        opentelemetry::KeyValue::new(
                            SERVICE_NAME,
                            name.clone(),
                        ),
                    ]))
                    .with_sampler(opentelemetry_sdk::trace::Sampler::AlwaysOn),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;

        global::set_tracer_provider(pipe.clone());
        let tracer = pipe.tracer(name);
        //let tracing_layer =
        // tracing_opentelemetry::layer().with_tracer(tracer);
        layers
            .push(tracing_opentelemetry::layer().with_tracer(tracer).boxed());
    };
    tracing_subscriber::registry().with(layers).init();
    Ok(())
}

pub fn stop_subscriber() { opentelemetry::global::shutdown_tracer_provider(); }
