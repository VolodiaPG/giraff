use std::sync::Arc;

use manager::model::domain::sla::Sla;
use manager::model::view::auction::Bid;
use manager::model::BidId;

use crate::controller::ControllerError;
use crate::service::function_life::FunctionLife;

/// Return a bid for the SLA.
pub async fn bid_on(sla: Sla, function: &Arc<dyn FunctionLife>) -> Result<Bid, ControllerError> {
    trace!("bidding on... {:?}", sla);
    let (id, record) = function
        .bid_on_new_function(sla)
        .await
        .map_err(ControllerError::from)?;

    Ok(Bid {
        bid: record.bid,
        sla: record.sla,
        id,
    })
}

/// Returns a bid for the SLA.
/// Creates the function on OpenFaaS and use the SLA to enable the limits
pub async fn provision_from_bid(
    id: BidId,
    function: &Arc<dyn FunctionLife>,
) -> Result<(), ControllerError> {
    trace!("Transforming bid into provisioned resource {:?}", id);

    function.validate_bid_and_provision_function(id).await?;

    Ok(())
}
