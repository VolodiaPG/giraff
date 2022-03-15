use log::trace;
use sla::Sla;

use crate::error::Error;
use crate::k8s::get_k8s_metrics;
use crate::models::Metrics;
use if_chain::if_chain;

pub async fn is_satisfiable(sla: &Sla) -> Result<bool, Error> {
    // TODO: maybe consider other backend than f64 for storing values (ie passing to fixed point u64)

    let aggregated_metrics = get_k8s_metrics().await?;

    Ok(!aggregated_metrics
        .iter()
        .map(|(_key, metrics)| satisfasibility_check(metrics, &sla))
        .all(|res| res == false))
}

pub fn satisfasibility_check(metrics: &Metrics, sla: &Sla) -> bool {
    if_chain! {
        if let Some(allocatable) = &metrics.allocatable;
        if let Some(usage) = &metrics.usage;
        if allocatable.memory - usage.memory > sla.memory;
        then
        {
            trace!("{:?}", (allocatable.memory - usage.memory).into_format_args(uom::si::information::megabyte, uom::fmt::DisplayStyle::Description));

            true
        }
        else
        {
            false
        }
    }
}
