use shared_models::NodeId;
use anyhow::Result;
use chrono::DateTime;
use chrono::Utc;
use shared_models::node::{PatchNode, PatchNodeResponse};
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio_cron_scheduler::{Job, JobScheduler};

pub fn cron_init(to_market_url: String, my_id: NodeId) {
    let sched = JobScheduler::new().unwrap();

    // TODO option to configure ?
    let to_market_url = Arc::new(to_market_url);
    let shared_last_answered_time = Arc::new(Mutex::new(Utc::now()));
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

async fn ping(to_market_url: Arc<String>, shared_last_answered_time: Arc<Mutex<DateTime<Utc>>>, my_id: Arc<NodeId>) {
    if let Ok(answered_at) = do_ping(to_market_url, my_id).await {
        let mut value = shared_last_answered_time.lock().await;
        *value = answered_at;
    } else {
        trace!("ping failed");
    }
}

async fn do_ping(to_market_url: Arc<String>, my_id: Arc<NodeId>) -> Result<DateTime<Utc>> {
    let client = reqwest::Client::new();
    trace!("pinging to market {}", to_market_url);

    let result: PatchNodeResponse = client
        .patch(format!("http://{}/api/node/{}", to_market_url, my_id))
        .body(
            serde_json::to_string(&PatchNode {
                created_at: Some(Utc::now()),
            })
            .unwrap(),
        )
        .send()
        .await?
        .json()
        .await?;

    Ok(result.answered_at)
}
