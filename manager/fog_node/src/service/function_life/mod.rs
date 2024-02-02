use crate::repository::cron::Cron;
use crate::repository::faas::FunctionTimeout;
use crate::repository::function_tracking::FunctionTracking;
use crate::service::auction::Auction;
#[cfg(any(
    feature = "auction",
    feature = "edge_first",
    feature = "edge_first_v2",
    feature = "edge_ward_v2",
    feature = "edge_ward_v3",
))]
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation, FUNCTION_LIVE_TIMEOUT_MSECS};
use anyhow::{Context, Result};
use backoff::exponential::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::SystemClock;
use helper::env_load;
use model::{BidId, SlaId};
use std::sync::Arc;

pub struct FunctionLife {
    function:              Arc<Function>,
    auction:               Arc<Auction>,
    node_situation:        Arc<NodeSituation>,
    #[cfg(any(
        feature = "auction",
        feature = "edge_first",
        feature = "edge_first_v2",
        feature = "edge_ward_v2",
        feature = "edge_ward_v3",
    ))]
    neighbor_monitor:      Arc<NeighborMonitor>,
    node_query:            Arc<NodeQuery>,
    function_tracking:     Arc<FunctionTracking>,
    cron:                  Arc<Cron>,
    function_live_timeout: Arc<std::time::Duration>,
}

#[cfg(feature = "auction")]
mod auction_placement;
#[cfg(feature = "auction")]
pub use auction_placement::*;

// #[cfg(feature = "cloud_only")]
// mod cloud_only_placement;
// #[cfg(feature = "cloud_only")]
// pub use cloud_only_placement::*;

// #[cfg(feature = "cloud_only_v2")]
// mod cloud_only_placement_v2;
// #[cfg(feature = "cloud_only_v2")]
// pub use cloud_only_placement_v2::*;

#[cfg(feature = "edge_first")]
mod edge_first_placement;
#[cfg(feature = "edge_first")]
pub use edge_first_placement::*;

#[cfg(feature = "edge_first_v2")]
mod edge_first_placement_v2;
#[cfg(feature = "edge_first_v2")]
pub use edge_first_placement_v2::*;

#[cfg(feature = "edge_ward")]
mod edge_ward_placement;
#[cfg(feature = "edge_ward")]
pub use edge_ward_placement::*;

#[cfg(feature = "edge_ward_v2")]
mod edge_ward_placement_v2;
#[cfg(feature = "edge_ward_v2")]
pub use edge_ward_placement_v2::*;

#[cfg(feature = "edge_ward_v3")]
mod edge_ward_placement_v3;
#[cfg(feature = "edge_ward_v3")]
pub use edge_ward_placement_v3::*;

use super::function::Function;

impl FunctionLife {
    pub fn new(
        function: Arc<Function>,
        auction: Arc<Auction>,
        node_situation: Arc<NodeSituation>,
        #[cfg(any(
            feature = "auction",
            feature = "edge_first",
            feature = "edge_first_v2",
            feature = "edge_ward_v2",
            feature = "edge_ward_v3",
        ))]
        neighbor_monitor: Arc<NeighborMonitor>,
        node_query: Arc<NodeQuery>,
        function_tracking: Arc<FunctionTracking>,
        cron: Arc<Cron>,
    ) -> Result<Self> {
        #[cfg(feature = "edge_first")]
        {
            info!("Using edge-first placement");
        }
        #[cfg(feature = "edge_first_v2")]
        {
            info!("Using edge-first v2 placement");
        }
        #[cfg(feature = "cloud_only")]
        {
            info!("Using cloud-only placement");
        }
        #[cfg(feature = "cloud_only_v2")]
        {
            info!("Using cloud-only v2 placement");
        }
        #[cfg(feature = "edge_ward")]
        {
            info!("Using edge-ward placement");
        }
        #[cfg(feature = "edge_ward_placement_v2")]
        {
            info!("Using edge-ward v2 placement");
        }
        #[cfg(feature = "edge_ward_placement_v3")]
        {
            info!("Using edge-ward v3 placement");
        }
        #[cfg(feature = "auction")]
        {
            info!("Using auction placement");
        }

        let function_live_timeout =
            env_load!(FunctionTimeout, FUNCTION_LIVE_TIMEOUT_MSECS, u64);
        let function_live_timeout =
            Arc::new(std::time::Duration::from_millis(
                function_live_timeout.into_inner(),
            ));
        Ok(Self {
            function,
            auction,
            node_situation,
            #[cfg(any(
                feature = "auction",
                feature = "edge_first",
                feature = "edge_first_v2",
                feature = "edge_ward_v2",
                feature = "edge_ward_v3",
            ))]
            neighbor_monitor,
            node_query,
            function_tracking,
            cron,
            function_live_timeout,
        })
    }

    pub async fn provision_function(&self, id: SlaId) -> Result<()> {
        let function = self.function.lock().await?;
        function.provision_function(id.clone()).await?;

        drop(function);

        backoff::future::retry(self.get_backoff(), || async {
            trace!("Checking if function is alive");
            self.function.check_and_set_function_is_live(id.clone()).await?;
            Ok(())
        })
        .await?;

        let live =
            self.function_tracking.get_live(&id).with_context(|| {
                format!("Failed to get data about the live function {}", id)
            })?;

        let function = self.function.clone();
        let id2 = id.clone();
        self.cron
            .add_oneshot(live.0.sla.duration, move || {
                let id = id.clone();
                let function = function.clone();
                Box::pin(async move {
                    let Ok(function) = function
                        .lock()
                        .await
                        .context(
                            "Failed to lock when calling cron to unprovision \
                             function",
                        )
                        .map_err(|err| error!("{:?}", err))
                    else {
                        return;
                    };

                    if let Err(err) = function.unprovision_function(id).await {
                        warn!(
                            "Failed to unprovision function as stated in the \
                             cron job {:?}",
                            err
                        );
                    }
                })
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to setup a oneshot cron job to stop function {} \
                     in the future",
                    id2
                )
            })?;
        Ok(())
    }

    fn get_backoff(&self) -> ExponentialBackoff<SystemClock> {
        let backoff = ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some((*self.function_live_timeout).clone()))
            .build();

        backoff
    }
}
