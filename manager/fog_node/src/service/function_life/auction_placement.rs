use super::*;
use crate::service::auction;
use futures::future::{join, try_join_all};
use uom::fmt::DisplayStyle::Abbreviation;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error> {
        let mut requests = vec![];

        for neighbor in self.node_situation.get_neighbors() {
            if neighbor == from {
                continue;
            }
            let Some(latency_outbound) = self
                    .neighbor_monitor
                    .get_latency_to_avg(&neighbor)
                    .await
                    else {
                        warn!("Cannot get Latency of {}", neighbor);
                        continue;
                    };
            if latency_outbound + accumulated_latency > sla.latency_max {
                debug!(
                    "Skipping neighbor {} because latency is too high ({}, a \
                     total of {}).",
                    neighbor,
                    latency_outbound.into_format_args(
                        uom::si::time::millisecond,
                        Abbreviation
                    ),
                    (latency_outbound + accumulated_latency).into_format_args(
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
                        + latency_outbound,
                },
                neighbor,
            ));
        }

        let promises = requests.iter().map(|(request, neighbor)| {
            self.node_query.request_neighbor_bid(request, neighbor.clone())
        });

        Ok(BidProposals {
            bids: try_join_all(promises)
                .await?
                .into_iter()
                .flat_map(|proposals: BidProposals| proposals.bids)
                .collect(),
        })
    }

    pub async fn bid_on_new_function_and_transmit(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals, Error> {
        let (result_bid, proposals) = join(
            self.auction.bid_on(sla.clone()),
            self.follow_up_to_neighbors(sla, from, accumulated_latency),
        )
        .await;
        let my_id = self.node_situation.get_my_id();

        let mut proposals = proposals?;

        match result_bid {
            Err(auction::Error::Unsatisfiable) => {
                warn!("Bid unsatisfiable, passing on...");
            }
            _ => {
                // let the error happen if it something else
                let (bid, bid_record) = result_bid?;
                proposals.bids.push(BidProposal {
                    node_id: my_id,
                    id:      bid,
                    bid:     bid_record.0.bid,
                });
            }
        }

        Ok(proposals)
    }
}
