#[macro_use]
extern crate log;
extern crate rocket;

use lazy_static::lazy_static;
use rocket::serde::json::Json;
use rocket::{delete, launch, post, put, routes, State};
use rocket_prometheus::prometheus::{
    register_histogram_vec, HistogramTimer, HistogramVec,
};
use rocket_prometheus::PrometheusMetrics;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};
use uuid::Uuid;

lazy_static! {
    static ref HTTP_PRINT_HISTOGRAM: HistogramVec = register_histogram_vec!(
        "http_request_duration_seconds_print",
        "The HTTP request latencies in seconds for the /print route, tagged \
         with the content `tag`.",
        &["tag"]
    )
    .unwrap();
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Payload {
    tag: String,
    id:  Uuid,
}

#[derive(Debug, Deserialize)]
pub struct StartCron {
    pub url:  String,
    pub tag:  String,
    pub data: String,
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
pub async fn put_cron(
    config: Json<StartCron>,
    cron_jobs: &State<Arc<Mutex<HashMap<String, JobScheduler>>>>,
    prom_timers: &State<Arc<Mutex<HashMap<Uuid, HistogramTimer>>>>,
) {
    let config = Arc::new(config.0);
    let prom_timers = prom_timers.inner().clone();
    let tag = config.tag.clone();
    info!(
        "Sending to {:?} on tag {:?} sending payload {:?}",
        config.url, config.tag, config.data
    );
    let sched = JobScheduler::new().unwrap();
    sched
        .add(
            Job::new_async("1/5 * * * * *", move |_, _| {
                let config = config.clone();
                let prom_timers = prom_timers.clone();
                Box::pin(async move {
                    let id = Uuid::new_v4();
                    prom_timers.lock().await.insert(
                        id.clone(),
                        HTTP_PRINT_HISTOGRAM
                            .with_label_values(&[&config.tag])
                            .start_timer(),
                    );
                    let tag = config.tag.clone();

                    let res = reqwest::Client::new()
                        .post(config.url.clone())
                        .body(
                            serde_json::to_string(&Payload { tag, id })
                                .unwrap(),
                        )
                        .send()
                        .await;
                    if let Err(err) = res {
                        warn!(
                            "Something went wrong sending a message using \
                             config {:?}, error is {:?}",
                            config, err
                        );
                    }
                })
            })
            .unwrap(),
        )
        .unwrap();

    if let Err(err) = sched.start() {
        error!("Cannot start cron sheduler: {:?}", err);
    }

    cron_jobs.lock().await.insert(tag, sched);
}

#[delete("/cron/<tag>")]
pub async fn delete_cron(
    tag: String,
    cron_jobs: &State<Arc<Mutex<HashMap<String, JobScheduler>>>>,
) {
    info!("Deleting cron on tag {:?}", tag);
    cron_jobs.lock().await.remove(&tag);
}

#[launch]
async fn rocket() -> _ {
    std::env::set_var("RUST_LOG", "info, echo=trace");
    env_logger::init();

    let jobs = Arc::new(Mutex::new(HashMap::<String, JobScheduler>::new()));
    let prom_timers =
        Arc::new(Mutex::new(HashMap::<Uuid, HistogramTimer>::new()));

    let metrics: [&HistogramVec; 1] = [&HTTP_PRINT_HISTOGRAM];
    let prometheus = PrometheusMetrics::new();
    for metric in metrics {
        prometheus.registry().register(Box::new(metric.clone())).unwrap();
    }

    rocket::build()
        .attach(prometheus.clone())
        .manage(jobs)
        .manage(prom_timers)
        .mount("/api", routes![print, put_cron, delete_cron])
        .mount("/metrics", prometheus)
}
