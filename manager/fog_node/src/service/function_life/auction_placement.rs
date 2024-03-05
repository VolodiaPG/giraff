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
    async fn follow_up_to_single_neighbor(
        &self,
        neighbor: &NodeId,
        sla: &Sla,
        from: &NodeId,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<Option<BidProposals>> {
        if neighbor == from {
            return Ok(None);
        }
        let Some(latency) =
            self.neighbor_monitor.get_latency_to(neighbor).await
        else {
            warn!("Cannot get Latency of {}", neighbor);
            return Ok(None);
        };

        let accumulated_latency_to_next_node =
            accumulated_latency.accumulate(latency);

        let would_be_lat = self.compute_latency(
            &accumulated_latency_to_next_node,
            sla.input_max_size,
        );
        let worse_lat = would_be_lat.median + would_be_lat.median_uncertainty;

        if worse_lat > sla.latency_max {
            let latency_outbound = accumulated_latency_to_next_node.median;
            debug!(
                "Skipping neighbor {} because latency is too high ({}, a \
                 total of {}), taking the sla input size into account.",
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
            return Ok(None);
        }
        let request = BidRequest {
            sla,
            node_origin: self.node_situation.get_my_id(),
            accumulated_latency: accumulated_latency_to_next_node,
        };

        let bid = self
            .node_query
            .request_neighbor_bid(&request, neighbor.clone())
            .await?;
        Ok(Some(bid))
    }

    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: &AccumulatedLatency,
    ) -> Result<BidProposals> {
        let neighbors = self.node_situation.get_neighbors();
        let promises = neighbors.iter().map(|neighbor| {
            self.follow_up_to_single_neighbor(
                neighbor,
                sla,
                &from,
                accumulated_latency,
            )
        });

        Ok(BidProposals {
            bids: try_join_all(promises)
                .await
                .context("Failed to get all neighboring bids")?
                .into_iter()
                .flatten()
                .flat_map(|proposals| proposals.bids)
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
