extern crate uom;

use std::collections::HashMap;
use std::str::FromStr;

use async_trait::async_trait;
use k8s_openapi::api::core::v1::Node;
use kube::{api::ListParams, Api, Client};
use lazy_regex::regex;
use uom::si::f64::Information;
use uom::si::information::gibibyte;

use manager::kube_metrics::node::NodeMetrics;
use manager::model::dto::k8s::{Allocatable, Metrics, Usage};

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Inherited an error when contacting the k8s API: {0}")]
    Kube(#[from] kube::Error),
    #[error("Unable to obtain the current key: {0}")]
    MissingKey(&'static str),
    #[error("Unable to parse the quantity: {0}")]
    QuantityParsing(String),
}

#[async_trait]
pub trait K8s: Sync + Send {
    async fn get_k8s_metrics(&self) -> Result<HashMap<String, Metrics>, Error>;
}

pub struct K8sImpl;

impl K8sImpl {
    pub fn new() -> Self {
        Self
    }
}
#[async_trait]
impl K8s for K8sImpl {
    async fn get_k8s_metrics(&self) -> Result<HashMap<String, Metrics>, Error> {
        trace!("get_k8s_metrics");
        let mut aggregated_metrics: HashMap<String, Metrics> = HashMap::new();

        let client = Client::try_default().await.map_err(Error::Kube)?;
        let node_metrics: Api<NodeMetrics> = Api::all(client.clone());
        let metrics = node_metrics
            .list(&ListParams::default())
            .await
            .map_err(Error::Kube)?;

        for metric in metrics {
            // let memory = memory.into_format_args(gibibyte, Description);
            let key = metric
                .metadata
                .name
                .ok_or(Error::MissingKey("metadata:name"))?;

            aggregated_metrics.insert(
                key,
                Metrics {
                    usage: Some(Usage {
                        cpu: metric.usage.cpu.0.to_owned(),
                        memory: parse_quantity(&metric.usage.memory.0[..])?,
                    }),
                    allocatable: None,
                },
            );
        }

        let nodes: Api<Node> = Api::all(client.clone());
        let nodes = nodes
            .list(&ListParams::default())
            .await
            .map_err(Error::Kube)?;

        for node in nodes {
            let status = node.status.ok_or(Error::MissingKey("status"))?;
            let allocatable = status.allocatable.ok_or(Error::MissingKey("allocatable"))?;
            let key = node
                .metadata
                .name
                .ok_or(Error::MissingKey("metadata:name"))?;
            let cpu = allocatable.get("cpu").ok_or(Error::MissingKey("cpu"))?;
            let memory = allocatable
                .get("memory")
                .ok_or(Error::MissingKey("memory"))?;

            // let memory = memory.into_format_args(gibibyte, Description);
            aggregated_metrics
                .get_mut(&key)
                .ok_or(Error::MissingKey("metadata:name"))?
                .allocatable = Some(Allocatable {
                cpu: cpu.0.to_owned(),
                memory: parse_quantity(&memory.0[..])?,
            });
        }

        Ok(aggregated_metrics)
    }
}

pub struct K8sFakeImpl;

impl K8sFakeImpl {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait]
impl K8s for K8sFakeImpl {
    async fn get_k8s_metrics(&self) -> Result<HashMap<String, Metrics>, Error> {
        trace!("get_k8s_metrics from fake_impl");
        let mut aggregated_metrics: HashMap<String, Metrics> = HashMap::new();

        aggregated_metrics.insert(
            "toto".to_owned(),
            Metrics {
                usage: Some(Usage {
                    cpu: "900000000n".to_owned(),
                    memory: Information::new::<gibibyte>(2.0),
                }),
                allocatable: Some(Allocatable {
                    cpu: "900000000n".to_owned(),
                    memory: Information::new::<gibibyte>(42.0),
                }),
            },
        );
        Ok(aggregated_metrics)
    }
}

fn parse_quantity<T>(quantity: &str) -> Result<T, Error>
where
    T: FromStr,
{
    let re = regex!(r"^(\d+)(\w+)$");

    let captures = re
        .captures(quantity)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let measure = captures
        .get(1)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;
    let unit = captures
        .get(2)
        .ok_or_else(|| Error::QuantityParsing(quantity.to_string()))?;

    let qty = format!("{} {}B", measure.as_str(), unit.as_str())
        .parse::<T>()
        .map_err(|_| Error::QuantityParsing(quantity.to_string()))?;

    Ok(qty)
}
