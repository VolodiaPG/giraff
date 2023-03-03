use super::*;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_parent<'a>(
        &'a self,
        sla: &'a Sla,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error> {
        if let Ok(bid) = self
            .node_query
            .request_neighbor_bid(
                &BidRequest {
                    sla,
                    node_origin: self.node_situation.get_my_id(),
                    accumulated_latency: accumulated_latency.to_owned(),
                },
                self.node_situation
                    .get_parent_id()
                    .ok_or(Error::NoCandidatesRetained)?,
            )
            .await
        {
            if !bid.bids.is_empty() {
                return Ok(bid);
            }
        }

        Err(Error::NoCandidatesRetained)
    }

    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        _from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error> {
        let bid =
            if let Ok((id, record)) = self.auction.bid_on(sla.clone()).await {
                BidProposal {
                    node_id: self.node_situation.get_my_id(),
                    id,
                    bid: record.0.bid,
                }
            } else {
                let mut follow_up =
                    self.follow_up_to_parent(sla, accumulated_latency).await?;
                trace!("Transmitting bid to other node...");
                follow_up.bids.pop().ok_or(Error::NoCandidatesRetained)?
            };

        Ok(BidProposals { bids: vec![bid] })
    }
}
