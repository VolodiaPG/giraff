use std::sync::Arc;

use manager::model::{view::auction::{BidProposals, BidRequest},
                     BidId};

use crate::{controller::ControllerError, service::function_life::FunctionLife};

/// Return a bid for the SLA. And makes the follow up to ask other nodes for their bids.
pub async fn bid_on(bid_request: BidRequest,
                    function: &Arc<dyn FunctionLife>)
                    -> Result<BidProposals, ControllerError> {
    trace!("bidding on... {:?}", bid_request);
    function.bid_on_new_function_and_transmit(bid_request.sla,
                                              bid_request.node_origin,
                                              bid_request.accumulated_latency)
            .await
            .map_err(ControllerError::from)
}

/// Returns a bid for the SLA.
/// Creates the function on OpenFaaS and use the SLA to enable the limits
pub async fn provision_from_bid(id: BidId,
                                function: &Arc<dyn FunctionLife>)
                                -> Result<(), ControllerError> {
    trace!("Transforming bid into provisioned resource {:?}", id);

    function.validate_bid_and_provision_function(id).await?;

    Ok(())
}
