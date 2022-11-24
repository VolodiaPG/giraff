extern crate core;
#[macro_use]
extern crate tracing;
extern crate rocket;

use chrono::serde::ts_microseconds;
use chrono::{DateTime, Utc};
use tracing_forest::ForestLayer;
use tracing_subscriber::prelude::*;
use tracing_subscriber::EnvFilter;

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use rocket::serde::json::Json;
use rocket::{delete, post, put, routes, State};
use rocket_prometheus::prometheus::{register_histogram_vec, HistogramVec};
use rocket_prometheus::PrometheusMetrics;
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
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Packet<'a> {
    #[serde(rename = "faasFunction")]
    FaaSFunction {
        to:   Uuid,
        // TODO check wether its better an id or a name
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

#[post("/print", data = "<payload>")]
pub async fn print(
    payload: Json<ResponseFromEcho>,
    prom_timers: &State<PromTimer>,
) {
    let Some(start) = prom_timers.get(&payload.data.id).map(|x|*x.value()) else {
        warn!("Received a print that was discarded");
        return;
    };

    info!("{:?}", payload);
    let now = chrono::offset::Utc::now();
    let elapsed = now - start;
    debug!("Elapsed (after being received): {}ms", elapsed.num_milliseconds());
    HTTP_PRINT_HISTOGRAM
        .with_label_values(&[&payload.data.tag])
        .observe(elapsed.num_seconds().abs() as f64);

    let elapsed = now - payload.timestamp;
    debug!(
        "Elapsed (start of processing by echo): {}ms",
        elapsed.num_milliseconds()
    );
    HTTP_TIME_TO_PROCESS_HISTOGRAM
        .with_label_values(&[&payload.data.tag])
        .observe(elapsed.num_seconds().abs() as f64);
    let elapsed = now - start;
    debug!(
        "Elapsed (after being contacted back): {}ms",
        elapsed.num_milliseconds()
    );

    info!("Measured {:?} ({:?})", payload.data.id, payload.data.tag);
}

#[put("/cron", data = "<config>")]
async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: &State<Arc<dashmap::DashMap<String, Arc<CronFn>>>>,
    prom_timers: &State<PromTimer>,
) {
    let config = Arc::new(config.0);
    let prom_timers = prom_timers.inner().clone();
    let tag = config.tag.clone();
    info!(
        "Created the CRON to send to {:?} on tag {:?}; then directly to {:?}",
        config.first_node_url, config.tag, config.iot_url
    );

    let job: CronFn = Box::new(move || {
        let prom_timers = prom_timers.clone();
        let config = config.clone();
        Box::pin(ping(prom_timers, config))
    });

    cron_jobs.insert(tag, Arc::new(job));
}

#[delete("/cron/<tag>")]
async fn delete_cron(
    tag: String,
    cron_jobs: &State<Arc<dashmap::DashMap<String, Arc<CronFn>>>>,
) {
    info!("Deleting cron on tag {:?}", tag);
    cron_jobs.remove(&tag);
}

#[instrument(
    level = "trace",
    skip(prom_timers, config),
    fields(tag=%config.tag)
)]
#[instrument(level = "trace")]
async fn ping(prom_timers: PromTimer, config: Arc<StartCron>) {
    let id = Uuid::new_v4();
    let tag = config.tag.clone();
    info!("Sending a ping to {:?}...", tag.clone());

    let res = reqwest::Client::new().post(config.first_node_url.clone()).body(
        serde_json::to_string(&Packet::FaaSFunction {
            to:   config.function_id.clone(),
            data: &serde_json::value::to_raw_value(&CronPayload {
                address_to_call: config.iot_url.clone(),
                data:            Payload { tag: tag.clone(), id },
            })
            .unwrap(),
        })
        .unwrap(),
    );

    prom_timers.insert(id.clone(), chrono::offset::Utc::now());
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

async fn rocket(jobs: Arc<dashmap::DashMap<String, Arc<CronFn>>>) {
    // Id of the request; Histogram that started w/ that request
    let prom_timers = Arc::new(dashmap::DashMap::<Uuid, DateTime<Utc>>::new());

    let metrics: [&HistogramVec; 2] =
        [&HTTP_PRINT_HISTOGRAM, &HTTP_TIME_TO_PROCESS_HISTOGRAM];
    let prometheus = PrometheusMetrics::new();
    for metric in metrics {
        prometheus.registry().register(Box::new(metric.clone())).unwrap();
    }

    let _ = rocket::build()
        .attach(prometheus.clone())
        .manage(jobs.clone())
        .manage(prom_timers)
        .mount("/api", routes![print, put_cron, delete_cron])
        .mount("/metrics", prometheus)
        .ignite()
        .await
        .expect("Rocket failed to ignite")
        .launch()
        .await
        .expect("Rocket launch failed");
}

async fn forever(jobs: Arc<dashmap::DashMap<String, Arc<CronFn>>>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        for value in jobs.iter() {
            let val = value.value().clone();
            tokio::spawn(val());
        }
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "warn,iot_emulation=trace");

    tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(ForestLayer::default())
        .init();

    debug!("Tracing initialized.");

    let jobs = Arc::new(dashmap::DashMap::<String, Arc<CronFn>>::new());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        // .worker_threads(num_cpus::get() * 2)
        .build()
        .expect("build runtime failed")
        .block_on(async {
            tokio::spawn(forever(jobs.clone()));
            let handle = tokio::spawn(rocket(jobs));

            handle.await.expect("handle failed");
        });
}
