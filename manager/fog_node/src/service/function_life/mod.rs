use std::sync::Arc;

use crate::repository::cron::{Cron, Task, UnprovisionFunction};
use crate::repository::faas::FunctionTimeout;
use crate::repository::function_tracking::FunctionTracking;
use crate::service::auction::Auction;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation, FUNCTION_LIVE_TIMEOUT_MSECS};
use anyhow::{Context, Result};
use backoff::exponential::{ExponentialBackoff, ExponentialBackoffBuilder};
use backoff::SystemClock;
use helper::env_load;
use model::view::auction::AccumulatedLatency;
use model::SlaId;
use tracing::{error, info, instrument, trace, warn};
use uom::si::f64::Time;
use uom::si::information::byte;
use uom::si::ratio::ratio;
use uom::si::rational64::Information;
use uom::si::time::second;

pub struct FunctionLife {
    function:              Arc<Function>,
    auction:               Arc<Auction>,
    node_situation:        Arc<NodeSituation>,
    #[allow(dead_code)]
    neighbor_monitor:      Arc<NeighborMonitor>,
    node_query:            Arc<NodeQuery>,
    #[allow(dead_code)]
    function_tracking:     Arc<FunctionTracking>,
    cron:                  Arc<Cron>,
    function_live_timeout: Arc<std::time::Duration>,
}

#[cfg(feature = "auction")]
mod auction_placement;

#[cfg(feature = "maxcpu")]
mod maxcpu;

#[cfg(feature = "mincpurandom")]
mod mincpurandom;

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

#[cfg(feature = "edge_furthest")]
mod edge_furthest_placement;

#[cfg(feature = "edge_ward")]
mod edge_ward_placement;

#[cfg(feature = "edge_ward_v2")]
mod edge_ward_placement_v2;

#[cfg(feature = "edge_ward_v3")]
mod edge_ward_placement_v3;
use super::function::Function;
#[allow(dead_code)]
const DEFAULT_MTU: f64 = 1500.0;
const TCP_MAX_CONGESTION_WINDOW_SIZE: f64 = 33.0; // In terms of number of MSS
#[allow(dead_code)]
#[allow(dead_code)]
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
        #[cfg(feature = "edge_furthest")]
        {
            info!("Using edge furthest placement");
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
        #[cfg(feature = "edge_ward_v2")]
        {
            info!("Using edge-ward v2 placement");
        }
        #[cfg(feature = "edge_ward_v3")]
        {
            info!("Using edge-ward v3 placement");
        }
        #[cfg(feature = "auction")]
        {
            info!("Using auction placement");
        }
        #[cfg(feature = "maxcpu")]
        {
            info!("Using maxcpu placement");
        }
        #[cfg(feature = "mincpurandom")]
        {
            info!("Using mincpurandom placement");
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

    #[instrument(level = "trace", skip(self))]
    pub async fn pay_function(&self, id: SlaId) -> Result<()> {
        let function = self.function.lock().await?;
        let paid = function.pay_function(id.clone()).await?; // Function is now in the system
                                                             //drop(function);

        let function = self.function.clone();
        let node = paid.node;
        let task = Task::UnprovisionFunction(UnprovisionFunction {
            sla: id.clone(),
            node,
        });
        let id2 = id.clone();

        self.cron
            .add_oneshot(paid.sla.duration, task, move || {
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
                        #[cfg(test)]
                        panic!(
                            "Failed to lock function service to unprovision \
                             function"
                        );

                        #[cfg(not(test))]
                        return;
                    };

                    if let Err(err) = function.finish_function(id).await {
                        warn!(
                            "Failed to drop paid function as stated in the \
                             cron job {:?}",
                            err
                        );

                        #[cfg(test)]
                        panic!(
                            "Failed to finish function when unprovisioning \
                             function: {}",
                            err.to_string()
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

    #[instrument(level = "trace", skip(self))]
    pub async fn provision_function(&self, id: SlaId) -> Result<()> {
        let function = self.function.lock().await?;
        function.provision_function(id.clone()).await?;
        drop(function);

        let res = backoff::future::retry(self.get_backoff(), || async {
            trace!("Checking if function is alive");
            self.function.check_and_set_function_is_live(id.clone()).await?;
            Ok(())
        })
        .await;

        if res.is_err() {
            warn!("Failed to check that the function is alive");
            let function = self.function.lock().await?;
            function.finish_function(id).await?;
        }
        res
    }

    fn get_backoff(&self) -> ExponentialBackoff<SystemClock> {
        let backoff = ExponentialBackoffBuilder::default()
            .with_max_elapsed_time(Some((*self.function_live_timeout).clone()))
            .build();

        backoff
    }

    #[allow(dead_code)]
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

#[allow(dead_code)]
fn get_tcp_latency(
    rtt: Time,
    packet_loss: uom::si::f64::Ratio,
    data_size: Information,
) -> Time {
    let data_size = *data_size.get::<byte>().numer() as f64
        / *data_size.get::<byte>().denom() as f64;
    let data_size = uom::si::f64::Information::new::<byte>(data_size);
    let mtu = uom::si::f64::Information::new::<byte>(DEFAULT_MTU);
    let mss = mtu - uom::si::f64::Information::new::<byte>(40.0); // mtu - <tcp
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
    let hand2 = 1.0
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
    use uom::si::f64::{Ratio, Time};
    use uom::si::information::{byte, kilobyte};
    use uom::si::ratio::ratio;
    use uom::si::rational64::Information;
    use uom::si::time::millisecond;

    use crate::service::function_life::get_tcp_latency;
    #[test]
    fn test_under_mtu_latency() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size =
            Information::new::<byte>(num_rational::Ratio::new(1000, 1));
        let packet_loss = Ratio::new::<ratio>(1e-7);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(under)lat: {:?}", lat);
        assert!(lat == 1.5 * rtt);
    }
    #[test]
    fn test_over_mtu_latency() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size =
            Information::new::<kilobyte>(num_rational::Ratio::new(100, 1));
        let packet_loss = Ratio::new::<ratio>(0.0);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);
        println!("(over)lat: {:?}", lat);
        assert!(lat > 2.0 * rtt);
    }

    #[test]
    fn test_over_mtu_latency_loss() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size =
            Information::new::<kilobyte>(num_rational::Ratio::new(100, 1));
        let packet_loss = Ratio::new::<ratio>(0.01);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(over loss)lat: {:?}", lat);
        assert!(lat > 3.0 * rtt);
    }
    #[test]
    fn test_under_mtu_latency_loss() {
        let rtt = Time::new::<millisecond>(20.0);
        let data_size =
            Information::new::<byte>(num_rational::Ratio::new(100, 1));
        let packet_loss = Ratio::new::<ratio>(0.01);
        let lat = get_tcp_latency(rtt, packet_loss, data_size);

        println!("(under loss)lat: {:?}", lat);
        assert!(lat == 1.5 * rtt);
    }
}
