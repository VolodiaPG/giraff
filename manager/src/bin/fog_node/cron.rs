use crate::prom_metrics::{
    CPU_ALLOCATABLE_GAUGE, CPU_USAGE_GAUGE, MEMORY_ALLOCATABLE_GAUGE,
    MEMORY_USAGE_GAUGE,
};
use crate::repository::k8s::K8s;
use crate::service::neighbor_monitor::NeighborMonitor;
use std::sync::Arc;
use tokio_cron_scheduler::{Job, JobScheduler};

pub fn init(
    neighbor_monitor: Arc<dyn NeighborMonitor>,
    k8s_repo: Arc<dyn K8s>,
) {
    let sched = JobScheduler::new().unwrap();

    // TODO option to configure ?
    sched
        .add(
            Job::new_async("1/15 * * * * *", move |_, _| {
                let neighbor_monitor = neighbor_monitor.clone();
                Box::pin(async move {
                    ping(neighbor_monitor).await;
                })
            })
            .unwrap(),
        )
        .unwrap();

    sched
        .add(
            Job::new_async("1/15 * * * * *", move |_, _| {
                let k8s_repo = k8s_repo.clone();
                Box::pin(async move {
                    let _ = measure(k8s_repo).await.map_err(|err| {
                        warn!(
                            "An error occurred while CRON measuring from \
                             K8S: {}",
                            err
                        )
                    });
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

async fn measure(k8s_repo: Arc<dyn K8s>) -> anyhow::Result<()> {
    let aggregated_metrics = k8s_repo.get_k8s_metrics().await?;
    for (name, metrics) in aggregated_metrics {
        let allocatable = metrics.allocatable.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Allocatable resource key not found in retrieved metrics"
            )
        })?;
        let usage = metrics.usage.as_ref().ok_or_else(|| {
            anyhow::anyhow!(
                "Usage resource key not found in retrieved metrics"
            )
        })?;
        MEMORY_ALLOCATABLE_GAUGE
            .with_label_values(&[&name])
            .set(allocatable.memory.value);
        MEMORY_USAGE_GAUGE.with_label_values(&[&name]).set(usage.memory.value);
        CPU_ALLOCATABLE_GAUGE
            .with_label_values(&[&name])
            .set(allocatable.cpu.value);
        CPU_USAGE_GAUGE.with_label_values(&[&name]).set(usage.cpu.value);
    }

    Ok(())
}
