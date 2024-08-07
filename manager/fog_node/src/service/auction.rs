use crate::monitoring::BidGauge;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use anyhow::{bail, Context, Result};
use chrono::{Timelike, Utc};
use helper::env_var;
use helper::monitoring::MetricsExporter;
use model::domain::sla::Sla;
use model::dto::function::Proposed;
use model::dto::node::MaxInFlight;
use model::view::auction::AccumulatedLatency;
use model::BidId;
use nutype::nutype;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;
use tracing::{instrument, trace};
use uom::num_traits::ToPrimitive;
use uom::si::rational64::{Information, Ratio};
use uuid::Uuid;

use super::function::Function;

#[nutype(
    derive(PartialEq, PartialOrd),
    validate(finite, greater_or_equal = 0.0)
)]
pub struct PricingRatio(f64);
env_var!(PRICING_CPU);
env_var!(PRICING_CPU_INITIAL);
env_var!(PRICING_MEM);
env_var!(PRICING_MEM_INITIAL);
env_var!(PRICING_GEOLOCATION);
env_var!(RATIO_AA);
env_var!(RATIO_BB);
env_var!(ELECTRICITY_PRICE);
struct ComputedBid {
    pub(crate) name:          String,
    #[allow(dead_code)]
    pub(crate) available_ram: Information,
    #[allow(dead_code)]
    pub(crate) available_cpu: Ratio,
    #[allow(dead_code)]
    pub(crate) used_ram:      Information,
    #[allow(dead_code)]
    pub(crate) used_cpu:      Ratio,
    pub(crate) bid:           f64,
}

pub struct Auction {
    resource_tracking:             Arc<ResourceTracking>,
    db:                            Arc<FunctionTracking>,
    metrics:                       Arc<MetricsExporter>,
    #[allow(dead_code)]
    function:                      Arc<Function>,
    in_flight_functions_per_sec_1: AtomicU32,
    in_flight_functions_per_sec_2: AtomicU32,
    max_in_flight:                 MaxInFlight,
}

impl Auction {
    pub fn new(
        resource_tracking: Arc<ResourceTracking>,
        db: Arc<FunctionTracking>,
        metrics: Arc<MetricsExporter>,
        function: Arc<Function>,
        max_in_flight_functions_proposals: MaxInFlight,
    ) -> Result<Self> {
        Ok(Self {
            resource_tracking,
            db,
            metrics,
            function,
            in_flight_functions_per_sec_1: AtomicU32::new(0),
            in_flight_functions_per_sec_2: AtomicU32::new(0),
            max_in_flight: max_in_flight_functions_proposals,
        })
    }

    /// Get a suitable (free enough) node to potentially run the designated SLA
    #[instrument(level = "trace", skip(self, sla))]
    async fn get_a_node(
        &self,
        sla: &Sla,
    ) -> Result<Option<(String, Information, Ratio, Information, Ratio)>> {
        for node in self.resource_tracking.get_nodes() {
            let (used_ram, used_cpu) =
                self.resource_tracking.get_used(node).await.with_context(
                    || {
                        format!(
                            "Failed to get used resources from tracking data \
                             for node {}",
                            node
                        )
                    },
                )?;
            let (available_ram, available_cpu) = self
                .resource_tracking
                .get_available(node)
                .await
                .with_context(|| {
                    format!(
                        "Failed to get available resources from tracking \
                         data for node {}",
                        node
                    )
                })?;
            if super::function::satisfiability_check(
                &used_ram,
                &used_cpu,
                &available_ram,
                &available_cpu,
                sla,
            ) {
                return Ok(Some((
                    node.clone(),
                    used_ram,
                    used_cpu,
                    available_ram,
                    available_cpu,
                )));
            }
        }
        Ok(None)
    }

