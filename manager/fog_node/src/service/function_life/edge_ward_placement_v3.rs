use super::*;
use anyhow::{anyhow, Context, Result};
use model::domain::sla::Sla;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::NodeId;
use uom::si::f64::Time;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_parent<'a>(
        &'a self,
        sla: &'a Sla,
        accumulated_latency: Time,
    ) -> Result<BidProposals> {
        let Some(parent) = self
            .node_situation
            .get_parent_id()
            else{
                return Ok(BidProposals {
                    bids: vec!()
                });
            };

        let latency_outbound = self
            .neighbor_monitor
            .get_latency_to_avg(&parent)
            .await
            .ok_or_else(|| anyhow!("Cannot get Latency of {}", parent))?;

        if latency_outbound + accumulated_latency < sla.latency_max {
            let bid = self
                .node_query
                .request_neighbor_bid(
                    &BidRequest {
                        sla,
                        node_origin: self.node_situation.get_my_id(),
                        accumulated_latency: accumulated_latency.to_owned()
                            + latency_outbound,
                    },
                    parent,
                )
                .await
                .context("Failed to request a bid from my parent")?;
            if !bid.bids.is_empty() {
                return Ok(bid);
            }
        };

        Ok(BidProposals { bids: vec![] })
    }

    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        _from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals> {
        trace!("Transmitting bid to other node...");
        let mut follow_up = self
            .follow_up_to_parent(sla, accumulated_latency)
            .await
            .context("Failed to follow up sla to my parent")?;
        let bid = follow_up.bids.pop();
        let bids = match bid {
            Some(bid) => {
                debug!("Using bid coming from above");
                vec![bid]
            }
            None => {
                if let Ok(Some((id, record))) =
                    self.auction.bid_on(sla.clone()).await
                {
                    info!("no bids are coming from above, bidded.");
                    vec![BidProposal {
                        node_id: self.node_situation.get_my_id(),
                        id,
                        bid: record.0.bid,
                    }]
                } else {
                    info!(
                        "no bids are coming from above, cannot bid, passing."
                    );
                    vec![]
                }
            }
        };
        Ok(BidProposals { bids })
    }
}
