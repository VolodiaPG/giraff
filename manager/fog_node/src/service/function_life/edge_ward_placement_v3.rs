use super::*;
use anyhow::anyhow;
use model::domain::sla::Sla;
use model::view::auction::{
    BidProposal, BidProposals, BidRequest, BidRequestOwned,
};
use tracing::debug;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_parent<'a>(
        &'a self,
        sla: &'a Sla,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<BidProposals> {
        let Some(parent) = self.node_situation.get_parent_id() else {
            return Ok(BidProposals { bids: vec![] });
        };

        let latency = self
            .neighbor_monitor
            .get_latency_to(&parent)
            .await
            .ok_or_else(|| anyhow!("Cannot get Latency of {}", parent))?;

        let accumulated_latency = accumulated_latency.accumulate(latency);

        if accumulated_latency.median < sla.latency_max {
            let bid = self
                .node_query
                .request_neighbor_bid(
                    &BidRequest {
                        sla,
                        node_origin: self.node_situation.get_my_id(),
                        accumulated_latency,
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
        bid_request: &BidRequestOwned,
    ) -> Result<BidProposals> {
        todo!("Maintance to be done");
        let sla = &bid_request.sla;
        let accumulated_latency = &bid_request.accumulated_latency;

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
                if let Ok(Some((id, record))) = self
                    .auction
                    .bid_on(sla.clone(), &accumulated_latency)
                    .await
                {
                    info!("no bids are coming from above, bidded.");
                    vec![BidProposal {
                        node_id: self.node_situation.get_my_id(),
                        id,
                        bid: record.bid,
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
