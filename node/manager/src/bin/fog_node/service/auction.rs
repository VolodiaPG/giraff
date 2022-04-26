use std::sync::Arc;

use async_trait::async_trait;
use if_chain::if_chain;

use manager::model::domain::sla::Sla;
use manager::model::dto::auction::BidRecord;
use manager::model::dto::k8s::Metrics;
use manager::model::BidId;

use crate::repository::auction::Auction as AuctionRepository;
use crate::repository::k8s::K8s as K8sRepository;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Inherited an error when contacting the k8s API: {0}")]
    Kube(#[from] kube::Error),
    #[error(transparent)]
    K8S(#[from] crate::repository::k8s::Error),
    #[error("BidId not found: {0}")]
    BidIdNotFound(BidId),
    #[error("The SLA is not satisfiable")]
    Unsatisfiable,
}

#[async_trait]
pub trait Auction: Send + Sync {
    /// Bid on the [Sla] and return the price.
    async fn bid_on(&self, sla: Sla) -> Result<(BidId, BidRecord), Error>;

    /// Promote the bid to a full fledged provisioned function in the database.
    async fn validate_bid(&self, id: &BidId) -> Result<BidRecord, Error>;
}

pub struct AuctionImpl {
    k8s: Arc<dyn K8sRepository>,
    db: Arc<dyn AuctionRepository>,
}

impl AuctionImpl {
    pub fn new(k8s: Arc<dyn K8sRepository>, db: Arc<dyn AuctionRepository>) -> Self {
        Self { k8s, db }
    }

    async fn compute_bid(&self, sla: &Sla) -> Result<f64, Error> {
        let aggregated_metrics = self.k8s.get_k8s_metrics().await?;

        let (name, metrics) = aggregated_metrics
            .iter()
            .find(|(_key, metrics)| self.satisfiability_check(&metrics, &sla))
            .ok_or(Error::Unsatisfiable)?;

        let allocatable = metrics.allocatable.as_ref().ok_or(Error::Unsatisfiable)?;
        let usage = metrics.usage.as_ref().ok_or(Error::Unsatisfiable)?;

        let price = (allocatable.memory - usage.memory) / sla.memory;
        let price: f64 = price.into();

        trace!("price on {:?} is {:?}", name, price);

        Ok(price)
    }

    /// Check if the SLA is satisfiable by the current node.
    fn satisfiability_check(&self, metrics: &Metrics, sla: &Sla) -> bool {
        if_chain! {
            if let Some(allocatable) = &metrics.allocatable;
            if let Some(usage) = &metrics.usage;
            if allocatable.memory - usage.memory > sla.memory;
            then
            {
                trace!("{:?}", (allocatable.memory - usage.memory).into_format_args(uom::si::information::megabyte, uom::fmt::DisplayStyle::Description));

                return true
            }
        }

        false
    }
}

#[async_trait]
impl Auction for AuctionImpl {
    async fn bid_on(&self, sla: Sla) -> Result<(BidId, BidRecord), Error> {
        let bid = self.compute_bid(&sla).await?;
        let bid = BidRecord { bid, sla };
        let id = self.db.insert(bid.to_owned()).await;
        Ok((id, bid))
    }

    async fn validate_bid(&self, id: &BidId) -> Result<BidRecord, Error> {
        let bid = self
            .db
            .get(id)
            .await
            .ok_or_else(|| Error::BidIdNotFound(id.to_owned()))?
            .clone();

        self.db.remove(id).await;

        Ok(bid)
    }
}
