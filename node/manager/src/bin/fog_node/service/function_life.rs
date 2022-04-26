use std::sync::Arc;

use async_trait::async_trait;

use manager::model::domain::sla::Sla;
use manager::model::dto::auction::BidRecord;
use manager::model::BidId;

use crate::service::auction::Auction;
use crate::service::faas::FaaSBackend;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    FaaS(#[from] crate::service::faas::Error),
}

#[async_trait]
pub trait FunctionLife: Send + Sync {
    /// Declares and bid on  the new function
    /// Will save the function in the database for later use
    async fn bid_on_new_function(&self, sla: Sla) -> Result<(BidId, BidRecord), Error>;

    async fn validate_bid_and_provision_function(&self, id: BidId) -> Result<(), Error>;
}

pub struct FunctionLifeImpl {
    function: Arc<dyn FaaSBackend>,
    auction: Arc<dyn Auction>,
}

impl FunctionLifeImpl {
    pub fn new(function: Arc<dyn FaaSBackend>, auction: Arc<dyn Auction>) -> Self {
        FunctionLifeImpl { function, auction }
    }
}

#[async_trait]
impl FunctionLife for FunctionLifeImpl {
    async fn bid_on_new_function(&self, sla: Sla) -> Result<(BidId, BidRecord), Error> {
        self.auction.bid_on(sla).await.map_err(Error::from)
    }

    async fn validate_bid_and_provision_function(&self, id: BidId) -> Result<(), Error> {
        let record = self.auction.validate_bid(&id).await?;
        self.function.provision_function(id, record).await?;
        Ok(())
    }
}
