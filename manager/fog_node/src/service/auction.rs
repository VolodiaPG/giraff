use crate::monitoring::BidGauge;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use anyhow::{Context, Result};
use chrono::Utc;
use helper::env_var;
use helper::monitoring::MetricsExporter;
use model::domain::sla::Sla;
use model::dto::function::{FunctionRecord, Proposed};
use model::view::auction::AccumulatedLatency;
use model::BidId;
use nutype::nutype;
use std::sync::Arc;
use uom::si::f64::{Information, Ratio};

#[nutype(validate(finite, min = 0.0))]
#[derive(PartialEq, PartialOrd)]
pub struct PricingRatio(f64);

env_var!(PRICING_CPU);
env_var!(PRICING_CPU_INITIAL);
env_var!(PRICING_MEM);
env_var!(PRICING_MEM_INITIAL);
env_var!(PRICING_GEOLOCATION);

pub struct Auction {
    resource_tracking: Arc<ResourceTracking>,
    db:                Arc<FunctionTracking>,
    metrics:           Arc<MetricsExporter>,
}

impl Auction {
    pub fn new(
        resource_tracking: Arc<ResourceTracking>,
        db: Arc<FunctionTracking>,
        metrics: Arc<MetricsExporter>,
    ) -> Self {
        Self { resource_tracking, db, metrics }
    }

    /// Get a suitable (free enough) node to potentially run the designated SLA
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

    #[cfg(not(feature = "valuation_rates"))]
    fn sigmoid(x: f32) -> f32 {
        let x = if x < 0.0 {
            0.0
        } else if x > 1.0 {
            1.0
        } else {
            x
        };

        1.0 / (1.0 + fast_math::exp_raw(-4.0 * (x - 0.5)))
    }

    /// Compute the bid value from the node environment
    #[cfg(not(feature = "valuation_rates"))]
    async fn compute_bid(
        &self,
        sla: &Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(String, f64)>> {
        use helper::env_load;

        let pricing_cpu = env_load!(PricingRatio, PRICING_CPU, f64);
        let pricing_mem = env_load!(PricingRatio, PRICING_MEM, f64);
        let pricing_geolocation =
            env_load!(PricingRatio, PRICING_GEOLOCATION, f64);

        let Some((name, used_ram, used_cpu, available_ram, available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")? else{
                    return Ok(None);
                };

        let ram_ratio_sla: f64 = (sla.memory / available_ram).into();
        let cpu_ratio_sla: f64 = (sla.cpu / available_cpu).into();
        let ram_ratio: f64 = (used_ram / available_ram).into();
        let cpu_ratio: f64 = (used_cpu / available_cpu).into();
        let latency_ratio: f64 = (sla.latency_max
            / (accumulated_latency.median
                - accumulated_latency.median_uncertainty))
            .into();
        let price = pricing_mem.into_inner() * ram_ratio_sla
            + pricing_cpu.into_inner() * cpu_ratio_sla
            + pricing_geolocation.into_inner()
                * (Auction::sigmoid(ram_ratio as f32) as f64
                    + Auction::sigmoid(cpu_ratio as f32) as f64
                    + Auction::sigmoid(latency_ratio as f32) as f64);

        trace!("price on {:?} is {:?}", name, price);

        Ok(Some((name, price)))
    }

    #[cfg(feature = "valuation_rates")]
    async fn compute_bid(
        &self,
        sla: &Sla,
        _accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(String, f64)>> {
        use helper::env_load;

        let pricing_cpu =
            env_load!(PricingRatio, PRICING_CPU, f64).into_inner();
        let pricing_cpu_initial =
            env_load!(PricingRatio, PRICING_CPU_INITIAL, f64).into_inner();
        let pricing_mem =
            env_load!(PricingRatio, PRICING_MEM, f64).into_inner();
        let pricing_mem_initial =
            env_load!(PricingRatio, PRICING_MEM_INITIAL, f64).into_inner();

        let Some((name, used_ram, used_cpu, available_ram, available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")? else{
                    return Ok(None);
                };

        let ram_ratio_sla: f64 = (sla.memory / available_ram).into();
        let cpu_ratio_sla: f64 = (sla.cpu / available_cpu).into();
        let ram_ratio: f64 = ((used_ram + sla.memory) / available_ram).into();
        let cpu_ratio: f64 = ((used_cpu + sla.cpu) / available_cpu).into();
        let price = (ram_ratio - ram_ratio_sla)
            * (pricing_mem * (ram_ratio_sla + ram_ratio)
                + 2.0 * pricing_mem_initial)
            / 2.0
            + (cpu_ratio - cpu_ratio_sla)
                * (pricing_cpu * (cpu_ratio_sla + cpu_ratio)
                    + 2.0 * pricing_cpu_initial)
                / 2.0;

        trace!("price on {:?} is {:?}", name, price);

        Ok(Some((name, price)))
    }

    pub async fn bid_on(
        &self,
        sla: Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(BidId, FunctionRecord<Proposed>)>> {
        let Some((node, bid)) = self
            .compute_bid(&sla, accumulated_latency)
            .await
            .context("Failed to compute bid for sla")? else{
                return Ok(None);
            };
        let record = FunctionRecord::new(bid, sla, node);
        let id = self.db.insert(record.clone());
        self.metrics
            .observe(BidGauge {
                bid,
                function_name: record.0.sla.function_live_name.clone(),
                sla_id: record.0.sla.id.to_string(),
                bid_id: id.to_string(),
                timestamp: Utc::now(),
            })
            .await?;
        Ok(Some((id, record)))
    }
}
