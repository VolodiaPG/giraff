use super::*;

impl FunctionLife {
    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        _from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error> {
        match self.node_situation.get_parent_id() {
            Some(parent) => {
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
                    .await?;
                if !bid.bids.is_empty() {
                    return Ok(bid);
                }

                Err(Error::NoCloudAvailable)
            }
            None => {
                let Ok((id, record)) =self.auction.bid_on(sla.clone()).await else {
                        return Err(Error::NoCloudAvailable);
                    };
                Ok(BidProposals {
                    bids: vec![BidProposal {
                        node_id: self.node_situation.get_my_id(),
                        id,
                        bid: record.0.bid,
                    }],
                })
            }
        }
    }
}
