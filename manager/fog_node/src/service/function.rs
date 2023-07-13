use crate::monitoring::ProvisionedFunctions;
use crate::repository::faas::FaaSBackend;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use anyhow::{ensure, Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use model::domain::sla::Sla;
use model::BidId;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uom::si::f64::{Information, Ratio};

pub struct Locked {}

pub struct Unlocked {}

pub struct Function<State = Unlocked> {
    function:          Arc<FaaSBackend>,
    node_situation:    Arc<NodeSituation>,
    neighbor_monitor:  Arc<NeighborMonitor>,
    node_query:        Arc<NodeQuery>,
    resource_tracking: Arc<ResourceTracking>,
    function_tracking: Arc<FunctionTracking>,
    metrics:           Arc<MetricsExporter>,
    lock_state:        PhantomData<State>,
    lock:              Arc<Semaphore>,
    permit:            Option<OwnedSemaphorePermit>,
}

/// Check if the SLA is satisfiable by the current node (designated by name
/// and metrics).
pub(in crate::service) fn satisfiability_check(
    used_ram: &Information,
    used_cpu: &Ratio,
    available_ram: &Information,
    available_cpu: &Ratio,
    sla: &Sla,
) -> bool {
    let would_be_used_ram = *used_ram + (sla.memory * sla.max_replica as f64);
    let would_be_used_cpu = *used_cpu + (sla.cpu * sla.max_replica as f64);

    would_be_used_cpu < *available_cpu && would_be_used_ram < *available_ram
}

impl Function {
    pub fn new(
        function: Arc<FaaSBackend>,
        node_situation: Arc<NodeSituation>,
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        resource_tracking: Arc<ResourceTracking>,
        function_tracking: Arc<FunctionTracking>,
        metrics: Arc<MetricsExporter>,
    ) -> Self {
        Self {
            function,
            node_situation,
            neighbor_monitor,
            node_query,
            resource_tracking,
            function_tracking,
            metrics,
            lock_state: PhantomData,
            lock: Arc::new(Semaphore::new(1)),
            permit: None,
        }
    }

    pub(in crate::service) async fn lock(&self) -> Result<Function<Locked>> {
        trace!("Locking Function Service...");
        let permit =
            Some(
                self.lock.clone().acquire_owned().await.context(
                    "Failed to obtain lock on the function service",
                )?,
            );

        Ok(Function {
            function: self.function.clone(),
            node_situation: self.node_situation.clone(),
            neighbor_monitor: self.neighbor_monitor.clone(),
            node_query: self.node_query.clone(),
            resource_tracking: self.resource_tracking.clone(),
            function_tracking: self.function_tracking.clone(),
            metrics: self.metrics.clone(),
            lock_state: PhantomData,
            lock: self.lock.clone(),
            permit,
        })
    }
}

impl<State> Drop for Function<State> {
    fn drop(&mut self) {
        trace!("Unlocking FunctionLife Service after drop...");
        let permit = self.permit.take();
        let Some(permit) =  permit else {
                return;
            };
        drop(permit);
    }
}

impl Function<Locked> {
    pub(in crate::service) async fn provision_function(
        &self,
        id: BidId,
    ) -> Result<()> {
        let proposed =
            self.function_tracking.get_proposed(&id).with_context(|| {
                format!(
                    "Failed to get proposed function {} from the tracking \
                     data",
                    id
                )
            })?;

        let (used_ram, used_cpu) = self
            .resource_tracking
            .get_used(&proposed.0.node)
            .await
            .with_context(|| {
                format!(
                    "Failed to get used resources from tracking data for \
                     node {}",
                    &proposed.0.node
                )
            })?;
        let (available_ram, available_cpu) = self
            .resource_tracking
            .get_available(&proposed.0.node)
            .await
            .with_context(|| {
                format!(
                    "Failed to get available resources from tracking data \
                     for node {}",
                    &proposed.0.node
                )
            })?;

        ensure!(
            satisfiability_check(
                &used_ram,
                &used_cpu,
                &available_ram,
                &available_cpu,
                &proposed.0.sla
            ),
            "The SLA cannot be respected because at least a constrainst is \
             not satisfiable (anymore?!)"
        );

        let name = proposed.0.node.clone();
        let sla_cpu = proposed.0.sla.cpu;
        let sla_memory = proposed.0.sla.memory;

        let provisioned = self
            .function
            .provision_function(id.clone(), (*proposed).clone())
            .await
            .with_context(|| {
                format!("Failed to provision the proposed function {}", id)
            })?;
        self.metrics
            .observe(ProvisionedFunctions {
                n:             1,
                function_name: provisioned.0.sla.function_live_name.clone(),
                sla_id:        provisioned.0.sla.id.to_string(),
                timestamp:     Utc::now(),
            })
            .await?;
        self.function_tracking.save_provisioned(&id, provisioned);

        let Ok(()) = self
                .resource_tracking
                .set_used(name, used_ram + sla_memory, used_cpu + sla_cpu)
                .await else {
                    error!("Could not set updated tracked cpu and memory");
                    return Ok(());
                };

        Ok(())
    }

    pub(in crate::service) async fn unprovision_function(
        &self,
        function: BidId,
    ) -> Result<()> {
        let record = self
            .function_tracking
            .get_provisioned(&function)
            .with_context(|| {
                format!(
                    "Failed to get provisioned function {} from the tracking \
                     data",
                    function
                )
            })?;
        if self.function.remove_function(&record).await.is_err() {
            warn!("Failed to delete function {}", function);
        }

        let name = record.0.node.clone();
        let sla_cpu = record.0.sla.cpu;
        let sla_memory = record.0.sla.memory;

        let record = (*record).clone().to_finished();

        self.metrics
            .observe(ProvisionedFunctions {
                n:             0,
                function_name: record.0.sla.function_live_name.clone(),
                sla_id:        record.0.sla.id.to_string(),
                timestamp:     Utc::now(),
            })
            .await?;
        self.function_tracking.save_finished(&function, record);

        let Ok((memory, cpu)) = self.resource_tracking.get_used(&name).await else{
            error!("Could not get tracked cpu and memory");
                return Ok(());
            };
        let Ok(()) = self
                .resource_tracking
                .set_used(name, memory - sla_memory, cpu - sla_cpu)
                .await else {
                    error!("Could not set updated tracked cpu and memory");
                    return Ok(());
                };

        Ok(())
    }
}
