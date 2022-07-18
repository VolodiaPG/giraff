use std::sync::Arc;

use async_trait::async_trait;
use uom::fmt::DisplayStyle::Abbreviation;
use uom::si::f64::Time;

use manager::model::domain::sla::Sla;
use manager::model::view::auction::{BidProposal, BidProposals, BidRequest};
use manager::model::{BidId, NodeId};

use crate::service::auction::Auction;
use crate::service::faas::FaaSBackend;
use crate::service::neighbor_monitor::NeighborMonitor;
use crate::{NodeQuery, NodeSituation};

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
    #[error("No candidates were found or returned an Ok result")]
    NoCandidatesRetained,
}

#[async_trait]
pub trait FunctionLife: Send + Sync {
    /// Declares and bid on  the new function
    /// Will save the function in the database for later use
    ///
    /// [from] is the last node that passed the request
    async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error>;

    async fn validate_bid_and_provision_function(
        &self,
        id: BidId,
    ) -> Result<(), Error>;
}

#[cfg(not(feature = "bottom_up_placement"))]
pub use auction_placement::*;

#[cfg(not(feature = "bottom_up_placement"))]
mod auction_placement {
    use super::*;

    use futures::future::{join3, try_join_all};

    pub struct FunctionLifeImpl {
        function:         Arc<dyn FaaSBackend>,
        auction:          Arc<dyn Auction>,
        node_situation:   Arc<dyn NodeSituation>,
        neighbor_monitor: Arc<dyn NeighborMonitor>,
        node_query:       Arc<dyn NodeQuery>,
    }

    impl FunctionLifeImpl {
        pub fn new(
            function: Arc<dyn FaaSBackend>,
            auction: Arc<dyn Auction>,
            node_situation: Arc<dyn NodeSituation>,
            neighbor_monitor: Arc<dyn NeighborMonitor>,
            node_query: Arc<dyn NodeQuery>,
        ) -> Self {
            debug!("Built using FunctionLifeImpl service");
            Self {
                function,
                auction,
                node_situation,
                neighbor_monitor,
                node_query,
            }
        }

        /// Follow up the [Sla] to the neighbors, and ignore the path where it
        /// came from.
        async fn follow_up_to_neighbors(
            &self,
            sla: &Sla,
            from: NodeId,
            accumulated_latency: Time,
        ) -> Result<BidProposals, Error> {
            let mut requests = vec![];

            for neighbor in self.node_situation.get_neighbors().await {
                if neighbor == from {
                    continue;
                }
                let latency_outbound = self
                    .neighbor_monitor
                    .get_latency_to_avg(&neighbor)
                    .await
                    .ok_or_else(|| {
                        Error::CannotGetLatency(neighbor.clone())
                    })?;
                if latency_outbound + accumulated_latency > sla.latency_max {
                    trace!(
                        "Skipping neighbor {} because latency is too high \
                         ({}).",
                        neighbor,
                        latency_outbound.into_format_args(
                            uom::si::time::millisecond,
                            Abbreviation
                        )
                    );
                    continue;
                }

                requests.push((
                    BidRequest {
                        sla,
                        node_origin: self.node_situation.get_my_id().await,
                        accumulated_latency: accumulated_latency
                            + latency_outbound,
                    },
                    neighbor,
                ));
            }

            let promises = requests.iter().map(|(request, neighbor)| {
                self.node_query.request_neighbor_bid(request, neighbor.clone())
            });

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
            sla: &Sla,
            from: NodeId,
            accumulated_latency: Time,
        ) -> Result<BidProposals, Error> {
            let (my_id, result_bid, proposals) = join3(
                self.node_situation.get_my_id(),
                self.auction.bid_on(sla.clone()),
                self.follow_up_to_neighbors(sla, from, accumulated_latency),
            )
            .await;

            let (bid, bid_record) = result_bid?;
            let mut proposals = proposals?;

            proposals.bids.push(BidProposal {
                node_id: my_id,
                id:      bid,
                bid:     bid_record.bid,
            });

            Ok(proposals)
        }

