use std::sync::Arc;

use async_trait::async_trait;
use uom::si::f64::{Information, Ratio};

use crate::prom_metrics::BID_GAUGE;
use model::domain::sla::Sla;
use model::dto::auction::BidRecord;
use model::BidId;

use crate::repository::auction::Auction as AuctionRepository;
use crate::repository::resource_tracking::ResourceTracking;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("BidId not found: {0}")]
    BidIdNotFound(BidId),
    #[error("The SLA is not satisfiable")]
    Unsatisfiable,
    #[error(transparent)]
    ResourceTracking(#[from] crate::repository::resource_tracking::Error),
}

#[async_trait]
pub trait Auction: Send + Sync {
    /// Bid on the [Sla] and return the price.
    async fn bid_on(&self, sla: Sla) -> Result<(BidId, BidRecord), Error>;

    /// Promote the bid to a full fledged provisioned function in the database.
    async fn validate_bid(&self, id: &BidId) -> Result<BidRecord, Error>;
}

pub struct AuctionImpl {
    resource_tracking: Arc<dyn ResourceTracking>,
    db:                Arc<dyn AuctionRepository>,
}

impl AuctionImpl {
    pub async fn new(
        resource_tracking: Arc<dyn ResourceTracking>,
        db: Arc<dyn AuctionRepository>,
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
        let would_be_used_ram = *used_ram + sla.memory;
        let would_be_used_cpu = *used_cpu + sla.cpu;

        would_be_used_cpu < *available_cpu
            && would_be_used_ram < *available_ram
    }
}

#[async_trait]
impl Auction for AuctionImpl {
    async fn bid_on(&self, sla: Sla) -> Result<(BidId, BidRecord), Error> {
        let (node, bid) = self.compute_bid(&sla).await?;
        let record = BidRecord { bid, sla, node };
        let id = self.db.insert(record.to_owned());
        BID_GAUGE
            .with_label_values(&[
                &record.sla.function_live_name,
                &id.to_string(),
            ])
            .set(bid);
        Ok((id, record))
    }

    async fn validate_bid(&self, id: &BidId) -> Result<BidRecord, Error> {
        let bid = self
            .db
            .get(id)
            .ok_or_else(|| Error::BidIdNotFound(id.to_owned()))?
            .clone();

        self.db.remove(id);

        let (used_mem, used_cpu) =
            self.resource_tracking.get_used(&bid.node).await?;
        let used_mem = used_mem + bid.sla.memory;
        let used_cpu = used_cpu + bid.sla.cpu;
        self.resource_tracking
            .update_used(bid.node.clone(), used_mem, used_cpu)
            .await?;

        Ok(bid)
    }
}
