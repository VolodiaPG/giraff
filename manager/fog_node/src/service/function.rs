use crate::monitoring::{PaidFunctions, ProvisionedFunctions};
use crate::repository::cron::{Cron, Task, TaskEntry, UnprovisionFunction};
use crate::repository::faas::FaaSBackend;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use anyhow::{bail, ensure, Context, Result};
use chrono::{DateTime, Duration, Utc};
use helper::monitoring::MetricsExporter;
use model::domain::sla::Sla;
use model::dto::function::Paid;
use model::SlaId;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use tracing::{error, trace, warn};
use uom::si::f64::{Information, Ratio};
use uom::si::time::millisecond;

pub struct UnprovisionEvent {
    #[allow(dead_code)]
    pub timestamp: DateTime<Utc>,
    #[allow(dead_code)]
    pub sla:       Sla,
    #[allow(dead_code)]
    pub node:      String,
}

pub struct Locked {}

pub struct Unlocked {}

pub struct Function<State = Unlocked> {
    function:          Arc<Box<dyn FaaSBackend>>,
    node_situation:    Arc<NodeSituation>,
    neighbor_monitor:  Arc<NeighborMonitor>,
    node_query:        Arc<NodeQuery>,
    resource_tracking: Arc<ResourceTracking>,
    function_tracking: Arc<FunctionTracking>,
    metrics:           Arc<MetricsExporter>,
    cron:              Arc<Cron>,
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
        function: Arc<Box<dyn FaaSBackend>>,
        node_situation: Arc<NodeSituation>,
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        resource_tracking: Arc<ResourceTracking>,
        function_tracking: Arc<FunctionTracking>,
        metrics: Arc<MetricsExporter>,
        cron: Arc<Cron>,
    ) -> Self {
        Self {
            function,
            node_situation,
            neighbor_monitor,
            node_query,
            resource_tracking,
            function_tracking,
            metrics,
            cron,
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
            cron: self.cron.clone(),
            lock_state: PhantomData,
            lock: self.lock.clone(),
            permit,
        })
    }

    pub(in crate::service) async fn check_and_set_function_is_live(
        &self,
        id: SlaId,
    ) -> Result<()> {
        let function = self
            .function_tracking
            .get_provisioned(&id)
            .with_context(|| {
                format!(
                    "Failed to get provisioned function {} from the tracking \
                     data",
                    id
                )
            })?;
        self.function.check_is_live(&function).await?;

        let live = function.to_live();
        self.function_tracking.save_live(&id, live);
        Ok(())
    }

    #[allow(dead_code)]
    pub async fn get_utilisation_variations(&self) -> Vec<UnprovisionEvent> {
        let mut ret = Vec::new();
        for elem in self.cron.tasks.lock().await.iter() {
            let TaskEntry {
                task:
                    Task::UnprovisionFunction(UnprovisionFunction {
                        sla,
                        node,
                        ..
                    }),
                created_at,
            } = elem;
            if let Some(sla) = self.function_tracking.get_finishable_sla(&sla)
            {
                let timestamp = *created_at
                    + Duration::try_milliseconds(
                        sla.duration.get::<millisecond>() as i64,
                    )
                    .unwrap();
                let node = node.to_string();
                ret.push(UnprovisionEvent { timestamp, sla, node });
            }
        }
        ret
    }
}

impl<State> Drop for Function<State> {
    fn drop(&mut self) {
        trace!("Unlocking FunctionLife Service after drop...");
        let permit = self.permit.take();
        let Some(permit) = permit else {
            return;
        };
        drop(permit);
    }
}

