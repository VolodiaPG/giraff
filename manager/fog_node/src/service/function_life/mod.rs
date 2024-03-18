use crate::repository::cron::Cron;
use crate::repository::faas::FunctionTimeout;
use crate::repository::function_tracking::FunctionTracking;
use crate::service::auction::Auction;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation, FUNCTION_LIVE_TIMEOUT_MSECS};
use anyhow::{anyhow, Context, Result};
use backoff::exponential::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::SystemClock;
use helper::env_load;
use model::view::auction::AccumulatedLatency;
use model::SlaId;
use std::sync::Arc;
use uom::si::f64::{Information, Ratio, Time};
use uom::si::information::byte;
use uom::si::ratio::ratio;
use uom::si::time::second;

pub struct FunctionLife {
    function:              Arc<Function>,
    auction:               Arc<Auction>,
    node_situation:        Arc<NodeSituation>,
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

#[cfg(feature = "powerrandom")]
mod powerrandom;
#[cfg(feature = "powerrandom")]
pub use powerrandom::*;

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

const DEFAULT_MTU: f64 = 1500.0;
const TCP_MAX_CONGESTION_WINDOW_SIZE: f64 = 33.0; // In terms of number of MSS
const TCP_TIMEOUT_SEC: f64 = 0.020;

impl FunctionLife {
    pub fn new(
        function: Arc<Function>,
        auction: Arc<Auction>,
        node_situation: Arc<NodeSituation>,
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
        #[cfg(feature = "powerrandom")]
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
            neighbor_monitor,
            node_query,
            function_tracking,
            cron,
            function_live_timeout,
        })
    }

    pub async fn pay_function(&self, id: SlaId) -> Result<()> {
        let function = self.function.lock().await?;
        function.pay_function(id.clone()).await?;
        drop(function);

        let paid = self
            .function_tracking
            .get_paid(&id)
            .ok_or(anyhow!("Failed to retrieve paid function {}", id))?;
        let function = self.function.clone();
        let id2 = id.clone();
        self.cron
            .add_oneshot(paid.0.sla.duration, move || {
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

                    if let Err(err) = function.drop_paid_function(id).await {
                        warn!(
                            "Failed to drop paid function as stated in the \
                             cron job {:?}",
                            err
                        );
                    }
                })
            })
            .await
            .with_context(|| {
                format!(
                    "Failed to setup a oneshot cron job to stop paid \
                     function {} in the future",
                    id2
                )
            })?;
        Ok(())
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

    fn compute_worse_latency(
        &self,
        accumulated_latency_to_next_node: &AccumulatedLatency,
        data_size: Information,
    ) -> Time {
        let worse = get_tcp_latency(
            2.0 * (accumulated_latency_to_next_node.median
                + accumulated_latency_to_next_node.median_uncertainty),
            accumulated_latency_to_next_node.packet_loss,
            data_size,
        );
        return worse;
    }
}

fn get_tcp_latency(
    rtt: Time,
    packet_loss: Ratio,
    data_size: Information,
) -> Time {
    let mtu = Information::new::<byte>(DEFAULT_MTU);
    let mss = mtu - Information::new::<byte>(40.0); // mtu - <tcp
                                                    // protocol overhead>
                                                    //let data_size = if data_size < mss { mss } else { data_size };
    let mut tcp_max_congestion_window_size = (data_size / mss).get::<ratio>();
    tcp_max_congestion_window_size =
        if tcp_max_congestion_window_size < TCP_MAX_CONGESTION_WINDOW_SIZE {
            tcp_max_congestion_window_size
        } else {
            TCP_MAX_CONGESTION_WINDOW_SIZE
        };
    let tcp_timeout = Time::new::<second>(TCP_TIMEOUT_SEC);
    let packet_loss = packet_loss.get::<ratio>();

    let hand1 = tcp_max_congestion_window_size / rtt;

    let subhand1 = 1.0;
    let subhand2 = 3.0 * f64::sqrt(6.0 * packet_loss / 8.0);
    let subhand = if subhand1 < subhand2 { subhand1 } else { subhand2 };
    let mut hand2 = 1.0
        / (f64::sqrt(packet_loss * 4.0 / 3.0) * rtt
            + tcp_timeout
                * subhand
                * packet_loss
                * (1.0 + 32.0 * packet_loss * packet_loss));
    // going to infity and buffer overflow
    let tcp_bandwidth = if hand1 < hand2 { hand1 } else { hand2 };
    let tcp_bandwidth = tcp_bandwidth * mss;

    let transmission_time = data_size / tcp_bandwidth;
    transmission_time + 0.5 * rtt
}
#[cfg(test)]
mod tests {
    use uom::si::f64::{Information, Ratio, Time};
    use uom::si::information::{byte, kilobyte};
    use uom::si::ratio::ratio;
    use uom::si::time::millisecond;

    use crate::service::function_life::get_tcp_latency;
    #[test]
    fn test_under_mtu_latency() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size = Information::new::<byte>(1000.0);
        let packet_loss = Ratio::new::<ratio>(1e-7);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(under)lat: {:?}", lat);
        assert!(lat == 1.5 * rtt);
    }
    #[test]
    fn test_over_mtu_latency() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size = Information::new::<kilobyte>(100.0);
        let packet_loss = Ratio::new::<ratio>(0.0);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);
        println!("(over)lat: {:?}", lat);
        assert!(lat > 2.0 * rtt);
    }

    #[test]
    fn test_over_mtu_latency_loss() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size = Information::new::<kilobyte>(100.0);
        let packet_loss = Ratio::new::<ratio>(0.01);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(over loss)lat: {:?}", lat);
        assert!(lat > 3.0 * rtt);
    }
    #[test]
    fn test_under_mtu_latency_loss() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size = Information::new::<byte>(100.0);
        let packet_loss = Ratio::new::<ratio>(0.01);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(under loss)lat: {:?}", lat);
        assert!(lat == 1.5 * rtt);
    }
}
