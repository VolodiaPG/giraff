use crate::monitoring::BidGauge;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use anyhow::{Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use model::domain::sla::Sla;
use model::dto::function::{FunctionRecord, Proposed};
use model::view::auction::AccumulatedLatency;
use model::BidId;
use std::sync::Arc;
use uom::si::f64::{Information, Ratio};

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

    /// Compute the bid value from the node environment
    #[cfg(not(feature = "valuation_rates"))]
    async fn compute_bid(
        &self,
        sla: &Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(String, f64)>> {
        use uom::si::f64::Time;
        use uom::si::time::millisecond;

        let Some((name, _used_ram, _used_cpu, available_ram, available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")? else{
                    return Ok(None);
                };

        let mut price = sla.memory / available_ram + sla.cpu / available_cpu;
        if accumulated_latency.median > Time::new::<millisecond>(1.0) {
            price = price * sla.latency_max / accumulated_latency.median;
        } else {
            price = price * sla.latency_max / Time::new::<millisecond>(1.0);
        }

        let price: f64 = price.into();

        trace!("price on {:?} is {:?}", name, price);

        Ok(Some((name, price)))
    }

    #[cfg(feature = "valuation_rates")]
    async fn compute_bid(
        &self,
        sla: &Sla,
        _accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<(String, f64)>> {
        use helper::uom_helper::cpu_ratio::millicpu;
        use uom::si::information::mebibyte;

        let Some((name, _used_ram, _used_cpu, _available_ram, _available_cpu)) =
            self.get_a_node(sla)
                .await
                .context("Failed to found a suitable node for the sla")? else{
                    return Ok(None);
                };

        let price_ram: f64 = std::env::var("VALUATION_PER_MIB")
            .context(
                "Failed to get value from 
        the environment variable VALUATION_PER_MIB",
            )?
            .parse()
            .context(
                "Failed to convert environment variable VALUATION_PER_MIB to \
                 a f64",
            )?;
        let price_cpu: f64 = std::env::var("VALUATION_PER_MILLICPU")
            .context(
                "Failed to get value from 
        the environment variable VALUATION_PER_MILLICPU",
            )?
            .parse()
            .context(
                "Failed to convert environment variable \
                 VALUATION_PER_MILLICPU to a f64",
            )?;

        let price_cpu = price_cpu / Ratio::new::<millicpu>(1.0);
        let price_ram = price_ram / Information::new::<mebibyte>(1.0);

        let price = sla.cpu * price_cpu + sla.memory * price_ram;

        let price: f64 = price.into();

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
