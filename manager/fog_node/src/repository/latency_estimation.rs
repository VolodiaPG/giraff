use crate::NodeSituation;
use anyhow::{ensure, Context, Result};
use model::domain::exp_average::ExponentialMovingAverage;
use model::NodeId;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;
use uom::si::f64::Time;

#[cfg(feature = "jaeger")]
type HttpClient = reqwest_middleware::ClientWithMiddleware;
#[cfg(not(feature = "jaeger"))]
type HttpClient = reqwest::Client;

#[derive(Debug)]
pub struct IndividualErrorList {
    list: Vec<(NodeId, anyhow::Error)>,
}

impl fmt::Display for IndividualErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<(NodeId, anyhow::Error)>> for IndividualErrorList {
    fn from(list: Vec<(NodeId, anyhow::Error)>) -> Self {
        IndividualErrorList { list }
    }
}

#[derive(Debug)]
pub struct LatencyEstimation {
    node_situation:     Arc<NodeSituation>,
    outgoing_latencies:
        Arc<dashmap::DashMap<NodeId, ExponentialMovingAverage>>,
    incoming_latencies:
        Arc<dashmap::DashMap<NodeId, ExponentialMovingAverage>>,
    client:             Arc<HttpClient>,
    alpha:              model::domain::exp_average::Alpha,
}

impl LatencyEstimation {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        client: Arc<HttpClient>,
        alpha: model::domain::exp_average::Alpha,
    ) -> Self {
        Self {
            node_situation,
            outgoing_latencies: Arc::new(dashmap::DashMap::new()),
            incoming_latencies: Arc::new(dashmap::DashMap::new()),
            client,
            alpha,
        }
    }

    /// Do the packet exchanges to get the latencies and return (outgoing,
    /// incoming)
    async fn make_latency_request_to(
        &self,
        node_id: &NodeId,
    ) -> Result<(Time, Time)> {
        let desc =
            self.node_situation.get_fog_node_neighbor(node_id).with_context(
                || format!("Failed to find neighbor node {}", node_id),
            )?;

        let ip = desc.ip;
        let port = desc.port_http;

        let sent_at = Instant::now();
        let _response = self
            .client
            .get(format!("http://{ip}:{port}/api/health").as_str())
            .send()
            .await?;
        let elapsed = sent_at.elapsed().as_millis();

        // TCP+HTTP: travel time = 2 * outgoing + 2 * incoming
        // here we suppose incoming = outgoing

        let latency = elapsed as f64 / 4.0;
        let outgoing_latency =
            Time::new::<uom::si::time::millisecond>(latency);
        let incoming_latency = outgoing_latency;

        Ok((outgoing_latency, incoming_latency))
    }

    pub async fn latency_to_neighbors(&self) -> Result<()> {
        let mut handles = Vec::new();
        let mut tried_nodes = Vec::new(); // same order as handles
        for node in self.node_situation.get_neighbors() {
            tried_nodes.push(node.clone());
            handles.push(async move {
                let (incoming, outgoing) =
                    self.make_latency_request_to(&node).await?;

                self.update_latency(&self.incoming_latencies, &node, incoming);
                self.update_latency(&self.outgoing_latencies, &node, outgoing);

                let desc = self
                    .node_situation
                    .get_fog_node_neighbor(&node)
                    .with_context(|| {
                        format!("Failed to find neighbor node {}", node)
                    })?;

                let ip = desc.ip;
                let port = desc.port_http;
                crate::prom_metrics::LATENCY_NEIGHBORS_GAUGE
                    .with_label_values(&[&format!("{ip}:{port}")])
                    .set(outgoing.value);

                let Some(lat) = self.outgoing_latencies.get(&node) else {
                    return Ok(());
                };
                crate::prom_metrics::LATENCY_NEIGHBORS_AVG_GAUGE
                    .with_label_values(&[&format!("{ip}:{port}")])
                    .set(lat.get().value);
                Ok(())
            });
        }

        let errors: Vec<(NodeId, anyhow::Error)> =
            futures::future::join_all(handles)
                .await
                .into_iter()
                .zip(tried_nodes.into_iter())
                .filter(|(result, _id)| result.is_err())
                .map(|(result, id)| (id, result.err().unwrap()))
                .collect();

        ensure!(
            errors.is_empty(),
            "Failed to ping for latency estimation because of {}",
            IndividualErrorList::from(errors)
        );
        Ok(())
    }

    pub async fn get_latency_to(&self, id: &NodeId) -> Option<Time> {
        self.outgoing_latencies.get(id).map(|x| x.get())
    }

    fn update_latency(
        &self,
        map: &dashmap::DashMap<NodeId, ExponentialMovingAverage>,
        key: &NodeId,
        value: Time,
    ) {
        let mut entry =
            map.get(key).map(|entry| entry.value().clone()).unwrap_or_else(
                || ExponentialMovingAverage::new(self.alpha.clone(), value),
            );
        entry.update(value);
        map.insert(key.clone(), entry);
    }
}
