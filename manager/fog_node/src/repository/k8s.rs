extern crate uom;
use anyhow::{anyhow, bail, Context, Result};
use k8s_openapi::api::core::v1::Node;
use kube::api::ListParams;
use kube::{Api, Client};
use kube_metrics::node::NodeMetrics;
use lazy_regex::regex;
use model::dto::k8s::{Allocatable, Metrics, Usage};
use std::collections::HashMap;
use std::str::FromStr;

pub struct K8s;

impl K8s {
    #[allow(dead_code)]
    pub fn new() -> Self { Self }

    pub async fn get_k8s_metrics(&self) -> Result<HashMap<String, Metrics>> {
        let mut aggregated_metrics: HashMap<String, Metrics> = HashMap::new();

        let client = Client::try_default()
            .await
            .context("Failed to create the K8S client")?;
        let node_metrics: Api<NodeMetrics> = Api::all(client.clone());
        let metrics = node_metrics
            .list(&ListParams::default())
            .await
            .context("Failed to list metrics from the k8s cluster")?;

        for metric in metrics {
            // let memory = memory.into_format_args(gibibyte, Description);
            let key = metric.metadata.name.ok_or(anyhow!(
                "Missing 'metadata:name' key in the retrieved list of \
                 metrics from k8s"
            ))?;

            aggregated_metrics.insert(key,
                                          Metrics { usage:       Some(Usage { cpu:    parse_quantity(&metric.usage.cpu.0[..],
                                                                                                     &MissingUnitType::Complete("ppb") /* https://discuss.kubernetes.io/t/metric-server-cpu-and-memory-units/7497 */)?,
                                                                              memory: parse_quantity(&metric.usage.memory.0[..],
                                                                                                     &MissingUnitType::Suffix("B") /* Bytes */)?, }),
                                                    allocatable: None, });
        }

        let nodes: Api<Node> = Api::all(client.clone());
        let nodes = nodes
            .list(&ListParams::default())
            .await
            .context("Failed to list nodes from the k8s cluster")?;

        for node in nodes {
            let status = node
                .status
                .ok_or(anyhow!(" Missing 'status' key from the k8s node"))?;
            let allocatable = status.allocatable.ok_or(anyhow!(
                " Missing 'allocatable' key from the k8s node"
            ))?;

            let key = node.metadata.name.ok_or(anyhow!(
                " Missing 'metadata:name' key from the k8s node"
            ))?;

            let cpu = allocatable.get("cpu").ok_or(anyhow!(
                " Missing 'cpu' key from the k8s node allocatables"
            ))?;

            let memory = allocatable.get("memory").ok_or(anyhow!(
                " Missing 'memory' key from the k8s node allocatables"
            ))?;

            // let memory = memory.into_format_args(gibibyte, Description);
            aggregated_metrics
                .get_mut(&key)
                .ok_or(anyhow!(
                    " Missing 'metadata:name' key from the k8s metrics"
                ))?
                .allocatable = Some(Allocatable {
                cpu:    parse_quantity(
                    &cpu.0[..],
                    &MissingUnitType::Complete(""),
                )?, /* https://discuss.kubernetes.io/t/metric-server-cpu-and-memory-units/7497 */
                memory: parse_quantity(
                    &memory.0[..],
                    &MissingUnitType::Suffix("B"),
                )?, /* Bytes */
            });
        }

        Ok(aggregated_metrics)
    }
}
enum MissingUnitType<'a> {
    /// Missing just the rightmost part, eg. B for Bytes
    Suffix(&'a str),
    /// the whole unit needs to be replaced, eg. received xxxxnano,
    /// replaced by xxxx nanocpu
    Complete(&'a str),
}

fn parse_quantity<T>(
    quantity: &str,
    missing_unit: &MissingUnitType,
) -> Result<T>
where
    T: FromStr,
{
    let re = regex!(r"^(\d+)(\w*)$");

    let captures = re
        .captures(quantity)
        .with_context(|| format!("Failed to parse quantity '{}'", quantity))?;
    let measure = captures.get(1).with_context(|| {
        format!("Failed to get part of parsed quantity '{}'", quantity)
    })?;
    let prefix = captures.get(2).map(|cap| cap.as_str()).unwrap_or("");
    //.ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;

    let unit = match missing_unit {
        MissingUnitType::Suffix(suffix) => format!("{prefix}{suffix}"),
        MissingUnitType::Complete(complete) => complete.to_string(),
    };

    let qty = format!("{} {}", measure.as_str(), unit).parse::<T>();
    let Ok(qty) = qty else{
            bail!(
                "Failed to parse the quantity string to the final required \
                 type '{}'",
                quantity.to_string()
            )
        };

    Ok(qty)
}

#[cfg(test)]
mod tests {
    use helper::uom_helper::cpu_ratio::nanocpu;
    use uom::fmt::DisplayStyle::Abbreviation;
    use uom::si::f64::{Information, Ratio};
    use uom::si::information::byte;
    // Note this useful idiom: importing names from outer (for mod tests)
    // scope.
    use super::*;

    #[test]
    fn test_ratio_cpu() -> Result<()> {
        assert_eq!(
            format!(
                "{}",
                parse_quantity::<Ratio>(
                    "1024n",
                    &MissingUnitType::Complete("ppb")
                )?
                .into_format_args(nanocpu, Abbreviation)
            ),
            "1023.9999999999999 nanocpu".to_owned()
        );

        Ok(())
    }

    #[test]
    fn test_quantity_memory() -> Result<()> {
        assert_eq!(
            format!(
                "{}",
                parse_quantity::<Information>(
                    "1024",
                    &MissingUnitType::Suffix("B")
                )?
                .into_format_args(byte, Abbreviation)
            ),
            "1024 B".to_owned()
        );

        Ok(())
    }
}
