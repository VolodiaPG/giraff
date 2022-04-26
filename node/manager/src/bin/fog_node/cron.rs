use std::sync::Arc;

use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

use manager::model::{
    BidId,
    NodeId,
    Reserved, view::node::{PostNode, PostNodeResponse},
};

pub fn generate_market_url_as_root_node(market_url: String) -> String {
    format!("http://{}/api/node", market_url)
}

pub fn generate_market_url_as_regular_node(to_market_url: String) -> String {
    format!("http://{}/api/routing/{}", to_market_url, BidId::from(Reserved::MarketPing))
}

pub fn init(to_market_url: String, my_id: NodeId) {
    let sched = JobScheduler::new().unwrap();

    // TODO option to configure ?
    let to_market_url = Arc::new(to_market_url);
    let shared_last_answered_time = Arc::new(Mutex::new((Utc::now(), Utc::now())));
    let my_id = Arc::new(my_id);
    sched
        .add(
            Job::new_async("1/5 * * * * *", move |_, _| {
                let to_market_url = to_market_url.clone();
                let shared_last_answered_time = shared_last_answered_time.clone();
                let my_id = my_id.clone();
                Box::pin(async move {
                    ping(to_market_url, shared_last_answered_time, my_id).await;
                })
            })
                .unwrap(),
        )
        .unwrap();

    sched.start().unwrap();
}

async fn ping(to_market_url: Arc<String>, shared_last_answered_time: Arc<Mutex<(DateTime<Utc>, DateTime<Utc>)>>, my_id: Arc<NodeId>) {
    if let Ok(answered_at) = do_ping(to_market_url, &shared_last_answered_time, my_id).await {
        let mut value = shared_last_answered_time.lock().await;
        *value = (Utc::now(), answered_at);
    } else {
        trace!("ping failed");
    }
}

async fn do_ping(to_market_url: Arc<String>, shared_last_answered_time: &Arc<Mutex<(DateTime<Utc>, DateTime<Utc>)>>, my_id: Arc<NodeId>) -> Result<DateTime<Utc>> {
    let client = reqwest::Client::new();
    trace!("pinging to market {}", to_market_url);

    let (last_answer_received_at, last_answered_at) = shared_last_answered_time.lock().await.to_owned();

    let result: PostNodeResponse = client
        .post(to_market_url.as_str())
        .body(
            serde_json::to_string(&PostNode {
                created_at: Utc::now(),
                last_answered_at: Some(last_answered_at),
                last_answer_received_at: Some(last_answer_received_at),
                from: (*my_id).clone(),
            })
                .unwrap(),
        )
        .send()
        .await?
        .json()
        .await?;

    Ok(result.answered_at)
}