        async fn validate_bid_and_provision_function(
            &self,
            id: BidId,
        ) -> Result<(), Error> {
            let record = self.auction.validate_bid(&id).await?;
            self.function.provision_function(id, record).await?;
            Ok(())
        }
    }
}

#[cfg(feature = "bottom_up_placement")]
pub use bottom_up_placement::*;

#[cfg(feature = "bottom_up_placement")]
mod bottom_up_placement {
    use super::*;

    pub struct FunctionLifeBottomUpImpl {
        function:         Arc<dyn FaaSBackend>,
        auction:          Arc<dyn Auction>,
        node_situation:   Arc<dyn NodeSituation>,
        neighbor_monitor: Arc<dyn NeighborMonitor>,
        node_query:       Arc<dyn NodeQuery>,
    }

    impl FunctionLifeBottomUpImpl {
        pub fn new(
            function: Arc<dyn FaaSBackend>,
            auction: Arc<dyn Auction>,
            node_situation: Arc<dyn NodeSituation>,
            neighbor_monitor: Arc<dyn NeighborMonitor>,
            node_query: Arc<dyn NodeQuery>,
        ) -> Self {
            debug!("Built using FunctionLifeBottomUpImpl service");
            Self {
                function,
                auction,
                node_situation,
                neighbor_monitor,
                node_query,
            }
        }

        /// Follow up the [Sla] to the neighbors, and ignore the path where it
        /// came from.
        async fn follow_up_to_neighbors(
            &self,
            sla: &Sla,
            from: NodeId,
            accumulated_latency: Time,
        ) -> Result<BidProposals, Error> {
            for neighbor in self.node_situation.get_neighbors().await {
                if neighbor == from {
                    continue;
                }
                let latency_outbound = self
                    .neighbor_monitor
                    .get_latency_to_avg(&neighbor)
                    .await
                    .ok_or_else(|| {
                        Error::CannotGetLatency(neighbor.clone())
                    })?;
                if latency_outbound + accumulated_latency > sla.latency_max {
                    trace!(
                        "Skipping neighbor {} because latency is too high \
                         ({}).",
                        neighbor,
                        latency_outbound.into_format_args(
                            uom::si::time::millisecond,
                            Abbreviation
                        )
                    );
                    continue;
                }

                let bid = self
                    .node_query
                    .request_neighbor_bid(
                        &BidRequest {
                            sla,
                            node_origin: self.node_situation.get_my_id().await,
                            accumulated_latency: accumulated_latency
                                + latency_outbound,
                        },
                        neighbor.clone(),
                    )
                    .await?;
                if !bid.bids.is_empty() {
                    return Ok(bid);
                }
            }

            Err(Error::NoCandidatesRetained)
        }
    }

    #[async_trait]
    impl FunctionLife for FunctionLifeBottomUpImpl {
        /// Here the operation will be sequential, first looking to place on a
        /// bottom node, or a child at least, and only then to consider
        /// itself as a candidate
        async fn bid_on_new_function_and_transmit(
            &self,
            sla: &Sla,
            from: NodeId,
            accumulated_latency: Time,
        ) -> Result<BidProposals, Error> {
            let bid = if let Ok((id, record)) =
                self.auction.bid_on(sla.clone()).await
            {
                BidProposal {
                    node_id: self.node_situation.get_my_id().await,
                    id,
                    bid: record.bid,
                }
            } else {
                let mut follow_up = self
                    .follow_up_to_neighbors(sla, from, accumulated_latency)
                    .await?;
                let bid =
                    follow_up.bids.pop().ok_or(Error::NoCandidatesRetained)?;
                trace!("Transmitting bid to other node...");
                bid
            };

            Ok(BidProposals { bids: vec![bid] })
        }

        async fn validate_bid_and_provision_function(
            &self,
            id: BidId,
        ) -> Result<(), Error> {
            let record = self.auction.validate_bid(&id).await?;
            self.function.provision_function(id, record).await?;
            Ok(())
        }
    }
}
