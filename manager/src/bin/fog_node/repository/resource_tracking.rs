use std::collections::HashMap;
use std::fmt::Debug;
use std::sync::Arc;

use crate::prom_metrics::{
    CPU_AVAILABLE_GAUGE, CPU_USED_GAUGE, MEMORY_AVAILABLE_GAUGE, MEMORY_USED_GAUGE,
};
use async_trait::async_trait;
use tokio::sync::RwLock;
use uom::si::f64::{Information, Ratio};
use uom::si::information::byte;
use uom::si::ratio::part_per_billion;

use crate::repository::k8s::K8s;
use crate::repository::resource_tracking::Error::NonExistentName;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("The metrics name (key) doesn't exist")]
    NonExistentName,
    #[error(transparent)]
    K8S(#[from] crate::repository::k8s::Error),
    #[error("The particular metric name was not found")]
    MetricsNotFound,
}

/// Behaviour of the routing
#[async_trait]
pub trait ResourceTracking: Debug + Sync + Send {
    /// Update a node given its name with said resource usage
    async fn update_used(&self, name: String, memory: Information, cpu: Ratio)
        -> Result<(), Error>;

    /// Get the used (memory, cpu).
    async fn get_used(&self, name: &'_ str) -> Result<(Information, Ratio), Error>;

    /// Get the available (memory, cpu).
    /// This value is the total available resources at the startup;
    /// it needs to be put in perspective with the usage values.
    async fn get_available(&self, name: &'_ str) -> Result<(Information, Ratio), Error>;

    /// Get all the detected nodes connected
    fn get_nodes(&self) -> &Vec<String>;
}

#[derive(Debug, Default)]
pub struct ResourceTrackingImpl {
    resources_available: RwLock<HashMap<String, (Information, Ratio)>>,
    resources_used: RwLock<HashMap<String, (Information, Ratio)>>,
    nodes: Vec<String>,
}

impl ResourceTrackingImpl {
    pub async fn new(k8s: Arc<dyn K8s>) -> Result<Self, Error> {
        let aggregated_metrics = k8s.get_k8s_metrics().await?;

        let resources_available: Result<HashMap<_, _>, Error> = aggregated_metrics
            .iter()
            .map(
                |(name, metrics)| -> Result<(String, (Information, Ratio)), Error> {
                    let allocatable = metrics.allocatable.as_ref().ok_or(Error::MetricsNotFound)?;
                    let used = metrics.usage.as_ref().ok_or(Error::MetricsNotFound)?;
                    let free_cpu = allocatable.cpu - used.cpu;
                    let free_ram = allocatable.memory - used.memory;
                    Ok((name.clone(), (free_ram, free_cpu)))
                },
            )
            .collect();
        let resources_available = resources_available?;

        let resources_used: HashMap<_, _> = aggregated_metrics
            .iter()
            .map(|(name, _)| {
                (
                    name.clone(),
                    (
                        Information::new::<byte>(0.0),
                        Ratio::new::<part_per_billion>(0.0),
                    ),
                )
            })
            .collect();
        let resources_used = RwLock::new(resources_used);

        let nodes = resources_available.keys().cloned().collect();

        let resources_available = RwLock::new(resources_available);

        Ok(Self {
            resources_available,
            resources_used,
            nodes,
        })
    }

    /// Check if the key exists in all storages
    async fn key_exists(&self, name: &str) -> Result<(), Error> {
        if self.resources_used.read().await.contains_key(name)
            && self.resources_available.read().await.contains_key(name)
        {
            return Ok(());
        }
        Err(NonExistentName)
    }

    /// Update the Prometheus metrics
    async fn update_metrics(&self, name: &'_ str) -> Result<(), Error> {
        let (used_mem, used_cpu) = *self
            .resources_used
            .read()
            .await
            .get(name)
            .ok_or(Error::NonExistentName)?;

        let (avail_mem, avail_cpu) = *self
            .resources_available
            .read()
            .await
            .get(name)
            .ok_or(Error::NonExistentName)?;

        MEMORY_USED_GAUGE
            .with_label_values(&[name])
            .set(used_mem.value);
        MEMORY_AVAILABLE_GAUGE
            .with_label_values(&[name])
            .set(avail_mem.value);
        CPU_USED_GAUGE
            .with_label_values(&[name])
            .set(used_cpu.value);
        CPU_AVAILABLE_GAUGE
            .with_label_values(&[name])
            .set(avail_cpu.value);

        Ok(())
    }
}

#[async_trait]
impl ResourceTracking for ResourceTrackingImpl {
    async fn update_used(
        &self,
        name: String,
        memory: Information,
        cpu: Ratio,
    ) -> Result<(), Error> {
        let _ = self.key_exists(&name).await?;
        self.resources_used
            .write()
            .await
            .insert(name.clone(), (memory, cpu));
        let _ = self.update_metrics(&name).await?;
        Ok(())
    }

    async fn get_used(&self, name: &'_ str) -> Result<(Information, Ratio), Error> {
        let _ = self.key_exists(name).await?;
        let _ = self.update_metrics(name).await?;
        Ok(*self.resources_used.read().await.get(name).unwrap())
    }

    async fn get_available(&self, name: &'_ str) -> Result<(Information, Ratio), Error> {
        let _ = self.key_exists(name).await?;
        let _ = self.update_metrics(name).await?;
        Ok(*self.resources_available.read().await.get(name).unwrap())
    }

    fn get_nodes(&self) -> &Vec<String> {
        &self.nodes
    }
}