    #[cfg(feature = "linear_rates")]
    #[instrument(level = "trace", skip(self, sla, _accumulated_latency))]
    async fn compute_bid(
        &self,
        sla: &Sla,
        _accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<ComputedBid>> {
        use helper::env_load;

        // let pricing_cpu =
        //     env_load!(PricingRatio, PRICING_CPU, f64).into_inner();
        let pricing_cpu_initial =
            env_load!(PricingRatio, PRICING_CPU_INITIAL, f64).into_inner();
        // let pricing_mem =
        //     env_load!(PricingRatio, PRICING_MEM, f64).into_inner();
        let pricing_mem_initial =
            env_load!(PricingRatio, PRICING_MEM_INITIAL, f64).into_inner();

        let Some((name, used_ram, used_cpu, available_ram, available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")?
        else {
            return Ok(None);
        };

        let ram_ratio_sla = (sla.memory / available_ram)
            .get()
            .to_f64()
            .context("Overflow while converting ratio of memory")?;
        let cpu_ratio_sla = (sla.cpu / available_cpu)
            .to_f64()
            .context("Overflow while converting ratio of cpu")?;
        let bid: f64 = ram_ratio_sla * pricing_mem_initial
            + cpu_ratio_sla * pricing_cpu_initial;

        trace!("price on {:?} is {:?}", name, bid);

        Ok(Some(ComputedBid {
            name,
            bid,
            available_cpu,
            available_ram,
            used_cpu,
            used_ram,
        }))
    }

    #[cfg(feature = "quadratic_rates")]
    #[instrument(level = "trace", skip(self, sla, _accumulated_latency))]
    async fn compute_bid(
        &self,
        sla: &Sla,
        _accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<ComputedBid>> {
        use crate::service::function::UnprovisionEvent;
        use chrono::Duration;
        use helper::env_load;
        use helper::uom_helper::cpu_ratio::cpu;
        use uom::si::time::second;

        let Some((name, used_ram, used_cpu, available_ram, available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")?
        else {
            return Ok(None);
        };

        let aa = env_load!(PricingRatio, RATIO_AA, f64).into_inner();
        let bb = env_load!(PricingRatio, RATIO_BB, f64).into_inner();
        let now = Utc::now();
        let mut utilisation = 0.0;
        for UnprovisionEvent { timestamp, sla, node, .. } in
            self.function.get_utilisation_variations().await.iter()
        {
            let duration = if *timestamp > now {
                *timestamp - now
            } else {
                Duration::microseconds(0)
            };
            let (_available_ram, available_cpu) = self
                .resource_tracking
                .get_available(node)
                .await
                .with_context(|| {
                    format!(
                        "Failed to get available resources from tracking \
                         data for node {}",
                        node
                    )
                })?;

            let duration = duration.num_seconds() as f64;
            utilisation += (sla.cpu / available_cpu)
                .get::<cpu>()
                .to_f64()
                .context("Overlfow while converting to f64")?
                * duration;
        }
        let sla_cpu = (sla.cpu / available_cpu)
            .get::<cpu>()
            .to_f64()
            .context("Overflow during conversion to f64")?;
        let sla_duration = sla.duration.get::<second>();
        let electricity_price =
            env_load!(PricingRatio, ELECTRICITY_PRICE, f64).into_inner();
        let bid = electricity_price
            * sla_cpu
            * (2.0 * aa * utilisation + (aa * sla_cpu + bb) * sla_duration);

        trace!("(quadratic) price on is {:?}", bid);
        assert!(bid > 0.0, "the bid wasn't > 0");

        Ok(Some(ComputedBid {
            name,
            bid,
            available_cpu,
            available_ram,
            used_cpu,
            used_ram,
        }))
    }

    #[cfg(any(feature = "maxcpu", feature = "mincpurandom"))]
    #[instrument(level = "trace", skip(self, sla))]
    async fn compute_bid_cpu(
        &self,
        sla: &Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(String, f64, f64)>> {
        use uom::si::ratio::basis_point;

        match self.compute_bid(sla, accumulated_latency).await? {
            Some(computed) => {
                // The more the cpu is used the lower the price and the easiest
                // to win
                let cpu_ratio_sla = computed.used_cpu / computed.available_cpu;
                let bid = cpu_ratio_sla
                    .get::<basis_point>()
                    .to_f64()
                    .context("Overflow during bid conversion to f64")?;
                // The normal "bid" is also the price usually, but not in that
                // valuation method
                let price = computed.bid;

                trace!("(random) price on {:?} is {:?}", computed.name, bid);
                Ok(Some((computed.name, bid, price)))
            }
            None => Ok(None),
        }
    }

    #[instrument(level = "trace", skip(self))]
    pub async fn bid_on(
        &self,
        sla: Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(BidId, Proposed)>> {
        #[cfg(not(any(feature = "maxcpu", feature = "mincpurandom")))]
        let Some(ComputedBid { name, bid, .. }) = self
            .compute_bid(&sla, accumulated_latency)
            .await
            .context("Failed to compute bid for sla")?
        else {
            return Ok(None);
        };

        if self.check_in_flight().await.is_err() {
            return Ok(None);
        }

        #[cfg(not(any(feature = "maxcpu", feature = "mincpurandom")))]
        let price = bid;

        #[cfg(any(feature = "maxcpu", feature = "mincpurandom"))]
        let Some((name, bid, price)) = self
            .compute_bid_cpu(&sla, accumulated_latency)
            .await
            .context("Failed to compute bid for sla")?
        else {
            return Ok(None);
        };
        let node = name;
        let record = Proposed::new(bid, sla, node);
        self.db.insert(record.clone());
        let id = Uuid::new_v4();
        let id = BidId::from(id);

        assert!(bid >= 0.0, "Proposed bid is negative");
        assert!(price > 0.0, "Proposed price is negative");
        self.metrics
            .observe(BidGauge {
                bid,
                price,
                function_name: record.sla.function_live_name.clone(),
                sla_id: record.sla.id.to_string(),
                bid_id: id.to_string(),
                timestamp: Utc::now(),
            })
            .await?;

        Ok(Some((id, record)))
    }

    /// Allow and increase the counter or refuses
    async fn check_in_flight(&self) -> Result<()> {
        let current_second = (Utc::now().second() % 2) as usize; // 0 or 1
        let inc;
        let err;

        if current_second == 0 {
            inc = self
                .in_flight_functions_per_sec_1
                .fetch_add(1, Ordering::Relaxed);
            err = self.in_flight_functions_per_sec_2.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |_| Some(0),
            );
        } else {
            inc = self
                .in_flight_functions_per_sec_2
                .fetch_add(1, Ordering::Relaxed);
            err = self.in_flight_functions_per_sec_1.fetch_update(
                Ordering::Relaxed,
                Ordering::Relaxed,
                |_| Some(0),
            );
        };

        if err.is_err() {
            bail!("Failed to update atomic");
        }

        let size = self.max_in_flight.clone().into_inner();
        if (inc as usize) > size {
            bail!("Too many in flight functions I have bidded upon")
        }

        Ok(())
    }
}

#[cfg(feature = "offline")]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::repository::cron::Cron;
    use crate::repository::faas::{FaaSBackend, FaaSBackendOfflineImpl};
    use crate::repository::k8s::{K8s, OFFLINE_NODE_K8S};
    use crate::repository::latency_estimation::{
        LatencyEstimation, LatencyEstimationOfflineImpl,
    };
    use crate::repository::node_query::NodeQuery;
    use crate::repository::node_situation::NodeSituation;
    use crate::service::function_life::FunctionLife;
    use crate::service::neighbor_monitor::NeighborMonitor;
    use helper::monitoring::InfluxAddress;
    use helper::uom_helper::cpu_ratio::{cpu, millicpu};
    use model::dto::node::{NodeCategory, NodeSituationData};
    use model::SlaId;
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use rand_distr::{Distribution, Uniform};
    use std::net::Ipv4Addr;
    use std::sync::atomic::AtomicU64;
    use std::time::Duration;
    use uom::si::f64::Time;
    use uom::si::information::{gigabyte, megabyte};
    use uom::si::rational64::{Information, Ratio};
    use uom::si::time::second;
    use yare::parameterized;

    pub struct AtomicF64 {
        storage: AtomicU64,
    }
    impl AtomicF64 {
        pub fn new(value: f64) -> Self {
            let as_u64 = value.to_bits();
            Self { storage: AtomicU64::new(as_u64) }
        }

        pub fn store(&self, value: f64, ordering: Ordering) {
            let as_u64 = value.to_bits();
            self.storage.store(as_u64, ordering)
        }

        pub fn load(&self, ordering: Ordering) -> f64 {
            let as_u64 = self.storage.load(ordering);
            f64::from_bits(as_u64)
        }
    }

    struct Instance {
        pub auction:           Arc<Auction>,
        pub function_life:     Arc<FunctionLife>,
        pub function_tracking: Arc<FunctionTracking>,
        pub resource_tracking: Arc<ResourceTracking>,
        pub reserved_cpu:      Ratio,
        pub reserved_memory:   Information,
        pub function:          Arc<Function>,
    }

    fn market_connected() -> NodeCategory {
        NodeCategory::MarketConnected {
            market_ip:   std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            market_port: 1234.into(),
        }
    }

    fn node_connected() -> NodeCategory {
        NodeCategory::NodeConnected {
            parent_latency:        Time::new::<second>(15.0),
            parent_id:             Uuid::new_v4().into(),
            parent_node_ip:        std::net::IpAddr::V4(Ipv4Addr::new(
                127, 0, 0, 1,
            )),
            parent_node_port_http: 1234.into(),
        }
    }

    async fn get_auction_impl(situation: NodeCategory) -> Instance {
        let k8s = Arc::new(K8s::new());
        let metrics = Arc::new(
            MetricsExporter::new(
                InfluxAddress::try_new("127.0.0.1:1234").unwrap(),
                helper::monitoring::InfluxOrg::try_new("toto").unwrap(),
                helper::monitoring::InfluxToken::try_new("xowyTh1iGcNAZsZeydESOHKvENvcyPaWg8hUe3tO4vPOw_buZVwOdUrqG3gwV314aYd9SWKHcxlykcQY_rwYVQ==").unwrap(),
                helper::monitoring::InfluxBucket::try_new("toto").unwrap(),
                helper::monitoring::InstanceName::try_new("toto").unwrap(),
            )
            .await
            .unwrap(),
        );
        let reserved_cpu = Ratio::new::<cpu>(num_rational::Ratio::new(20, 1));
        let reserved_memory =
            Information::new::<gigabyte>(num_rational::Ratio::new(8, 1));
        let resource_tracking = Arc::new(
            ResourceTracking::new(
                k8s,
                metrics.clone(),
                reserved_cpu,
                reserved_memory,
            )
            .await
            .unwrap(),
        );
        let backend: Arc<Box<dyn FaaSBackend>> = Arc::new(Box::new(
            FaaSBackendOfflineImpl::new(Duration::from_secs(2)),
        ));
        let node_situation = Arc::new(NodeSituation::new(NodeSituationData {
            situation,
            my_id: Uuid::new_v4().into(),
            my_public_ip: std::net::IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)),
            my_public_port_http: 12345.into(),
            my_public_port_faas: 1234.into(),
            tags: vec!["toto".to_string()],
            reserved_memory,
            reserved_cpu,
            max_in_flight_functions_proposals: MaxInFlight::try_new(160)
                .unwrap(),
            children: dashmap::DashMap::new(),
        }));
        let latency_estimation_repo: Arc<Box<dyn LatencyEstimation>> =
            Arc::new(Box::new(LatencyEstimationOfflineImpl::new(
                node_situation.clone(),
            )));

        let neighbor_monitor =
            Arc::new(NeighborMonitor::new(latency_estimation_repo));

        let client_builder = reqwest_middleware::ClientBuilder::new(
            reqwest::Client::builder().build().unwrap(),
        );
        let http_client = Arc::new(client_builder.build());

        let node_query = Arc::new(NodeQuery::new(
            node_situation.clone(),
            http_client.clone(),
        ));

        let function_tracking = Arc::new(FunctionTracking::default());
        let cron = Arc::new(
            Cron::new(uom::si::f64::Time::new::<uom::si::time::second>(15.0))
                .await
                .expect("Failed to start Cron repository"),
        );

        let function = Arc::new(Function::new(
            backend.clone(),
            node_situation.clone(),
            neighbor_monitor.clone(),
            node_query.clone(),
            resource_tracking.clone(),
            function_tracking.clone(),
            metrics.clone(),
            cron.clone(),
        ));
        let auction = Arc::new(
            Auction::new(
                resource_tracking.clone(),
                function_tracking.clone(),
                metrics,
                function.clone(),
                node_situation.get_max_in_flight_functions_proposals(),
            )
            .unwrap(),
        );

        let function_life = Arc::new(
            FunctionLife::new(
                function.clone(),
                auction.clone(),
                node_situation,
                neighbor_monitor,
                node_query,
                function_tracking.clone(),
                cron,
            )
            .unwrap(),
        );
        Instance {
            auction,
            function_life,
            function_tracking,
            resource_tracking,
            reserved_cpu,
            reserved_memory,
            function,
        }
    }

    #[parameterized(
        node = { node_connected(), },
        market = { market_connected(), },
    )]
    #[test_macro(tokio::test())]
    async fn test_sla_refusal(situation: NodeCategory) {
        let auction = get_auction_impl(situation).await.auction;
        let sla = Sla {
            id:                 Uuid::new_v4().into(),
            memory:             Information::new::<gigabyte>(
                num_rational::Ratio::new(1000, 1),
            ),
            cpu:                Ratio::new::<millicpu>(
                num_rational::Ratio::new(100, 1),
            ),
            latency_max:        Time::new::<second>(1.0),
            duration:           Time::new::<second>(5.0),
            max_replica:        1,
            function_image:     "toto".to_string(),
            function_live_name: "toto".to_string(),
            data_flow:          vec![],
            env_vars:           vec![],
            input_max_size:     Information::new::<megabyte>(
                num_rational::Ratio::new(1, 1),
            ),
        };

        let acc = AccumulatedLatency::default();

        let res = auction.bid_on(sla, &acc).await.expect("Error bidding");
        assert!(res.is_none());
    }

    #[parameterized(
        node = {node_connected()},
        market = {market_connected()}
    )]
    #[test_macro(tokio::test)]
    async fn test_sla_acceptance(situation: NodeCategory) {
        let Instance {
            auction,
            function_life,
            function_tracking,
            resource_tracking,
            ..
        } = get_auction_impl(situation).await;

        let sla = Sla {
            id:                 Uuid::new_v4().into(),
            memory:             Information::new::<megabyte>(
                num_rational::Ratio::new(100, 1),
            ),
            cpu:                Ratio::new::<millicpu>(
                num_rational::Ratio::new(100, 1),
            ),
            latency_max:        Time::new::<second>(1.0),
            duration:           Time::new::<second>(5.0),
            max_replica:        1,
            function_image:     "toto".to_string(),
            function_live_name: "toto".to_string(),
            data_flow:          vec![],
            env_vars:           vec![],
            input_max_size:     Information::new::<megabyte>(
                num_rational::Ratio::new(1, 1),
            ),
        };

        let acc = AccumulatedLatency::default();

        let res =
            auction.bid_on(sla.clone(), &acc).await.expect("Bidding faield");

        assert!(res.is_some());
        let (_id, record) = res.unwrap();

        let id = sla.id;
        assert_eq!(id.clone(), record.sla.id);

        function_life
            .pay_function(id.clone())
            .await
            .expect("Function paid failed");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(cc, sla.cpu);
        assert_eq!(ram, sla.memory);

        function_life
            .provision_function(id.clone())
            .await
            .expect("Provisioning paid function failed");

        tokio::time::sleep(Duration::from_secs(6)).await;

        function_tracking
            .get_finished(&id.clone())
            .expect("Function is not finished, not ok");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(
            ram,
            Information::new::<megabyte>(num_rational::Ratio::new(0, 1))
        );
        assert_eq!(cc, Ratio::new::<cpu>(num_rational::Ratio::new(0, 1)));
    }

