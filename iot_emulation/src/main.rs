extern crate log;
#[macro_use]
extern crate rocket;

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use lazy_static::lazy_static;
use rocket::{delete, post, put, routes, State};
use rocket::futures::future::join_all;
use rocket::serde::json::Json;
use rocket_prometheus::prometheus::{
    HistogramTimer, HistogramVec, register_histogram_vec,
};
use rocket_prometheus::PrometheusMetrics;
use serde::{Deserialize, Serialize};
use serde_json::value::RawValue;
use tokio::sync::{Mutex, RwLock};
use tokio::time;
use uuid::Uuid;

type CronFn = Box<
    dyn Fn() -> Pin<Box<dyn Future<Output=()> + Send >>
    + Send
    + Sync
>;

lazy_static! {
    static ref HTTP_PRINT_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds_print",
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
        to: Uuid,
        // TODO check wether its better an id or a name
        #[serde(borrow)]
        data: &'a RawValue,
    },
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Payload {
    tag: String,
    id: Uuid,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct CronPayload {
    address_to_call: String,
    data: Payload,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FaaSPacket {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartCron {
    pub iot_url: String,
    pub first_node_url: String,
    pub function_id: Uuid,
    pub tag: String,
}

#[post("/print", data = "<payload>")]
pub async fn print(
    payload: Json<Payload>,
    prom_timers: &State<Arc<Mutex<HashMap<Uuid, HistogramTimer>>>>,
) {
    info!("{:?}", payload);
    if let Some(timer) = prom_timers.lock().await.remove(&payload.id) {
        timer.observe_duration();
    }
}

#[put("/cron", data = "<config>")]
async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: &State<Arc<RwLock<HashMap<String, CronFn>>>>,
    prom_timers: &State<Arc<Mutex<HashMap<Uuid, HistogramTimer>>>>,
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
        let config  = config.clone();
        Box::pin(ping(prom_timers, config))
    });

    cron_jobs.write().await.insert(tag, job);
}

#[delete("/cron/<tag>")]
async fn delete_cron(
    tag: String,
    cron_jobs: &State<Arc<RwLock<HashMap<String, CronFn>>>>,
) {
    info!("Deleting cron on tag {:?}", tag);
    cron_jobs.write().await.remove(&tag);
}

async fn ping(
    prom_timers: Arc<Mutex<HashMap<Uuid, HistogramTimer>>>,
    config: Arc<StartCron>,
) {
    let id = Uuid::new_v4();
    {
        prom_timers.lock().await.insert(
            id.clone(),
            HTTP_PRINT_HISTOGRAM
                .with_label_values(&[&config.tag])
                .start_timer(),
        );
    }
    let tag = config.tag.clone();

    let res = reqwest::Client::new()
        .post(config.first_node_url.clone())
        .body(
            serde_json::to_string(&Packet::FaaSFunction {
                to: config.function_id.clone(),
                data: &serde_json::value::to_raw_value(&CronPayload {
                    address_to_call: config.iot_url.clone(),
                    data: Payload { tag, id },
                })
                    .unwrap(),
            })
                .unwrap(),
        )
        .send()
        .await;
    if let Err(err) = res {
        warn!(
            "Something went wrong sending a message using config {:?}, error \
             is {:?}",
            config, err
        );
    }
}

async fn rocket(jobs: Arc<RwLock<HashMap<String, CronFn>>>) {
    // Id of the request; Histogram that started w/ that request
    let prom_timers =
        Arc::new(Mutex::new(HashMap::<Uuid, HistogramTimer>::new()));

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

async fn forever(jobs: Arc<RwLock<HashMap<String, CronFn>>>) {
    let mut interval = time::interval(Duration::from_secs(5));

    loop {
        interval.tick().await;
        join_all(jobs.read().await.values().map(|func| func())).await;
    }
}

fn main() {
    std::env::set_var("RUST_LOG", "info, echo=trace");
    env_logger::init();

    let jobs = Arc::new(RwLock::new(HashMap::<String, CronFn>::new()));

    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("build runtime failed")
        .block_on(async {
            tokio::spawn(forever(jobs.clone()));
            let handle =tokio::spawn(rocket(jobs));

            handle.await.expect("handle failed");
        });

}
