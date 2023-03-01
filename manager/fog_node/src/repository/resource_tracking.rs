use std::fmt::Debug;
use std::sync::Arc;

use crate::prom_metrics::{
    CPU_AVAILABLE_GAUGE, CPU_USED_GAUGE, MEMORY_AVAILABLE_GAUGE,
    MEMORY_USED_GAUGE,
};
use uom::si::f64::{Information, Ratio};
use uom::si::information::{self, byte};
use uom::si::ratio::part_per_billion;

use crate::repository::resource_tracking::Error::NonExistentName;

use super::k8s::K8s;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The metrics name (key) doesn't exist")]
    NonExistentName,
    #[error(transparent)]
    K8S(#[from] crate::repository::k8s::Error),
    #[error("The particular metric name was not found")]
    MetricsNotFound,
}

#[derive(Debug, Default)]
pub struct ResourceTracking {
    resources_available: dashmap::DashMap<String, (Information, Ratio)>,
    resources_used:      dashmap::DashMap<String, (Information, Ratio)>,
    nodes:               Vec<String>,
}

impl ResourceTracking {
    pub async fn new(
        k8s: Arc<K8s>,
        reserved_cpu: Ratio,
        reserved_memory: Information,
    ) -> Result<Self, Error> {
        let aggregated_metrics = k8s.get_k8s_metrics().await?;

        let resources_available: Result<dashmap::DashMap<_, _>, Error> =
            aggregated_metrics
                .iter()
                .map(|(name, metrics)| {
                    let allocatable = metrics
                        .allocatable
                        .as_ref()
                        .ok_or(Error::MetricsNotFound)?;
                    let used = metrics
                        .usage
                        .as_ref()
                        .ok_or(Error::MetricsNotFound)?;
                    let free_cpu = allocatable.cpu - used.cpu;
                    let free_ram = allocatable.memory - used.memory;

                    if free_cpu < reserved_cpu {
                        warn!(
                            "Configured reserved CPU (that will be used) >
                    K8S available CPU detected: {:?} > {:?}",
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
                    K8S available RAM detected: {:?} > {:?}",
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

        Ok(Self { resources_available, resources_used, nodes })
    }

    /// Check if the key exists in all storages
    async fn key_exists(&self, name: &str) -> Result<(), Error> {
        if self.resources_used.contains_key(name)
            && self.resources_available.contains_key(name)
        {
            return Ok(());
        }
        Err(NonExistentName)
    }

    /// Update the Prometheus metrics
    async fn update_metrics(&self, name: &'_ str) -> Result<(), Error> {
        let (used_mem, used_cpu) =
            *self.resources_used.get(name).ok_or(Error::NonExistentName)?;

        let (avail_mem, avail_cpu) = *self
            .resources_available
            .get(name)
            .ok_or(Error::NonExistentName)?;

        MEMORY_USED_GAUGE.with_label_values(&[name]).set(used_mem.value);
        MEMORY_AVAILABLE_GAUGE.with_label_values(&[name]).set(avail_mem.value);
        CPU_USED_GAUGE.with_label_values(&[name]).set(used_cpu.value);
        CPU_AVAILABLE_GAUGE.with_label_values(&[name]).set(avail_cpu.value);

        Ok(())
    }

    pub async fn set_used(
        &self,
        name: String,
        memory: Information,
        cpu: Ratio,
    ) -> Result<(), Error> {
        let _ = self.key_exists(&name).await?;
        self.resources_used.insert(name.clone(), (memory, cpu));
        let _ = self.update_metrics(&name).await?;
        Ok(())
    }

    pub async fn get_used(
        &self,
        name: &'_ str,
    ) -> Result<(Information, Ratio), Error> {
        let _ = self.key_exists(name).await?;
        let _ = self.update_metrics(name).await?;
        Ok(*self.resources_used.get(name).unwrap())
    }

    pub async fn get_available(
        &self,
        name: &'_ str,
    ) -> Result<(Information, Ratio), Error> {
        let _ = self.key_exists(name).await?;
        let _ = self.update_metrics(name).await?;
        Ok(*self.resources_available.get(name).unwrap())
    }

    pub fn get_nodes(&self) -> &Vec<String> { &self.nodes }
}
