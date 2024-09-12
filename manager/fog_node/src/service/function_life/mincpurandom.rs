use super::*;
use futures::future::try_join_all;
use model::domain::sla::Sla;
use model::dto::node::NodeDescription;
use model::view::auction::{
    BidProposal, BidProposals, BidRequest, BidRequestOwned,
};
use model::NodeId;
use rand::seq::SliceRandom;
use rand::thread_rng;
use tracing::debug;
use uom::fmt::DisplayStyle::Abbreviation;

impl FunctionLife {
    async fn follow_up_to_single_neighbor(
        &self,
        neighbor: &NodeId,
        sla: &Sla,
        from: &NodeId,
        accumulated_latency: &AccumulatedLatency,
        nb_propositions_required: usize,
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
                (worse_lat).into_format_args(
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
            nb_propositions_required,
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
        number_neighbors_to_follow_to: usize,
    ) -> Result<BidProposals> {
        let mut neighbors = self.node_situation.get_neighbors();
        neighbors.shuffle(&mut thread_rng());

        let nb_propositions_required =
            if number_neighbors_to_follow_to <= neighbors.len() {
                1
            } else {
                (number_neighbors_to_follow_to as f64 / neighbors.len() as f64)
                    .ceil() as usize
            };

        let promises = neighbors
            .iter()
            .take(number_neighbors_to_follow_to)
            .map(|neighbor| {
                self.follow_up_to_single_neighbor(
                    neighbor,
                    sla,
                    &from,
                    accumulated_latency,
                    nb_propositions_required,
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
        bid_request: &BidRequestOwned,
    ) -> Result<BidProposals> {
        let sla = &bid_request.sla;
        let from = &bid_request.node_origin;
        let accumulated_latency = &bid_request.accumulated_latency;
        let mut nb_propositions_required =
            bid_request.nb_propositions_required;

        if nb_propositions_required == 0 {
            return Ok(BidProposals { bids: vec![] });
        }

        let my_id = self.node_situation.get_my_id();
        let my_proposal =
            match self.auction.bid_on(sla.clone(), &accumulated_latency).await
            {
                Ok(Some((bid_id, bid_record))) => Some(BidProposal {
                    node_id: my_id,
                    id:      bid_id,
                    bid:     bid_record.bid,
                }),
                _ => {
                    warn!("Bid unsatisfiable, passing on...");
                    None
                }
            };

        if my_proposal.is_some() {
            nb_propositions_required -= 1;
        }

        let proposals = self.follow_up_to_neighbors(
            sla,
            from.clone(),
            accumulated_latency,
            nb_propositions_required,
        );
        let mut proposals = proposals.await.with_context(|| {
            format!(
                "Failed to bid an transmit on the sla coming from {}",
                from.clone()
            )
        })?;

        if let Some(my_proposal) = my_proposal {
            proposals.bids.push(my_proposal);
        }

        Ok(proposals)
    }
}
