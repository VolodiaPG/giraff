use crate::service::neighbor_monitor::NeighborMonitor;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub fn init(neighbor_monitor: Arc<dyn NeighborMonitor>) {
    let sched = JobScheduler::new().unwrap();

    // TODO option to configure ?
    sched
        .add(
            Job::new_async("1/5 * * * * *", move |_, _| {
                let neighbor_monitor = neighbor_monitor.clone();
                Box::pin(async move {
                    ping(neighbor_monitor).await;
                })
            })
            .unwrap(),
        )
        .unwrap();

    sched.start().unwrap();
}

async fn ping(neighbor_monitor: Arc<dyn NeighborMonitor>) {
    if let Err(e) = neighbor_monitor.ping_neighbors_rtt().await {
        warn!("ping_neighbors_rtt failed: {}", e.to_string());
    };
}
