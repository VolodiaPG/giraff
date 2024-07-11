use crate::monitoring::{CpuObservedFromPlatform, MemoryObservedFromPlatform};
use crate::repository::cron::Cron;
use crate::repository::k8s::K8s;
use crate::service::neighbor_monitor::NeighborMonitor;
use anyhow::{Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use std::sync::Arc;
use tracing::warn;

pub async fn init(
    cron: Arc<Cron>,
    neighbor_monitor: Arc<NeighborMonitor>,
    k8s_repo: Arc<K8s>,
    metrics: Arc<MetricsExporter>,
) -> Result<()> {
    cron.add_periodic(move || {
        let neighbor_monitor = neighbor_monitor.clone();
        Box::pin(ping(neighbor_monitor))
    })
    .await
    .context("Failed to add periodic task to ping neighbors")?;

    cron.add_periodic(move || {
        let k8s_repo = k8s_repo.clone();
        let metrics = metrics.clone();
        Box::pin(measure(k8s_repo, metrics))
    })
    .await
    .context(
        "Failed to add periodic task to get measurements of k8s cluster \
         metrics",
    )?;

    Ok(())
}

async fn ping(neighbor_monitor: Arc<NeighborMonitor>) {
    if let Err(e) = neighbor_monitor.ping_neighbors_rtt().await {
        warn!("ping_neighbors_rtt failed: {:?}", e);
    };
}

async fn measure(k8s_repo: Arc<K8s>, metrics: Arc<MetricsExporter>) {
    let _ = _measure(k8s_repo, metrics).await.map_err(|err| {
        warn!("An error occurred while CRON measuring from K8S: {:?}", err)
    });
}

async fn _measure(
    k8s_repo: Arc<K8s>,
    metricsdb: Arc<MetricsExporter>,
) -> anyhow::Result<()> {
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

        let timestamp = Utc::now();
        metricsdb
            .observe(MemoryObservedFromPlatform {
                allocatable: allocatable.memory.value,
                used: usage.memory.value,
                name: name.to_string(),
                timestamp,
            })
            .await?;
        metricsdb
            .observe(CpuObservedFromPlatform {
                allocatable: allocatable.cpu.value,
                used: usage.cpu.value,
                name: name.to_string(),
                timestamp,
            })
            .await?;
    }

    Ok(())
}
