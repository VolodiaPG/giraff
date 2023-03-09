use super::*;
use anyhow::{anyhow, ensure, Context, Result};

impl FunctionLife {
    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        _from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals> {
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
                    .await
                    .context(
                        "Failed to pass the sla to my parent, towards the \
                         Cloud",
                    )?;

                ensure!(
                    !bid.bids.is_empty(),
                    "Failed to find an available Cloud, the list of bids \
                     (candidates) is empty"
                );
                Ok(bid)
            }
            None => {
                let (id, record) = self
                    .auction
                    .bid_on(sla.clone())
                    .await
                    .context("Failed to bid on sla")?
                    .ok_or_else(|| anyhow!("Cannot accept sla"))?;
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
