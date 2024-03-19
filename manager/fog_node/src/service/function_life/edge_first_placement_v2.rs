use super::*;
use model::domain::sla::Sla;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::NodeId;
use uom::fmt::DisplayStyle::Abbreviation;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors<'a>(
        &'a self,
        sla: &'a Sla,
        from: &NodeId,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<BidProposals> {
        let neighbors = self.node_situation.get_neighbors();
        let mut latencies: Vec<(NodeId, AccumulatedLatency)> =
            Vec::with_capacity(neighbors.len());

        // Get all latencies
        for neighbor in neighbors {
            if neighbor == *from {
                continue;
            }

            let Some(latency) =
                self.neighbor_monitor.get_latency_to(&neighbor).await
            else {
                warn!("Cannot get Latency of {}", neighbor);
                continue;
            };

            let latency = accumulated_latency.accumulate(latency);

            let worse_lat =
                self.compute_worse_latency(&latency, sla.input_max_size);

            if worse_lat > sla.latency_max {
                trace!(
                    "Skipping neighbor {} because latency is too high ({}). \
                     (taking into account the sla input size)",
                    neighbor,
                    latency.median.into_format_args(
                        uom::si::time::millisecond,
                        Abbreviation
                    )
                );
                continue;
            }

            latencies.push((neighbor, latency))
        }

        // Sort by furthest: we want to keep the best for when we need it

        latencies.sort_by(|(_, a), (_, b)| {
            b.median.partial_cmp(&a.median).unwrap()
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

    /// Here the operation will be sequential, first looking to place on a
    /// bottom node, or a child at least, and only then to consider
    /// itself as a candidate
    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: AccumulatedLatency,
    ) -> Result<BidProposals> {
        trace!("Transmitting bid to other nodes...");
        let mut follow_up = self
            .follow_up_to_neighbors(sla, &from, &accumulated_latency)
            .await
            .context("Failed to follow up sla to my neighbors")?;
        let bid = follow_up.bids.pop();
        let bids = match bid {
            Some(bid) => {
                debug!("Using bid coming from a neighbor");
                vec![bid]
            }
            None => {
                if let Ok(Some((id, record))) = self
                    .auction
                    .bid_on(sla.clone(), &accumulated_latency)
                    .await
                {
                    info!("no bids are coming from any neighbors, bidded.");
                    vec![BidProposal {
                        node_id: self.node_situation.get_my_id(),
                        id,
                        bid: record.0.bid,
                    }]
                } else {
                    info!(
                        "no bids are coming from none of my neighbors + \
                         cannot bid, passing."
                    );
                    vec![]
                }
            }
        };
        Ok(BidProposals { bids })
    }
}
