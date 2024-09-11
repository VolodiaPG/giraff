use super::*;
use futures::future::{join, join_all};
use model::domain::sla::Sla;
use model::dto::node::NodeDescription;
use model::view::auction::{
    BidProposal, BidProposals, BidRequest, BidRequestOwned,
};
use model::NodeId;
use tracing::debug;
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

        let Some(NodeDescription { advertised_bandwidth, .. }) =
            self.node_situation.get_fog_node_neighbor(neighbor)
        else {
            warn!("Cannot neighbor bandwidth of {}", neighbor);
            return Ok(None);
        };

        let accumulated_latency_to_next_node =
            accumulated_latency.accumulate(latency, advertised_bandwidth);

        let worse_lat = self.compute_worse_latency(
            &accumulated_latency_to_next_node,
            sla.input_max_size,
        );

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
            bids: join_all(promises)
                .await
                .into_iter()
                .filter_map(|result| match result {
                    Ok(value) => Some(value),
                    Err(e) => {
                        error!("Encountered an error: {:?}", e);
                        None
                    }
                })
                .filter_map(|opt| match opt {
                    Some(value) => Some(value),
                    None => {
                        warn!("Empty bid from neighbor");
                        None
                    }
                })
                .flat_map(|proposals| proposals.bids)
                .collect(),
        })
    }

    pub async fn bid_on_new_function_and_transmit(
        &self,
        bid_request: &BidRequestOwned,
    ) -> Result<BidProposals> {
        let sla = &bid_request.sla;
        let from = &bid_request.node_origin;
        let accumulated_latency = &bid_request.accumulated_latency;

        let (result_bid, proposals) = join(
            self.auction.bid_on(sla.clone(), &accumulated_latency),
            self.follow_up_to_neighbors(
                sla,
                from.clone(),
                accumulated_latency,
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
                bid:     bid_record.bid,
            });
        } else {
            warn!("Bid unsatisfiable, passing on...");
        }

        Ok(proposals)
    }
}
