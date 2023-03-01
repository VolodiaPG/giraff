use crate::repository::faas::FaaSBackend;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use crate::service::auction::Auction;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use model::domain::sla::Sla;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::{BidId, NodeId};
use std::marker::PhantomData;
use std::sync::Arc;
use tokio::sync::{OwnedSemaphorePermit, Semaphore};
use uom::si::f64::Time;

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
    #[cfg(any(feature = "edge_first", feature = "edge_ward"))]
    #[error("No candidates were found or returned an Ok result")]
    NoCandidatesRetained,
    #[cfg(feature = "cloud_only")]
    #[error("No cloud was found or returned an Ok result")]
    NoCloudAvailable,
}

pub struct Locked {}
pub struct Unlocked {}

pub struct FunctionLife<State = Unlocked> {
    function:          Arc<FaaSBackend>,
    auction:           Arc<Auction>,
    node_situation:    Arc<NodeSituation>,
    neighbor_monitor:  Arc<NeighborMonitor>,
    node_query:        Arc<NodeQuery>,
    resource_tracking: Arc<ResourceTracking>,
    function_tracking: Arc<FunctionTracking>,
    lock_state:        PhantomData<State>,
    lock:              Arc<Semaphore>,
    permit:            Option<OwnedSemaphorePermit>,
}

#[cfg(not(any(
    feature = "edge_first",
    feature = "cloud_only",
    feature = "edge_ward"
)))]
mod auction_placement;
#[cfg(not(any(
    feature = "edge_first",
    feature = "cloud_only",
    feature = "edge_ward"
)))]
pub use auction_placement::*;

#[cfg(feature = "cloud_only")]
mod cloud_only_placement;
#[cfg(feature = "cloud_only")]
pub use cloud_only_placement::*;

#[cfg(feature = "edge_first")]
mod edge_first_placement;
#[cfg(feature = "edge_first")]
pub use edge_first_placement::*;

#[cfg(feature = "edge_ward")]
mod edge_ward_placement;
#[cfg(feature = "edge_ward")]
pub use edge_ward_placement::*;

impl FunctionLife {
    pub fn new(
        function: Arc<FaaSBackend>,
        auction: Arc<Auction>,
        node_situation: Arc<NodeSituation>,
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        resource_tracking: Arc<ResourceTracking>,
        function_tracking: Arc<FunctionTracking>,
    ) -> Self {
        #[cfg(feature = "edge_first")]
        {
            info!("Using edge-first placement");
        }
        #[cfg(feature = "cloud_only")]
        {
            info!("Using cloud-only placement");
        }
        #[cfg(feature = "edge_ward")]
        {
            info!("Using edge-ward placement");
        }
        #[cfg(not(any(
            feature = "edge_first",
            feature = "cloud_only",
            feature = "edge_ward"
        )))]
        {
            info!("Using auction (default) placement");
        }
        Self {
            function,
            auction,
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

    pub async fn lock(&self) -> Result<FunctionLife<Locked>, Error> {
        trace!("Locking FunctionLife Service...");
        let permit = Some(
            self.lock
                .clone()
                .acquire_owned()
                .await
                .map_err(|_| Error::Lock)?,
        );

        Ok(FunctionLife {
            function: self.function.clone(),
            auction: self.auction.clone(),
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

impl<State> Drop for FunctionLife<State> {
    fn drop(&mut self) {
        trace!("Unlocking FunctionLife Service after drop...");
        let permit = std::mem::replace(&mut self.permit, None);
        let Some(permit) =  permit else {
                return;
            };
        drop(permit);
    }
}

impl FunctionLife<Locked> {
    pub async fn validate_bid_and_provision_function(
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
                warn!("Could not get tracked cpu and memory");
                return Ok(());
            };
        let Ok(()) = self
                .resource_tracking
                .set_used(name, memory + sla_memory, cpu + sla_cpu)
                .await else {
                    warn!("Could not set updated tracked cpu and memory");
                    return Ok(());
                };
        Ok(())
    }

    pub fn unlock(mut self) -> FunctionLife<Unlocked> {
        trace!("Unlocking FunctionLife Service...");
        let permit = std::mem::replace(&mut self.permit, None);
        if let Some(permit) = permit {
            drop(permit);
        };

        FunctionLife {
            function:          self.function.clone(),
            auction:           self.auction.clone(),
            node_situation:    self.node_situation.clone(),
            neighbor_monitor:  self.neighbor_monitor.clone(),
            node_query:        self.node_query.clone(),
            resource_tracking: self.resource_tracking.clone(),
            function_tracking: self.function_tracking.clone(),
            lock_state:        PhantomData,
            lock:              self.lock.clone(),
            permit:            None,
        }
    }
}
