use crate::monitoring::SlaSeen;
use crate::service::function_life::FunctionLife;
use anyhow::{Context, Result};
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use model::view::auction::{BidProposals, BidRequestOwned};
use model::BidId;
use std::sync::Arc;
use uom::si::time::second;

/// Return a bid for the SLA. And makes the follow up to ask other nodes for
/// their bids.
pub async fn bid_on(
    bid_request: BidRequestOwned,
    function: &Arc<FunctionLife>,
    metrics: &Arc<MetricsExporter>,
) -> Result<BidProposals> {
    trace!("bidding on... {:?}", bid_request);
    metrics
        .observe(SlaSeen {
            n: 1,
            accumulated_latency_median: bid_request
                .accumulated_latency
                .median
                .get::<second>(),
            accumulated_latency_median_uncertainty: bid_request
                .accumulated_latency
                .median_uncertainty
                .get::<second>(),
            function_name: bid_request.sla.function_live_name.clone(),
            sla_id: bid_request.sla.id.to_string(),
            timestamp: Utc::now(),
        })
        .await?;
    function
        .bid_on_new_function_and_transmit(
            &bid_request.sla,
            bid_request.node_origin,
            bid_request.accumulated_latency,
        )
        .await
        .context("Failed to bid on function and transmit it to neighbors")
}

/// Returns a bid for the SLA.
/// Creates the function on OpenFaaS and use the SLA to enable the limits
pub async fn provision_from_bid(
    id: BidId,
    function: &Arc<FunctionLife>,
) -> Result<()> {
    trace!("Transforming bid into provisioned resource {:?}", id);
    function.provision_function(id.clone()).await.with_context(|| {
        format!("Failed to provision function from bid {}", id)
    })?;
    Ok(())
}
