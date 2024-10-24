use super::*;
use anyhow::anyhow;
use model::domain::sla::Sla;
use model::view::auction::{
    BidProposal, BidProposals, BidRequest, BidRequestOwned,
};
use model::dto::node::NodeDescription;
use model::NodeId;
use uom::fmt::DisplayStyle::Abbreviation;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors<'a>(
        &'a self,
        sla: &'a Sla,
        from: NodeId,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<BidProposals> {
        // Filter nodes
        let nodes: Vec<NodeId> = self
            .node_situation
            .get_neighbors()
            .into_iter()
            .filter(|node| node != &from)
            .collect();

        let mut latencies: Vec<(NodeId, AccumulatedLatency)> =
            Vec::with_capacity(nodes.len());

        // Get all latencies
        for neighbor in nodes {
            let Some(latency) =
                self.neighbor_monitor.get_latency_to(&neighbor).await
            else {
                warn!("Cannot get Latency of {}", neighbor);
                continue;
            };
            let Some(NodeDescription { advertised_bandwidth, .. }) =
                self.node_situation.get_fog_node_neighbor(&neighbor)
            else {
                warn!("Cannot get neighbor bandwidth of {}", neighbor);
                continue;
            };

            let latency = accumulated_latency.accumulate(latency, advertised_bandwidth);

            let worse_lat =
                self.compute_worse_latency(&latency, sla.input_max_size);

            if worse_lat > sla.latency_max {
                trace!(
                    "Skipping neighbor {} because latency is too high \
                     ({})(taking into account sla input size)",
                    neighbor,
                    worse_lat.into_format_args(
                        uom::si::time::millisecond,
                        Abbreviation
                    )
                );
                continue;
            }

            latencies.push((neighbor, latency))
        }

        // Sort by closest

        latencies.sort_by(|(_, a), (_, b)| {
            a.median.partial_cmp(&b.median).unwrap()
        });

        for (neighbor, accumulated_latency) in latencies {
            let Ok(bid) = self
                .node_query
                .request_neighbor_bid(
                    &BidRequest {
                        sla,
                        node_origin: self.node_situation.get_my_id(),
                        accumulated_latency: accumulated_latency.to_owned(),
                    },
                    neighbor.clone(),
                )
                .await
            else {
                continue;
            };
            if !bid.bids.is_empty() {
                return Ok(bid);
            }
        }

        info!("No candidate retained after filtering latencies");
        Ok(BidProposals { bids: vec![] })
    }

    /// Here the operation will be sequential, first looking to place a bid on
    /// itself. If that fails it then probes the others sequentally
    pub async fn bid_on_new_function_and_transmit(
        &self,
        bid_request: &BidRequestOwned,
    ) -> Result<BidProposals> {
        let sla = &bid_request.sla;
        let from = &bid_request.node_origin;
        let accumulated_latency = &bid_request.accumulated_latency;

        let bid = if let Ok(Some((id, record))) =
            self.auction.bid_on(sla.clone(), &accumulated_latency).await
        {
            BidProposal {
                node_id: self.node_situation.get_my_id(),
                id,
                bid: record.bid,
            }
        } else {
            trace!("Transmitting bid to other node...");
            let mut follow_up = self
                .follow_up_to_neighbors(sla, from.clone(), accumulated_latency)
                .await
                .context("Failed to follow up sla to neighbors")?;
            follow_up.bids.pop().ok_or(anyhow!(
                "No canditates were returned after fetching candidates from \
                 neighbors"
            ))?
        };

        Ok(BidProposals { bids: vec![bid] })
    }
}
