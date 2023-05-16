use super::*;
use anyhow::{Context, Result};
use futures::future::{join, try_join_all};
use model::domain::sla::Sla;
use model::view::auction::{
    AccumulatedLatency, BidProposal, BidProposals, BidRequest,
};
use model::NodeId;
use uom::fmt::DisplayStyle::Abbreviation;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<BidProposals> {
        let mut requests = vec![];

        for neighbor in self.node_situation.get_neighbors() {
            if neighbor == from {
                continue;
            }
            let Some(latency) = self
                    .neighbor_monitor
                    .get_latency_to(&neighbor)
                    .await
                    else {
                        warn!("Cannot get Latency of {}", neighbor);
                        continue;
                    };

            if latency.median
                + accumulated_latency.median
                + accumulated_latency.median_uncertainty
                > sla.latency_max
            {
                let latency_outbound = latency.median;
                debug!(
                    "Skipping neighbor {} because latency is too high ({}, a \
                     total of {}).",
                    neighbor,
                    latency_outbound.into_format_args(
                        uom::si::time::millisecond,
                        Abbreviation
                    ),
                    (latency_outbound + accumulated_latency.median)
                        .into_format_args(
                            uom::si::time::millisecond,
                            Abbreviation
                        ),
                );
                continue;
            }
            requests.push((
                BidRequest {
                    sla,
                    node_origin: self.node_situation.get_my_id(),
                    accumulated_latency: accumulated_latency
                        .accumulate(latency),
                },
                neighbor,
            ));
        }

        let promises = requests.iter().map(|(request, neighbor)| {
            self.node_query.request_neighbor_bid(request, neighbor.clone())
        });

        Ok(BidProposals {
            bids: try_join_all(promises)
                .await
                .context("Failed to get all neighboring bids")?
                .into_iter()
                .flat_map(|proposals: BidProposals| proposals.bids)
                .collect(),
        })
    }

    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: AccumulatedLatency,
    ) -> Result<BidProposals> {
        let (result_bid, proposals) = join(
            self.auction.bid_on(sla.clone(), &accumulated_latency),
            self.follow_up_to_neighbors(
                sla,
                from.clone(),
                &accumulated_latency,
            ),
        )
        .await;
        let my_id = self.node_situation.get_my_id();

        let mut proposals = proposals.with_context(|| {
            format!(
                "Failed to bid an transmit on the sla coming from {}",
                from.clone()
            )
        })?;

        let result_bid = result_bid.context("Failed to bid on the sla")?;

        if let Some((bid, bid_record)) = result_bid {
            proposals.bids.push(BidProposal {
                node_id: my_id,
                id:      bid,
                bid:     bid_record.0.bid,
            });
        } else {
            warn!("Bid unsatisfiable, passing on...");
        }

        Ok(proposals)
    }
}
