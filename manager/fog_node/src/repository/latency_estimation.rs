use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use uom::si::f64::Time;

use model::domain::rolling_avg::RollingAvg;
use model::NodeId;

use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Rtt estimation was carried for {0} nodes, got {1} errors: {2}")]
    FailedPing(usize, usize, IndividualErrorList),
}

#[derive(Debug, thiserror::Error)]
pub enum IndividualError {
    #[error("Did not found node: {0}")]
    NodeNotFound(NodeId),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct IndividualErrorList {
    list: Vec<(NodeId, IndividualError)>,
}

impl fmt::Display for IndividualErrorList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.list)
    }
}

impl From<Vec<(NodeId, IndividualError)>> for IndividualErrorList {
    fn from(list: Vec<(NodeId, IndividualError)>) -> Self {
        IndividualErrorList { list }
    }
}

#[async_trait]
pub trait LatencyEstimation: Debug + Sync + Send {
    /// Make the requests to the neighbors to get the Latency to our children +
    /// parent.
    async fn latency_to_neighbors(&self) -> Result<(), Error>;
    async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time>;
    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time>;
}

#[derive(Debug)]
pub struct LatencyEstimationImpl {
    node_situation:     Arc<dyn NodeSituation>,
    outgoing_latencies: Arc<dashmap::DashMap<NodeId, RollingAvg>>,
    incoming_latencies: Arc<dashmap::DashMap<NodeId, RollingAvg>>,
}

impl LatencyEstimationImpl {
    pub fn new(node_situation: Arc<dyn NodeSituation>) -> Self {
        Self {
            node_situation,
            outgoing_latencies: Arc::new(dashmap::DashMap::new()),
            incoming_latencies: Arc::new(dashmap::DashMap::new()),
        }
    }

    /// Do the packet exchanges to get the latencies and return (outgoing,
    /// incoming)
    async fn make_latency_request_to(
        &self,
        node_id: &NodeId,
    ) -> Result<(Time, Time), IndividualError> {
        let desc = self
            .node_situation
            .get_fog_node_neighbor(node_id)
            .ok_or_else(|| IndividualError::NodeNotFound(node_id.clone()))?;

        let ip = desc.ip;
        let port = desc.port;

        let client = reqwest::Client::new();
        let sent_at = Instant::now();
        let _response = client
            .head(format!("http://{}:{}/api/health", ip, port).as_str())
            .send()
            .await?;
        let elapsed = sent_at.elapsed().as_millis();

        let latency = elapsed as f64 / 2.0;
        let outgoing_latency =
            Time::new::<uom::si::time::millisecond>(latency);
        let incoming_latency = outgoing_latency;

        Ok((outgoing_latency, incoming_latency))
    }
}

#[async_trait]
impl LatencyEstimation for LatencyEstimationImpl {
    async fn latency_to_neighbors(&self) -> Result<(), Error> {
        let mut handles = Vec::new();
        let mut tried_nodes = Vec::new(); // same order as handles
        for node in self.node_situation.get_neighbors() {
            tried_nodes.push(node.clone());
            handles.push(async move {
                let (incoming, outgoing) =
                    self.make_latency_request_to(&node).await?;

                Self::update_latency(
                    &self.incoming_latencies,
                    &node,
                    incoming,
                );
                Self::update_latency(
                    &self.outgoing_latencies,
                    &node,
                    outgoing,
                );

                let desc = self
                    .node_situation
                    .get_fog_node_neighbor(&node)
                    .ok_or_else(|| {
                        IndividualError::NodeNotFound(node.clone())
                    })?;

                let ip = desc.ip;
                let port = desc.port;
                crate::prom_metrics::LATENCY_NEIGHBORS_GAUGE
                    .with_label_values(&[&format!("{}:{}", ip, port)])
                    .set(outgoing.value);

                if let Some(lat) = self.outgoing_latencies.get(&node) {
                    crate::prom_metrics::LATENCY_NEIGHBORS_AVG_GAUGE
                        .with_label_values(&[&format!("{}:{}", ip, port)])
                        .set(lat.get_avg().value);
                }
                Ok(())
            });
        }

        let attempts = handles.len();
        let errors: Vec<(NodeId, IndividualError)> =
            futures::future::join_all(handles)
                .await
                .into_iter()
                .zip(tried_nodes.into_iter())
                .filter(|(result, _id)| result.is_err())
                .map(|(result, id)| (id, result.err().unwrap()))
                .collect();

        if !errors.is_empty() {
            return Err(Error::FailedPing(
                attempts,
                errors.len(),
                IndividualErrorList::from(errors),
            ));
        }
        Ok(())
    }

    async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time> {
        self.outgoing_latencies.get(id).map(|avg| avg.get_avg())
    }

    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time> {
        self.incoming_latencies.get(id).map(|avg| avg.get_avg())
    }
}

impl LatencyEstimationImpl {
    fn update_latency(
        map: &dashmap::DashMap<NodeId, RollingAvg>,
        key: &NodeId,
        value: Time,
    ) {
        let mut entry = map
            .get(key)
            .map(|entry| entry.value().clone())
            .unwrap_or_else(RollingAvg::default);
        entry.update(value);
        map.insert(key.clone(), entry);
    }
}
