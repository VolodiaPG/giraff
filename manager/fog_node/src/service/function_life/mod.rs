use crate::repository::cron::Cron;
use crate::repository::function_tracking::FunctionTracking;
use crate::service::auction::Auction;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use model::domain::sla::Sla;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::{BidId, NodeId};
use std::sync::Arc;
use uom::si::f64::Time;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    Function(#[from] super::function::Error),
    #[error(transparent)]
    NodeQuery(#[from] crate::repository::node_query::Error),
    #[error("BidId {0} cannot be retrieved")]
    MissingBidId(BidId),
    #[error(transparent)]
    Cron(#[from] crate::repository::cron::Error),
    #[cfg(any(feature = "edge_first", feature = "edge_ward"))]
    #[error("No candidates were found or returned an Ok result")]
    NoCandidatesRetained,
    #[cfg(feature = "cloud_only")]
    #[error("No cloud was found or returned an Ok result")]
    NoCloudAvailable,
}

pub struct FunctionLife {
    function:          Arc<Function>,
    auction:           Arc<Auction>,
    node_situation:    Arc<NodeSituation>,
    neighbor_monitor:  Arc<NeighborMonitor>,
    node_query:        Arc<NodeQuery>,
    function_tracking: Arc<FunctionTracking>,
    cron:              Arc<Cron>,
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

use super::function::Function;

impl FunctionLife {
    pub fn new(
        function: Arc<Function>,
        auction: Arc<Auction>,
        node_situation: Arc<NodeSituation>,
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        function_tracking: Arc<FunctionTracking>,
        cron: Arc<Cron>,
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
            function_tracking,
            cron,
        }
    }

    pub async fn provision_function(&self, id: BidId) -> Result<(), Error> {
        let function = self.function.lock().await?;
        function.provision_function(id.clone()).await?;

        let provisioned = self
            .function_tracking
            .get_provisioned(&id)
            .ok_or_else(|| Error::MissingBidId(id.clone()))?;

        drop(function);

        let function = self.function.clone();
        self.cron.add_oneshot(provisioned.0.sla.duration, move || {
           let id = id.clone();
           let function = function.clone();
            Box::pin(async move {
            let Ok(function) = function.lock().await else {
                warn!("Failed to lock when calling cron to unprovision function");
                return;
            };
            if function.unprovision_function(id).await.is_err(){
                warn!("Failed to unprovision function");
            }
        })
        }).await?;
        Ok(())
    }
}
