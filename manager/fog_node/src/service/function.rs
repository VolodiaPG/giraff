use crate::repository::faas::FaaSBackend;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use model::BidId;
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    FaaS(#[from] crate::repository::faas::Error),
    #[error(transparent)]
    NodeQuery(#[from] crate::repository::node_query::Error),
    #[error("Lock failed to be secured")]
    Lock,
    #[error("BidId {0} cannot be retrieved")]
    MissingBidId(BidId),
}
pub struct Locked {}

pub struct Unlocked {}

pub struct Function<State = Unlocked> {
    function:          Arc<FaaSBackend>,
    node_situation:    Arc<NodeSituation>,
    neighbor_monitor:  Arc<NeighborMonitor>,
    node_query:        Arc<NodeQuery>,
    resource_tracking: Arc<ResourceTracking>,
    function_tracking: Arc<FunctionTracking>,
    lock_state:        PhantomData<State>,
    lock:              Arc<Semaphore>,
    permit:            Option<OwnedSemaphorePermit>,
}

impl Function {
    pub fn new(
        function: Arc<FaaSBackend>,
        node_situation: Arc<NodeSituation>,
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        resource_tracking: Arc<ResourceTracking>,
        function_tracking: Arc<FunctionTracking>,
    ) -> Self {
        Self {
            function,
            node_situation,
            neighbor_monitor,
            node_query,
            resource_tracking,
            function_tracking,
            lock_state: PhantomData,
            lock: Arc::new(Semaphore::new(1)),
            permit: None,
        }
    }

    pub(in crate::service) async fn lock(
        &self,
    ) -> Result<Function<Locked>, Error> {
        trace!("Locking Function Service...");
        let permit = Some(
            self.lock
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| Error::Lock)?,
        );

        Ok(Function {
            function: self.function.clone(),
            node_situation: self.node_situation.clone(),
            neighbor_monitor: self.neighbor_monitor.clone(),
            node_query: self.node_query.clone(),
            resource_tracking: self.resource_tracking.clone(),
            function_tracking: self.function_tracking.clone(),
            lock_state: PhantomData,
            lock: self.lock.clone(),
            permit,
        })
    }
}

impl<State> Drop for Function<State> {
    fn drop(&mut self) {
        trace!("Unlocking FunctionLife Service after drop...");
        let permit = std::mem::replace(&mut self.permit, None);
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
    ) -> Result<(), Error> {
        let proposed = self
            .function_tracking
            .get_proposed(&id)
            .ok_or_else(|| Error::MissingBidId(id.clone()))?;
        let name = proposed.0.node.clone();
        let sla_cpu = proposed.0.sla.cpu;
        let sla_memory = proposed.0.sla.memory;

        let provisioned = self
            .function
            .provision_function(id.clone(), (*proposed).clone())
            .await?;

        self.function_tracking.save_provisioned(&id, provisioned);

        let Ok((memory, cpu)) = self.resource_tracking.get_used(&name).await else{
            error!("Could not get tracked cpu and memory");
                return Ok(());
            };
        let Ok(()) = self
                .resource_tracking
                .set_used(name, memory + sla_memory, cpu + sla_cpu)
                .await else {
                    error!("Could not set updated tracked cpu and memory");
                    return Ok(());
                };
        Ok(())
    }

    pub(in crate::service) async fn unprovision_function(
        &self,
        function: BidId,
    ) -> Result<(), Error> {
        let record = self
            .function_tracking
            .get_provisioned(&function)
            .ok_or_else(|| Error::MissingBidId(function.clone()))?;
        if self.function.remove_function(&record).await.is_err() {
            warn!("Failed to delete function {}", function);
        }

        let name = record.0.node.clone();
        let sla_cpu = record.0.sla.cpu;
        let sla_memory = record.0.sla.memory;

        let record = (*record).clone().to_finished();
        self.function_tracking.save_finished(&function, record);

        let Ok((memory, cpu)) = self.resource_tracking.get_used(&name).await else{
            error!("Could not get tracked cpu and memory");
                return Ok(());
            };
        let Ok(()) = self
                .resource_tracking
                .set_used(name, memory + sla_memory, cpu + sla_cpu)
                .await else {
                    error!("Could not set updated tracked cpu and memory");
                    return Ok(());
                };

        Ok(())
    }

    // pub(crate) fn unlock(self) -> Function<Unlocked> {
    //     // Unlock done through custom impl of Drop after cunsumption of self
    //     Function {
    //         function:          self.function.clone(),
    //         node_situation:    self.node_situation.clone(),
    //         neighbor_monitor:  self.neighbor_monitor.clone(),
    //         node_query:        self.node_query.clone(),
    //         resource_tracking: self.resource_tracking.clone(),
    //         function_tracking: self.function_tracking.clone(),
    //         lock_state:        PhantomData,
    //         lock:              self.lock.clone(),
    //         permit:            None,
    //     }
    // }
}
