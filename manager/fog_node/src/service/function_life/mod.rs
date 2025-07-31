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
use num_traits::ToPrimitive;
use tracing::{error, info, instrument, trace, warn};
use uom::si::f64::Time;
use uom::si::information::byte;
use uom::si::information_rate::byte_per_second;
use uom::si::ratio::ratio;
use uom::si::rational64::Information;
use uom::si::time::{millisecond, second};

pub struct FunctionLife {
    function: Arc<Function>,
    auction: Arc<Auction>,
    node_situation: Arc<NodeSituation>,
    #[allow(dead_code)]
    neighbor_monitor: Arc<NeighborMonitor>,
    node_query: Arc<NodeQuery>,
    #[allow(dead_code)]
    function_tracking: Arc<FunctionTracking>,
    cron: Arc<Cron>,
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
// MSS
const DEFAULT_TCP_MSS: f64 = 1460.0; // In terms of number of MSS
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

    async fn prov(&self, id: SlaId) -> Result<()> {
        let function = self.function.lock().await?;
        function.provision_function(id).await
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn provision_function(&self, id: SlaId) -> Result<()> {
        self.prov(id.clone()).await?;

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
            accumulated_latency_to_next_node.median
                + accumulated_latency_to_next_node.median_uncertainty,
            accumulated_latency_to_next_node.packet_loss,
            accumulated_latency_to_next_node.bandwidth,
            data_size,
        );
        trace!(
            "compute_worse_latency: {:?}, data: {:?}, bandwidth: {:?}",
            worse,
            data_size,
            accumulated_latency_to_next_node.bandwidth
        );
        return worse;
    }
}

//#[allow(dead_code)]
//#[instrument(level = "trace")]
//fn get_tcp_latency(
//    one_way_latency: Time,
//    _packet_loss: uom::si::f64::Ratio,
//    bandwidth: uom::si::rational64::InformationRate,
//    data_size: Information,
//) -> Time {
//    let data_size = *data_size.get::<byte>().numer() as f64
//        / *data_size.get::<byte>().denom() as f64;
//    let bandwidth = *bandwidth.get::<byte_per_second>().numer() as f64
//        / *bandwidth.get::<byte_per_second>().denom() as f64;
//
//    let tcp_overhead = DEFAULT_MTU - DEFAULT_TCP_MSS;
//    let nb_segments = data_size / DEFAULT_TCP_MSS;
//    let data_size_to_send = tcp_overhead * nb_segments + data_size;
//    let pure_data_transmission_time = data_size_to_send / bandwidth;
//
//    Time::new::<second>(pure_data_transmission_time) + one_way_latency
//}
#[allow(dead_code)]
const TCP_MAX_CONGESTION_WINDOW_SIZE: f64 = 33.0; // In terms of number of MSS

#[allow(dead_code)]
#[instrument(level = "trace")]
fn get_tcp_latency(
    one_way: Time,
    packet_loss: uom::si::f64::Ratio,
    bandwidth: uom::si::rational64::InformationRate,
    data_size: Information,
) -> Time {
    // Modeling TCP Throughput: A Simple Model and its Empirical Validation *
    // Jitendra Padhye Victor Firoiu Don Towsley Jim Kurose
    trace!(
        "latency (one_way): {:?} ms",
        one_way.get::<millisecond>().to_f64()
    );
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
    let rtt = 2.0 * one_way;

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
    let ret = transmission_time + 1.5 * rtt;
    trace!("transmission_time: {:?} ms", ret.get::<millisecond>().to_f64());
    let ret = ret * 1.3;
    trace!(
        "transmission_time with 30% err margin: {:?} ms",
        ret.get::<millisecond>().to_f64()
    );

    ret
}
//#[cfg(test)]
//mod tests {
//    use crate::service::function_life::get_tcp_latency;
//    use uom::si::f64::{Ratio, Time};
//    use uom::si::information::byte;
//    use uom::si::information_rate::bit_per_second;
//    use uom::si::ratio::ratio;
//    use uom::si::rational64::{Information, InformationRate};
//    use uom::si::time::millisecond;
//    use yare::parameterized;
//
//    #[parameterized(
//        small_data_big_bandwidth = { 1000, 1_000_000, 1.0, 1.5 },
//        big_data_small_bandwidth = { 10_000, 400_000, 11.0, 12.0 },
//        bigger_data_small_bandwidth = { 260_000, 100_000, 1069.0, 1070.0 },
//        normal_payload = { 48_000, 100_000_0000, 1.0, 1.1 },
//        bigger_data_big_bandwidth = { 260_000, 100_000_000, 2.0, 2.5 },
//        bigger_data_bigger_bandwidth = { 260_000, 1_000_000_000, 1.0, 1.5 },
//    )]
//    fn test_latency_bandwidth(
//        data_size: i64,
//        bandwidth: i64,
//        expected_low: f64,
//        expected_high: f64,
//    ) {
//        let oneway = Time::new::<millisecond>(20.0);
//        let data_size =
//            Information::new::<byte>(num_rational::Ratio::new(data_size, 1));
//        let bandwidth = InformationRate::new::<bit_per_second>(
//            num_rational::Ratio::new(bandwidth, 1),
//        );
//        let packet_loss = Ratio::new::<ratio>(0.0);
//        let lat = get_tcp_latency(oneway, packet_loss, bandwidth, data_size);
//
//        println!("lat: {:?}", lat);
//        assert!(lat > expected_low * oneway);
//        assert!(lat < expected_high * oneway);
//    }
//}
