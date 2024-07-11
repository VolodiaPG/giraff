use super::*;
use anyhow::ensure;
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
        accumulated_latency: AccumulatedLatency,
    ) -> Result<BidProposals> {
        let Some(parent) = self.node_situation.get_parent_id() else {
            return Ok(BidProposals { bids: vec![] });
        };
        let bid = self
            .node_query
            .request_neighbor_bid(
                &BidRequest {
                    sla,
                    node_origin: self.node_situation.get_my_id(),
                    accumulated_latency: accumulated_latency.to_owned(),
                },
                parent,
            )
            .await
            .context("Failed to request a bid from my parent")?;

        ensure!(
            !bid.bids.is_empty(),
            "The next candidates did not returned any bid"
        );

        Ok(bid)
    }

    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        bid_request: &BidRequestOwned,
    ) -> Result<BidProposals> {
        let sla = &bid_request.sla;
        let accumulated_latency = &bid_request.accumulated_latency;

        let bid = if let Ok(Some((id, record))) =
            self.auction.bid_on(sla.clone(), accumulated_latency).await
        {
            BidProposal {
                node_id: self.node_situation.get_my_id(),
                id,
                bid: record.bid,
            }
        } else {
            trace!("Transmitting bid to other node...");
            let mut follow_up = self
                .follow_up_to_parent(sla, accumulated_latency.clone())
                .await
                .context("Failed to follow up sla to my parent")?;
            follow_up.bids.pop().ok_or(anyhow!(
                "No canditates were returned after fetching candidates \
                 starting from my parent"
            ))?
        };

        Ok(BidProposals { bids: vec![bid] })
    }
}
