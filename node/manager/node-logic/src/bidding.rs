use log::trace;
use sla::Sla;

use crate::error::Error;
use crate::k8s::get_k8s_metrics;
use if_chain::if_chain;

pub async fn bid(sla: &Sla) -> Result<f64, Error> {
    // TODO: maybe consider other backend than f64 for storing values (ie passing to fixed point u64)

    let aggregated_metrics = get_k8s_metrics().await?;

    if_chain! {
        if let Some((name, metrics)) = aggregated_metrics.iter().find(|(_key, metrics)| crate::satisfiability::satisfasibility_check(metrics, sla));
        if let Some(allocatable) = &metrics.allocatable;
        if let Some(usage) = &metrics.usage;
        then
        {
            let price = (allocatable.memory - usage.memory)/sla.memory;
            let price: f64 = price.into();
            trace!("price on {:?} is {:?}", name, price);
            Ok(price)
        }
        else
        {
            Err(Error::Unsatisfiable)
        }
    }
}
