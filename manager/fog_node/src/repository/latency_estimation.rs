use crate::NodeSituation;
use anyhow::{ensure, Context, Result};
use model::domain::exp_average::ExponentialMovingAverage;
use model::NodeId;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::Time;

#[derive(Debug)]
pub struct IndividualErrorList {
    list: Vec<anyhow::Error>,
}

impl fmt::Display for IndividualErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<anyhow::Error>> for IndividualErrorList {
    fn from(list: Vec<anyhow::Error>) -> Self { IndividualErrorList { list } }
}

#[derive(Debug)]
pub struct LatencyEstimation {
    node_situation: Arc<NodeSituation>,
    latency:        Arc<dashmap::DashMap<NodeId, ExponentialMovingAverage>>,
    alpha:          model::domain::exp_average::Alpha,
}

impl LatencyEstimation {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        alpha: model::domain::exp_average::Alpha,
    ) -> Self {
        Self {
            node_situation,
            latency: Arc::new(dashmap::DashMap::new()),
            alpha,
        }
    }

    async fn ping(&self, node: NodeId) -> Result<()> {
        let ip = self
            .node_situation
            .get_fog_node_neighbor(&node)
            .with_context(|| format!("Failed to find node {node}"))?
            .ip;

        let (_, dur) = surge_ping::ping(ip, &[0; 32])
            .await
            .with_context(|| format!("Ping to {ip} failed"))?;

        let latency = Time::new::<uom::si::time::millisecond>(
            dur.as_millis() as f64 / 2.0,
        );

        crate::prom_metrics::LATENCY_NEIGHBORS_GAUGE
            .with_label_values(&[&format!("{ip}")])
            .set(latency.value);
        let latency = self.update_latency(&self.latency, &node, latency);

        crate::prom_metrics::LATENCY_NEIGHBORS_AVG_GAUGE
            .with_label_values(&[&format!("{ip}")])
            .set(latency.value);

        Ok(())
    }

    pub async fn latency_to_neighbors(&self) -> Result<()> {
        let neighbors = self.node_situation.get_neighbors();
        let mut handles = Vec::with_capacity(neighbors.len());
        for node in neighbors {
            handles.push(self.ping(node));
        }

        let errors: Vec<anyhow::Error> = futures::future::join_all(handles)
            .await
            .into_iter()
            .filter(|result| result.is_err())
            .map(|result| result.err().unwrap())
            .collect();

        ensure!(
            errors.is_empty(),
            "Failed to ping for latency estimation because of {}",
            IndividualErrorList::from(errors)
        );
        Ok(())
    }

    pub async fn get_latency_to(&self, id: &NodeId) -> Option<Time> {
        self.latency.get(id).map(|x| x.get())
    }

    fn update_latency(
        &self,
        map: &dashmap::DashMap<NodeId, ExponentialMovingAverage>,
        key: &NodeId,
        value: Time,
    ) -> Time {
        let Some(mut entry) = map.get_mut(key)
            else{
                map.insert(
                    key.clone(),
                    ExponentialMovingAverage::new(self.alpha.clone(), value),
                );
                return value;
            };

        entry.value_mut().update(value)
    }
}