impl Function<Locked> {
    pub(in crate::service) async fn pay_function(
        &self,
        id: SlaId,
    ) -> Result<Paid> {
        let proposal =
            self.function_tracking.get_proposed(&id).with_context(|| {
                format!(
                    "Failed to get proposed function {} from the tracking \
                     data",
                    id
                )
            })?;

        let (used_ram, used_cpu) = self
            .resource_tracking
            .get_used(&proposal.node)
            .await
            .with_context(|| {
                format!(
                    "Failed to get used resources from tracking data for \
                     node {}",
                    &proposal.node
                )
            })?;
        let (available_ram, available_cpu) = self
            .resource_tracking
            .get_available(&proposal.node)
            .await
            .with_context(|| {
                format!(
                    "Failed to get available resources from tracking data \
                     for node {}",
                    &proposal.node
                )
            })?;

        ensure!(
            satisfiability_check(
                &used_ram,
                &used_cpu,
                &available_ram,
                &available_cpu,
                &proposal.sla
            ),
            "The SLA cannot be respected because at least a constrainst is \
             not satisfiable (anymore?!)"
        );

        let name = proposal.node.clone();
        let sla_cpu = proposal.sla.cpu;
        let sla_memory = proposal.sla.memory;

        let paid = proposal.to_paid();

        self.metrics
            .observe(PaidFunctions {
                n:             1,
                function_name: paid.sla.function_live_name.clone(),
                sla_id:        paid.sla.id.to_string(),
                timestamp:     Utc::now(),
            })
            .await?;
        self.function_tracking.save_paid(&id, paid);
        let paid = self.function_tracking.get_paid(&id).context(
            "Could not find any record of the just registerd payment",
        )?;

        let Ok(()) = self
            .resource_tracking
            .set_used(name, used_ram + sla_memory, used_cpu + sla_cpu)
            .await
        else {
            bail!("Could not set updated tracked cpu and memory");
        };

        Ok(paid)
    }

    pub(in crate::service) async fn provision_function(
        &self,
        id: SlaId,
    ) -> Result<()> {
        let paid_proposal =
            self.function_tracking.get_paid(&id).with_context(|| {
                format!(
                    "Failed to get proposed function {} from the tracking \
                     data",
                    id
                )
            })?;

        let provisioned = self
            .function
            .provision_function(id.clone(), paid_proposal)
            .await
            .with_context(|| {
                format!("Failed to provision the proposed function {}", id)
            })?;
        self.metrics
            .observe(ProvisionedFunctions {
                n:             1,
                function_name: provisioned.sla.function_live_name.clone(),
                sla_id:        provisioned.sla.id.to_string(),
                timestamp:     Utc::now(),
            })
            .await?;
        self.function_tracking.save_provisioned(&id, provisioned);

        Ok(())
    }

    pub(in crate::service) async fn finish_function(
        &self,
        function: SlaId,
    ) -> Result<()> {
        if let Some(boxed_record) =
            self.function_tracking.get_finishable(&function)
        {
            let record = boxed_record.to_finished();

            if let Some(removable) =
                self.function_tracking.get_removable(&function)
            {
                if self.function.remove_function(removable).await.is_err() {
                    warn!("Failed to delete function {}", function);
                }
            }

            let name = record.node.clone();
            let sla_cpu = record.sla.cpu;
            let sla_memory = record.sla.memory;

            self.metrics
                .observe(ProvisionedFunctions {
                    n:             0,
                    function_name: record.sla.function_live_name.clone(),
                    sla_id:        record.sla.id.to_string(),
                    timestamp:     Utc::now(),
                })
                .await?;
            self.function_tracking.save_finished(&function, record);

            let Ok((memory, cpu)) =
                self.resource_tracking.get_used(&name).await
            else {
                error!("Could not get tracked cpu and memory");
                return Ok(());
            };
            let Ok(()) = self
                .resource_tracking
                .set_used(name, memory - sla_memory, cpu - sla_cpu)
                .await
            else {
                error!("Could not set updated tracked cpu and memory");
                return Ok(());
            };

            Ok(())
        } else {
            bail!(
                "Current function is not in a state where it can be dropped \
                 from the active pool of functions running on a node"
            );
        }
    }
}
