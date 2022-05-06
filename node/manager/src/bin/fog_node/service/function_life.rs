use crate::service::auction::Auction;
use crate::service::faas::FaaSBackend;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};
use async_trait::async_trait;
use futures::future::{join, join3, try_join_all};
use manager::model::domain::sla::Sla;
use manager::model::view::auction::{BidProposal, BidProposals};
use manager::model::{BidId, NodeId};
use std::sync::Arc;
use uom::fmt::DisplayStyle::Abbreviation;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Auction(#[from] crate::service::auction::Error),
    #[error(transparent)]
    FaaS(#[from] crate::service::faas::Error),
    #[error(transparent)]
    NodeQuery(#[from] crate::repository::node_query::Error),
    #[error("Cannot get latency of node {0}")]
    CannotGetLatency(NodeId),
}

#[async_trait]
pub trait FunctionLife: Send + Sync {
    /// Declares and bid on  the new function
    /// Will save the function in the database for later use
    ///
    /// [from] is the last node that passed the request
    async fn bid_on_new_function_and_transmit(
        &self,
        sla: Sla,
        from: NodeId,
    ) -> Result<BidProposals, Error>;

    async fn validate_bid_and_provision_function(&self, id: BidId) -> Result<(), Error>;
}

pub struct FunctionLifeImpl {
    function: Arc<dyn FaaSBackend>,
    auction: Arc<dyn Auction>,
    node_situation: Arc<dyn NodeSituation>,
    neighbor_monitor: Arc<dyn NeighborMonitor>,
    node_query: Arc<dyn NodeQuery>,
}

impl FunctionLifeImpl {
    pub fn new(
        function: Arc<dyn FaaSBackend>,
        auction: Arc<dyn Auction>,
        node_situation: Arc<dyn NodeSituation>,
        neighbor_monitor: Arc<dyn NeighborMonitor>,
        node_query: Arc<dyn NodeQuery>,
    ) -> Self {
        Self {
            function,
            auction,
            node_situation,
            neighbor_monitor,
            node_query,
        }
    }

    /// Follow up the [Sla] to the neighbors, and ignore the path where it came from.
    async fn follow_up_to_neighbors(&self, sla: Sla, from: NodeId) -> Result<BidProposals, Error> {
        let mut promises = vec![];
        for neighbor in self.node_situation.get_neighbors().await {
            if neighbor == from {
                continue;
            }
            let latency_outbound = self
                .neighbor_monitor
                .get_latency_to_avg(&neighbor)
                .await
                .ok_or_else(|| Error::CannotGetLatency(neighbor.clone()))?;
            if latency_outbound > sla.latency_max {
                trace!(
                    "Skipping neighbor {} because latency is too high ({}).",
                    neighbor,
                    latency_outbound.into_format_args(uom::si::time::millisecond, Abbreviation)
                );
                continue;
            }

            promises.push(self.node_query.request_neighbor_bid(&sla, neighbor.clone()));
        }

        Ok(BidProposals {
            bids: try_join_all(promises)
                .await?
                .into_iter()
                .flat_map(|proposals: BidProposals| proposals.bids)
                .collect(),
        })
    }
}

#[async_trait]
impl FunctionLife for FunctionLifeImpl {
    async fn bid_on_new_function_and_transmit(
        &self,
        sla: Sla,
        from: NodeId,
    ) -> Result<BidProposals, Error> {
        let (my_id, result_bid) = join(
            self.node_situation.get_my_id(),
            self.auction.bid_on(sla.clone()),
            // self.follow_up_to_neighbors(sla, from),
        )
        .await;

        let (bid, bid_record) = result_bid?;
        let mut proposals = BidProposals { bids: vec![] };
        proposals.bids.push(BidProposal {
            node_id: my_id,
            id: bid,
            bid: bid_record.bid,
        });

        Ok(proposals)
    }

    async fn validate_bid_and_provision_function(&self, id: BidId) -> Result<(), Error> {
        let record = self.auction.validate_bid(&id).await?;
        self.function.provision_function(id, record).await?;
        Ok(())
    }
}
