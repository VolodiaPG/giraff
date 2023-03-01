use crate::prom_metrics::BID_GAUGE;
use crate::repository::function_tracking::FunctionTracking;
use crate::repository::resource_tracking::ResourceTracking;
use model::domain::sla::Sla;
use model::dto::function::{FunctionRecord, Proposed};
use model::BidId;
use std::sync::Arc;
use uom::si::f64::{Information, Ratio};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("The SLA is not satisfiable")]
    Unsatisfiable,
    #[error(transparent)]
    ResourceTracking(#[from] crate::repository::resource_tracking::Error),
}

pub struct Auction {
    resource_tracking: Arc<ResourceTracking>,
    db:                Arc<FunctionTracking>,
}

impl Auction {
    pub fn new(
        resource_tracking: Arc<ResourceTracking>,
        db: Arc<FunctionTracking>,
    ) -> Self {
        Self { resource_tracking, db }
    }

    /// Get a suitable (free enough) node to potentially run the designated SLA
    async fn get_a_node(
        &self,
        sla: &Sla,
    ) -> Result<(String, Information, Ratio, Information, Ratio), Error> {
        for node in self.resource_tracking.get_nodes() {
            let (used_ram, used_cpu) =
                self.resource_tracking.get_used(node).await?;
            let (available_ram, available_cpu) =
                self.resource_tracking.get_available(node).await?;
            if self.satisfiability_check(
                &used_ram,
                &used_cpu,
                &available_ram,
                &available_cpu,
                sla,
            ) {
                return Ok((
                    node.clone(),
                    used_ram,
                    used_cpu,
                    available_ram,
                    available_cpu,
                ));
            }
        }
        Err(Error::Unsatisfiable)
    }

    /// Compute the bid value from the node environment
    async fn compute_bid(&self, sla: &Sla) -> Result<(String, f64), Error> {
        let (name, _used_ram, _used_cpu, available_ram, available_cpu) =
            self.get_a_node(sla).await?;

        let price = sla.memory / available_ram + sla.cpu / available_cpu;

        let price: f64 = price.into();

        trace!("price on {:?} is {:?}", name, price);

        Ok((name, price))
    }

    /// Check if the SLA is satisfiable by the current node (designated by name
    /// and metrics).
    fn satisfiability_check(
        &self,
        used_ram: &Information,
        used_cpu: &Ratio,
        available_ram: &Information,
        available_cpu: &Ratio,
        sla: &Sla,
    ) -> bool {
        let would_be_used_ram =
            *used_ram + (sla.memory * sla.max_replica as f64);
        let would_be_used_cpu = *used_cpu + (sla.cpu * sla.max_replica as f64);

        would_be_used_cpu < *available_cpu
            && would_be_used_ram < *available_ram
    }

    pub async fn bid_on(
        &self,
        sla: Sla,
    ) -> Result<(BidId, FunctionRecord<Proposed>), Error> {
        let (node, bid) = self.compute_bid(&sla).await?;
        let record = FunctionRecord::new(bid, sla, node);
        let id = self.db.insert(record.clone());
        BID_GAUGE
            .with_label_values(&[
                &record.0.sla.function_live_name,
                &id.to_string(),
            ])
            .set(bid);
        Ok((id, record))
    }
}
