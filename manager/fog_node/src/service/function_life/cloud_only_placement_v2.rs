use super::*;
use anyhow::{Context, Result};
use futures::future::{join, try_join_all};
use model::domain::sla::Sla;
use model::view::auction::{BidProposal, BidProposals, BidRequest};
use model::NodeId;
use uom::si::f64::Time;

impl FunctionLife {
    /// Follow up the [Sla] to the neighbors, and ignore the path where it
    /// came from.
    async fn follow_up_to_neighbors(
        &self,
        sla: &Sla,
        from: NodeId,
        accumulated_latency: Time,
    ) -> Result<BidProposals> {
        let mut requests = vec![];

        if std::env::var("IS_CLOUD").map(|x| x == "is_cloud").unwrap_or(false)
        {
            for neighbor in self.node_situation.get_neighbors() {
                if neighbor == from {
                    continue;
                }

                requests.push((
                    BidRequest {
                        sla,
                        node_origin: self.node_situation.get_my_id(),
                        accumulated_latency,
                    },
                    neighbor,
                ));
            }
        } else if let Some(parent) = self.node_situation.get_parent_id() {
            requests.push((
                BidRequest {
                    sla,
                    node_origin: self.node_situation.get_my_id(),
                    accumulated_latency,
                },
                parent,
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
        accumulated_latency: Time,
    ) -> Result<BidProposals> {
        let (result_bid, proposals) = join(
            self.auction.bid_on(sla.clone(), accumulated_latency),
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
                "Failed to bid and transmit on the sla coming from {}",
                from.clone()
            )
        })?;

        let result_bid = result_bid.context("Failed to bid on the sla")?;

        if std::env::var("IS_CLOUD").map(|x| x == "is_cloud").unwrap_or(false)
        {
            if let Some((bid, bid_record)) = result_bid {
                proposals.bids.push(BidProposal {
                    node_id: my_id,
                    id:      bid,
                    bid:     bid_record.0.bid,
                });
            } else {
                warn!("Bid unsatisfiable, passing on...");
            }
        } else {
            debug!(
                "Node is not a Cloud, c.f. IS_CLOUD env var value: {}",
                std::env::var("IS_CLOUD").unwrap_or(
                    "<no env var IS_CLOUD have been found.>".to_string()
                )
            )
        }

        Ok(proposals)
    }
}
