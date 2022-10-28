extern crate log;
#[macro_use]
extern crate rocket;

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
use tokio::sync::mpsc::{self, Sender};
use tokio::time;
use uuid::Uuid;

type CronFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

type PromTimer = Arc<flurry::HashMap<Uuid, Sender<()>>>;

lazy_static! {
    static ref HTTP_PRINT_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "iot_emulation_http_request_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, tagged \
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
pub async fn print(payload: Json<Payload>, prom_timers: &State<PromTimer>) {
    let timer = prom_timers.pin().remove(&payload.id).cloned();
    if let Some(tx) = timer {
        tx.send(()).await.unwrap();
    }
    info!("{:?}", payload);
}

#[put("/cron", data = "<config>")]
async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: &State<Arc<flurry::HashMap<String, CronFn>>>,
    prom_timers: &State<PromTimer>,
) {
    let config = Arc::new(config.0);
    let prom_timers = prom_timers.inner().clone();
    let tag = config.tag.clone();
    info!(
        "Sending to {:?} on tag {:?}; then to {:?}",
        config.first_node_url, config.tag, config.iot_url
    );

    let job: CronFn = Box::new(move || {
        let prom_timers = prom_timers.clone();
        let config = config.clone();
        Box::pin(ping(prom_timers, config))
    });

    cron_jobs.pin().insert(tag, job);
}

#[delete("/cron/<tag>")]
async fn delete_cron(
    tag: String,
    cron_jobs: &State<Arc<flurry::HashMap<String, CronFn>>>,
) {
    info!("Deleting cron on tag {:?}", tag);
    cron_jobs.pin().remove(&tag);
}

async fn ping(prom_timers: PromTimer, config: Arc<StartCron>) {
    let id = Uuid::new_v4();
    let (tx, mut rx) = mpsc::channel(1);

    {
        prom_timers.pin().insert(id.clone(), tx);
    }
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
    let measure =
        HTTP_PRINT_HISTOGRAM.with_label_values(&[&config.tag]).start_timer();
    let res = res.send().await;

    info!("Ping sent to {:?}.", tag);
    if let Err(err) = res {
        warn!(
            "Discarded measure because something went wrong sending a \
             message using config {:?}, error is {:?}",
            config, err
        );
        measure.stop_and_discard();
        return;
    }

    if rx.recv().await.is_none() {
        warn!("Channed received None, discarding...");
        measure.stop_and_discard();
        return;
    }

    measure.observe_duration();
    info!("Measured {:?} ({:?})", id, tag);
}

async fn rocket(jobs: Arc<flurry::HashMap<String, CronFn>>) {
    // Id of the request; Histogram that started w/ that request
    let prom_timers = Arc::new(flurry::HashMap::<Uuid, Sender<()>>::new());

    let metrics: [&HistogramVec; 1] = [&HTTP_PRINT_HISTOGRAM];
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

async fn forever(jobs: Arc<flurry::HashMap<String, CronFn>>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        let pin = jobs.pin();
        for value in pin.values() {
            tokio::spawn(value());
        }
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "info, echo=trace");
    env_logger::init();

    let jobs = Arc::new(flurry::HashMap::<String, CronFn>::new());

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime failed")
        .block_on(async {
            tokio::spawn(forever(jobs.clone()));
            let handle = tokio::spawn(rocket(jobs));

            handle.await.expect("handle failed");
        });
}
