use crate::monitoring::{CpuObservedFromFogNode, MemoryObservedFromFogNode};

use super::k8s::K8s;
use anyhow::{ensure, Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::{Information, Ratio};
use uom::si::information::{self, byte};
use uom::si::ratio::part_per_billion;

#[derive(Debug)]
pub struct ResourceTracking {
    metrics:             Arc<MetricsExporter>,
    resources_available: dashmap::DashMap<String, (Information, Ratio)>,
    resources_used:      dashmap::DashMap<String, (Information, Ratio)>,
    nodes:               Vec<String>,
}

impl ResourceTracking {
    pub async fn new(
        k8s: Arc<K8s>,
        metrics: Arc<MetricsExporter>,
        reserved_cpu: Ratio,
        reserved_memory: Information,
    ) -> Result<Self> {
        let aggregated_metrics = k8s
            .get_k8s_metrics()
            .await
            .context("Failed to get the available cluster metrics")?;

        let resources_available: Result<dashmap::DashMap<_, _>> =
            aggregated_metrics
                .iter()
                .map(|(name, metrics)| {
                    let allocatable = metrics.allocatable.as_ref().context(
                        "Failed to retrieve allocatable resources from the \
                         cluster metrics gathered",
                    )?;
                    let used = metrics.usage.as_ref().context(
                        "Failed to retrieve used resources from the cluster \
                         metrics gathered",
                    )?;
                    let free_cpu = allocatable.cpu - used.cpu;
                    let free_ram = allocatable.memory - used.memory;

                    if free_cpu < reserved_cpu {
                        warn!(
                            "Configured reserved CPU (that will be used) >
                    K8S available CPU detected: {:?} -> {:?}",
                            free_cpu.into_format_args(
                                helper::uom_helper::cpu_ratio::cpu,
                                uom::fmt::DisplayStyle::Abbreviation
                            ),
                            reserved_cpu.into_format_args(
                                helper::uom_helper::cpu_ratio::cpu,
                                uom::fmt::DisplayStyle::Abbreviation
                            )
                        );
                    }

                    if free_ram < reserved_memory {
                        warn!(
                            "Configured reserved RAM (that will be used) >
                    K8S available RAM detected: {:?} -> {:?}",
                            free_ram.into_format_args(
                                information::gigabyte,
                                uom::fmt::DisplayStyle::Abbreviation
                            ),
                            reserved_memory.into_format_args(
                                information::gigabyte,
                                uom::fmt::DisplayStyle::Abbreviation
                            )
                        );
                    }
                    Ok((name.clone(), (reserved_memory, reserved_cpu)))
                })
                .collect();
        let resources_available = resources_available?;

        let resources_used: dashmap::DashMap<_, _> = aggregated_metrics
            .keys()
            .map(|name| {
                (
                    name.clone(),
                    (
                        Information::new::<byte>(0.0),
                        Ratio::new::<part_per_billion>(0.0),
                    ),
                )
            })
            .collect();

        let nodes =
            { resources_available.iter().map(|x| x.key().clone()).collect() };

        Ok(Self { resources_available, metrics, resources_used, nodes })
    }

    /// Check if the key exists in all storages
    async fn key_exists(&self, name: &str) -> Result<()> {
        ensure!(
            self.resources_used.contains_key(name),
            "Key '{}' doesn't exist in the resource used map",
            name
        );
        ensure!(
            self.resources_available.contains_key(name),
            "Key '{}' doesn't exist in the resource availables map",
            name
        );
        Ok(())
    }

    /// Update the Prometheus metrics
    async fn update_metrics(&self, name: &'_ str) -> Result<()> {
        let (used_mem, used_cpu) =
            *self.resources_used.get(name).with_context(|| {
                format!(
                    "Key '{}' doesn't exist in the resource used map",
                    name
                )
            })?;

        let (avail_mem, avail_cpu) =
            *self.resources_available.get(name).with_context(|| {
                format!(
                    "Key '{}' doesn't exist in the resource available map",
                    name
                )
            })?;

        let timestamp = Utc::now();
        self.metrics
            .observe(MemoryObservedFromFogNode {
                initial_allocatable: avail_mem.get::<information::gigabyte>(),
                used: used_mem.get::<information::gigabyte>(),
                name: name.to_string(),
                timestamp,
            })
            .await?;
        self.metrics
            .observe(CpuObservedFromFogNode {
                initial_allocatable: avail_cpu
                    .get::<helper::uom_helper::cpu_ratio::cpu>(
                ),
                used: used_cpu.get::<helper::uom_helper::cpu_ratio::cpu>(),
                name: name.to_string(),
                timestamp,
            })
            .await?;

        Ok(())
    }

    pub async fn set_used(
        &self,
        name: String,
        memory: Information,
        cpu: Ratio,
    ) -> Result<()> {
        self.key_exists(&name)
            .await
            .with_context(|| format!("Cannot set used metric {}", name))?;
        assert!(
            cpu > Ratio::new::<helper::uom_helper::cpu_ratio::cpu>(0.0),
            "CPU value to set is not > 0"
        );
        assert!(
            memory > Information::new::<information::gigabyte>(0.0),
            "Memory value to set is not > 0"
        );
        self.resources_used.insert(name.clone(), (memory, cpu));
        self.update_metrics(&name).await.with_context(|| {
            format!("Failed to update prometheus metric {}", name)
        })?;
        Ok(())
    }

    pub async fn get_used(
        &self,
        name: &'_ str,
    ) -> Result<(Information, Ratio)> {
        self.key_exists(name)
            .await
            .with_context(|| format!("Cannot get used metric {}", name))?;
        self.update_metrics(name).await.with_context(|| {
            format!("Failed to update prometheus metric {}", name)
        })?;
        Ok(*self.resources_used.get(name).unwrap())
    }

    pub async fn get_available(
        &self,
        name: &'_ str,
    ) -> Result<(Information, Ratio)> {
        self.key_exists(name)
            .await
            .with_context(|| format!("Cannot get used metric {}", name))?;
        self.update_metrics(name).await.with_context(|| {
            format!("Failed to update prometheus metric {}", name)
        })?;
        Ok(*self.resources_available.get(name).unwrap())
    }

    pub fn get_nodes(&self) -> &Vec<String> { &self.nodes }
}