    /// The function has a duration < to the time it takes to provision the
    /// function on the node
    #[parameterized(
        node = {node_connected()},
        market = {market_connected()},
    )]
    #[test_macro(tokio::test)]
    async fn test_finished_before_live(situation: NodeCategory) {
        let Instance {
            auction,
            function_life,
            function_tracking,
            resource_tracking,
            ..
        } = get_auction_impl(situation).await;
        let sla = Sla {
            id:                 Uuid::new_v4().into(),
            memory:             Information::new::<megabyte>(
                num_rational::Ratio::new(100, 1),
            ),
            cpu:                Ratio::new::<millicpu>(
                num_rational::Ratio::new(100, 1),
            ),
            latency_max:        Time::new::<second>(1.0),
            duration:           Time::new::<second>(1.0),
            max_replica:        1,
            function_image:     "toto".to_string(),
            function_live_name: "toto".to_string(),
            data_flow:          vec![],
            env_vars:           vec![],
            input_max_size:     Information::new::<megabyte>(
                num_rational::Ratio::new(1, 1),
            ),
        };

        let acc = AccumulatedLatency::default();

        let res =
            auction.bid_on(sla.clone(), &acc).await.expect("Bidding faield");

        assert!(res.is_some());
        let (_id, record) = res.unwrap();

        let id = sla.id;
        assert_eq!(id.clone(), record.sla.id);

        function_life
            .pay_function(id.clone())
            .await
            .expect("Function paid failed");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(cc, sla.cpu);
        assert_eq!(ram, sla.memory);

        tokio::time::sleep(Duration::from_secs(2)).await;

        function_tracking
            .get_finished(&id.clone())
            .expect("Function is not finished, not ok");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(
            ram,
            Information::new::<megabyte>(num_rational::Ratio::new(0, 1))
        );
        assert_eq!(cc, Ratio::new::<cpu>(num_rational::Ratio::new(0, 1)));
    }

    #[parameterized(
        node = {node_connected()},
        market = {market_connected()},
    )]
    #[test_macro(tokio::test)]
    async fn test_mutliple_functions(situation: NodeCategory) {
        let Instance {
            auction,
            function_life,
            function_tracking,
            resource_tracking,
            reserved_cpu,
            reserved_memory,
            ..
        } = get_auction_impl(situation).await;

        let sla = Sla {
            id:                 Uuid::new_v4().into(),
            memory:             Information::new::<megabyte>(
                num_rational::Ratio::new(100, 1),
            ),
            cpu:                Ratio::new::<millicpu>(
                num_rational::Ratio::new(100, 1),
            ),
            latency_max:        Time::new::<second>(1.0),
            duration:           Time::new::<second>(1.0),
            max_replica:        1,
            function_image:     "toto".to_string(),
            function_live_name: "toto".to_string(),
            data_flow:          vec![],
            env_vars:           vec![],
            input_max_size:     Information::new::<megabyte>(
                num_rational::Ratio::new(1, 1),
            ),
        };

        let mut sla2 = sla.clone();
        sla2.id = Uuid::new_v4().into();
        sla2.duration = Time::new::<second>(5.0);
        sla2.cpu = Ratio::new::<millicpu>(num_rational::Ratio::new(200, 1));
        sla2.memory =
            Information::new::<megabyte>(num_rational::Ratio::new(50, 1));

        let acc = AccumulatedLatency::default();

        auction.bid_on(sla.clone(), &acc).await.expect("Bidding faield");
        auction.bid_on(sla2.clone(), &acc).await.expect("Bidding faield");

        function_life
            .pay_function(sla2.id.clone())
            .await
            .expect("Function paid failed");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(cc, sla2.cpu);
        assert_eq!(ram, sla2.memory);

        function_life
            .pay_function(sla.id.clone())
            .await
            .expect("Function paid failed");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(cc, sla.cpu + sla2.cpu);
        assert_eq!(ram, sla.memory + sla2.memory);

        tokio::time::sleep(Duration::from_secs(2)).await;

        function_tracking
            .get_finished(&sla.id.clone())
            .expect("Function is not finished, not ok");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(cc, sla2.cpu);
        assert_eq!(ram, sla2.memory);

        tokio::time::sleep(Duration::from_secs(4)).await;

        function_tracking
            .get_finished(&sla2.id.clone())
            .expect("Function is not finished, not ok");

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(
            ram,
            Information::new::<megabyte>(num_rational::Ratio::new(0, 1))
        );
        assert_eq!(cc, Ratio::new::<millicpu>(num_rational::Ratio::new(0, 1)));

        let (ram_a, cc_a) =
            resource_tracking.get_available(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(ram_a, reserved_memory);
        assert_eq!(cc_a, reserved_cpu);

        assert_eq!(ram_a + ram, reserved_memory);
        assert_eq!(cc_a + cc, reserved_cpu);
    }

    /// Check that a lot (millions) of request can be handled without breaking
    /// anything
    #[parameterized(
        node = {node_connected()},
        market = {market_connected()},
    )]
    #[test_macro(tokio::test(flavor = "multi_thread", worker_threads = 20))]
    async fn test_lots_functions_parallel(situation: NodeCategory) {
        //let result = tokio::time::timeout(Duration::from_secs(120), async {
        _test_lots_functions_parallel(situation).await;
        //    "done"
        //})
        //.await;
        //assert!(result.is_ok())
    }

    async fn _test_lots_functions_parallel(situation: NodeCategory) {
        let Instance {
            auction,
            function_life,
            function_tracking,
            resource_tracking,
            ..
        } = get_auction_impl(situation).await;

        let mut handles = Vec::new();

        for ii in 0..2_000_000 {
            let function_life = function_life.clone();
            let auction = auction.clone();

            let hh = tokio::spawn(async move {
                let mut r = StdRng::seed_from_u64(ii);
                let law = Uniform::from(1..5);
                tokio::time::sleep(Duration::from_secs(
                    law.sample(&mut r) * 6,
                ))
                .await;

                let sla_id: SlaId = Uuid::new_v4().into();
                let sla_duration =
                    Time::new::<second>(law.sample(&mut r) as f64);

                let sla = Sla {
                    id:                 sla_id.clone(),
                    memory:             Information::new::<megabyte>(
                        num_rational::Ratio::new(
                            law.sample(&mut r) as i64 * 10,
                            1,
                        ),
                    ),
                    cpu:                Ratio::new::<millicpu>(
                        num_rational::Ratio::new(
                            law.sample(&mut r) as i64 * 10,
                            1,
                        ),
                    ),
                    latency_max:        Time::new::<second>(10.0),
                    duration:           sla_duration.clone(),
                    max_replica:        1,
                    function_image:     "toto".to_string(),
                    function_live_name: "toto".to_string(),
                    data_flow:          vec![],
                    env_vars:           vec![],
                    input_max_size:     Information::new::<megabyte>(
                        num_rational::Ratio::new(1, 1),
                    ),
                };

                let mut pay_it = auction
                    .bid_on(sla, &AccumulatedLatency::default())
                    .await
                    .unwrap()
                    .is_some();

                pay_it = pay_it && law.sample(&mut r) > 1;

                if pay_it {
                    pay_it = function_life
                        .pay_function(sla_id.clone())
                        .await
                        .is_ok();
                }
                if pay_it {
                    tokio::time::sleep(Duration::from_secs(
                        sla_duration.get::<second>().to_f64().unwrap().ceil()
                            as u64
                            + 2,
                    ))
                    .await;
                }
                (
                    pay_it,
                    pay_it || sla_duration <= Time::new::<second>(2.0),
                    sla_id,
                )
            });

            handles.push(hh);
        }

        let mut ran = 0;
        let mut successes = 0;
        for hh in handles {
            let (pay_it, success, sla_id) = hh.await.unwrap();
            if success {
                successes += 1;
            }
            if pay_it {
                assert!(function_tracking.get_finished(&sla_id).is_some());
                ran += 1;
            }
        }
        assert!(successes > 0);
        assert!(successes >= 10_000);

        assert!(ran > 0);
        assert!(ran >= 8000 / 50);

        let (ram, cc) =
            resource_tracking.get_used(OFFLINE_NODE_K8S).await.unwrap();
        assert_eq!(
            ram,
            Information::new::<megabyte>(num_rational::Ratio::new(0, 1))
        );
        assert_eq!(cc, Ratio::new::<millicpu>(num_rational::Ratio::new(0, 1)));
    }

    #[cfg(feature = "quadratic_rates")]
    #[parameterized(
        node = {node_connected()},
        market = {market_connected()},
    )]
    #[test_macro(tokio::test)]
    async fn test_quadratic_rates_always_increasing(situation: NodeCategory) {
        let Instance { auction, function_life, .. } =
            get_auction_impl(situation).await;

        let mut handles = Vec::new();

        let last_bid_price = Arc::new(AtomicF64::new(0.0));

        for ii in 0..10 {
            let function_life = function_life.clone();
            let auction = auction.clone();
            let last_bid_price = last_bid_price.clone();

            let hh = tokio::spawn(async move {
                tokio::time::sleep(Duration::from_secs(ii)).await;

                let sla_id: SlaId = Uuid::new_v4().into();
                let sla_duration = Time::new::<second>(15.0);

                let sla = Sla {
                    id:                 sla_id.clone(),
                    memory:             Information::new::<megabyte>(
                        num_rational::Ratio::new(50, 1),
                    ),
                    cpu:                Ratio::new::<millicpu>(
                        num_rational::Ratio::new(50, 1),
                    ),
                    latency_max:        Time::new::<second>(10.0),
                    duration:           sla_duration.clone(),
                    max_replica:        1,
                    function_image:     "toto".to_string(),
                    function_live_name: "toto".to_string(),
                    data_flow:          vec![],
                    env_vars:           vec![],
                    input_max_size:     Information::new::<megabyte>(
                        num_rational::Ratio::new(1, 1),
                    ),
                };

                let bid = auction
                    .bid_on(sla, &AccumulatedLatency::default())
                    .await
                    .unwrap();
                let mut pay_it = bid.is_some();

                if let Some((_, propal)) = bid {
                    assert_eq!(propal.node, OFFLINE_NODE_K8S);
                    assert!(
                        propal.bid > last_bid_price.load(Ordering::SeqCst)
                    );
                    last_bid_price.store(propal.bid, Ordering::SeqCst);
                }

                if pay_it {
                    pay_it = function_life
                        .pay_function(sla_id.clone())
                        .await
                        .is_ok();
                }
                if pay_it {
                    tokio::time::sleep(Duration::from_secs(
                        sla_duration.get::<second>().to_f64().unwrap().ceil()
                            as u64,
                    ))
                    .await;
                }
                (
                    pay_it,
                    pay_it || sla_duration <= Time::new::<second>(2.0),
                    sla_id,
                )
            });

            handles.push(hh);
        }

        for hh in handles {
            hh.await.unwrap();
        }
    }

    #[cfg(feature = "quadratic_rates")]
    async fn _test_quadratic_rates_inner_loop(
        function_life: Arc<FunctionLife>,
        auction: Arc<Auction>,
        last_bid_price: Arc<AtomicF64>,
        wait: u64,
    ) -> f64 {
        tokio::time::sleep(Duration::from_secs(wait)).await;

        let sla_id: SlaId = Uuid::new_v4().into();
        let sla_duration = Time::new::<second>(15.0);

        let sla = Sla {
            id:                 sla_id.clone(),
            memory:             Information::new::<megabyte>(
                num_rational::Ratio::new(50, 1),
            ),
            cpu:                Ratio::new::<millicpu>(
                num_rational::Ratio::new(50, 1),
            ),
            latency_max:        Time::new::<second>(10.0),
            duration:           sla_duration.clone(),
            max_replica:        1,
            function_image:     "toto".to_string(),
            function_live_name: "toto".to_string(),
            data_flow:          vec![],
            env_vars:           vec![],
            input_max_size:     Information::new::<megabyte>(
                num_rational::Ratio::new(1, 1),
            ),
        };

        let bid =
            auction.bid_on(sla, &AccumulatedLatency::default()).await.unwrap();
        let mut pay_it = bid.is_some();

        if let Some((_, propal)) = bid {
            assert_eq!(propal.node, OFFLINE_NODE_K8S);
            assert!(propal.bid > last_bid_price.load(Ordering::SeqCst));
            last_bid_price.store(propal.bid, Ordering::SeqCst);
        }

        if pay_it {
            pay_it = function_life.pay_function(sla_id.clone()).await.is_ok();
        }

        let ret = last_bid_price.load(Ordering::SeqCst);
        if pay_it {
            tokio::time::sleep(Duration::from_secs(
                sla_duration.get::<second>().to_f64().unwrap().ceil() as u64,
            ))
            .await;
        }

        return ret;
    }

    #[cfg(feature = "quadratic_rates")]
    #[parameterized(
        node = {node_connected()},
        market = {market_connected()},
    )]
    #[test_macro(tokio::test(flavor = "multi_thread", worker_threads = 20))]
    async fn test_quadratic_rates_repeatability(situation: NodeCategory) {
        use futures::future::join_all;
        let Instance { auction, function_life, function, .. } =
            get_auction_impl(situation).await;

        let mut handles = Vec::new();

        let last_bid_price = Arc::new(AtomicF64::new(0.0));

        for ii in 0..10 {
            let function_life = function_life.clone();
            let auction = auction.clone();
            let last_bid_price = last_bid_price.clone();

            let hh = tokio::spawn(_test_quadratic_rates_inner_loop(
                function_life,
                auction,
                last_bid_price,
                ii,
            ));
            handles.push(hh);
        }

        let results: Vec<f64> = join_all(handles)
            .await
            .into_iter()
            .map(|res| res.unwrap())
            .collect();

        assert_eq!(function.get_utilisation_variations().await.len(), 0);

        handles = Vec::new();
        last_bid_price.store(0.0, Ordering::SeqCst);

        for ii in 0..10 {
            let function_life = function_life.clone();
            let auction = auction.clone();
            let last_bid_price = last_bid_price.clone();

            let hh = tokio::spawn(_test_quadratic_rates_inner_loop(
                function_life,
                auction,
                last_bid_price,
                ii,
            ));
            handles.push(hh);
        }

        let results2: Vec<f64> = join_all(handles)
            .await
            .into_iter()
            .map(|res| res.unwrap())
            .collect();
        let mut ii = 0;
        for (result, result2) in results.iter().zip(results2.iter()) {
            assert!(
                *result < *result2 + 0.001 || *result2 < *result + 0.001,
                "Failed on {}",
                ii
            );
            ii += 1;
        }

        assert_eq!(function.get_utilisation_variations().await.len(), 0);
    }
}
