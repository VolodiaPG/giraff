use crate::prom_metrics::{
    CPU_ALLOCATABLE_GAUGE, CPU_USAGE_GAUGE, MEMORY_ALLOCATABLE_GAUGE,
    MEMORY_USAGE_GAUGE,
};
use crate::repository::k8s::K8s;
use crate::service::neighbor_monitor::NeighborMonitor;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type CronFn =
    Box<dyn Fn() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send + Sync>;

pub async fn init(
    neighbor_monitor: Arc<dyn NeighborMonitor>,
    k8s_repo: Arc<dyn K8s>,
) -> Vec<CronFn> {
    let mut jobs: Vec<CronFn> = Vec::new();

    jobs.push(Box::new(move || {
        let neighbor_monitor = neighbor_monitor.clone();
        Box::pin(ping(neighbor_monitor))
    }));

    jobs.push(Box::new(move || {
        let k8s_repo = k8s_repo.clone();
        Box::pin(measure(k8s_repo))
    }));

    jobs
}

async fn ping(neighbor_monitor: Arc<dyn NeighborMonitor>) {
    if let Err(e) = neighbor_monitor.ping_neighbors_rtt().await {
        warn!("ping_neighbors_rtt failed: {}", e.to_string());
    };
}

async fn measure(k8s_repo: Arc<dyn K8s>) {
    let _ = _measure(k8s_repo).await.map_err(|err| {
        warn!("An error occurred while CRON measuring from K8S: {}", err)
    });
}

async fn _measure(k8s_repo: Arc<dyn K8s>) -> anyhow::Result<()> {
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
