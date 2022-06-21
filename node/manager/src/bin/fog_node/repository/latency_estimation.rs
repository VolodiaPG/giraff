use std::collections::HashMap;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uom::si::f64::Time;

use manager::model::domain::rolling_avg::RollingAvg;
use manager::model::view::ping::{Ping, PingResponse};
use manager::model::NodeId;

use crate::NodeSituation;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Rtt estimation was carried for {0} nodes, got {1} errors: {2}")]
    FailedPing(usize, usize, IndividualErrorList),
}

#[derive(Debug, thiserror::Error)]
pub enum IndividualError {
    #[error("Got negative RTTs")]
    NegativeTimeInterval,
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
    /// Make the requests to the neighbors to get the Latency to our children + parent.
    async fn latency_to_neighbors(&self) -> Result<(), Error>;
    async fn get_latency_to_avg(&self, id: &NodeId) -> Option<Time>;
    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time>;
}

#[derive(Debug)]
pub struct LatencyEstimationImpl {
    node_situation: Arc<dyn NodeSituation>,
    outgoing_latencies: Arc<RwLock<HashMap<NodeId, RollingAvg>>>,
    incoming_latencies: Arc<RwLock<HashMap<NodeId, RollingAvg>>>,
}

impl LatencyEstimationImpl {
    pub fn new(node_situation: Arc<dyn NodeSituation>) -> Self {
        Self {
            node_situation,
            outgoing_latencies: Arc::new(RwLock::new(HashMap::new())),
            incoming_latencies: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Compute latencies and return (outgoing, incoming)
    #[inline]
    fn compute_latency(
        &self,
        ping: &Ping,
        response: &PingResponse,
        received_at: &chrono::DateTime<chrono::Utc>,
    ) -> Result<(Time, Time), IndividualError> {
        // let latency =
        //     (response.received_at.timestamp_millis() - ping.sent_at.timestamp_millis()) as i64;
        // if latency < 0 {
        //     warn!("Got negative outbound latency: {}", latency);
        //     return Err(IndividualError::NegativeTimeInterval);
        // }
        // let outgoing_latency = Time::new::<uom::si::time::millisecond>(latency as f64);
        //
        // let latency =
        //     (received_at.timestamp_millis() - response.received_at.timestamp_millis()) as i64;
        // if latency < 0 {
        //     warn!("Got negative inbound latency: {}", latency);
        //     return Err(IndividualError::NegativeTimeInterval);
        // }
        // let incoming_latency = Time::new::<uom::si::time::millisecond>(latency as f64);

        // TODO fix that simplistic estimation with somthing considering the real values instead of symetric ones!!

        let latency_round =
            (received_at.timestamp_millis() - ping.sent_at.timestamp_millis()) as i64;

        if latency_round < 0 {
            warn!("Got negative latency: {}", latency_round);
            return Err(IndividualError::NegativeTimeInterval);
        }

        let outgoing_latency =
            Time::new::<uom::si::time::millisecond>((latency_round as f64) / 2.0);
        let incoming_latency =
            Time::new::<uom::si::time::millisecond>((latency_round as f64) / 2.0);

        Ok((outgoing_latency, incoming_latency))
    }

    /// Do the packet exchanges to get the latencies and return (outgoing, incoming)
    async fn make_latency_request_to(
        &self,
        node_id: &NodeId,
    ) -> Result<(Time, Time), IndividualError> {
        let desc = self
            .node_situation
            .get_fog_node_neighbor(node_id)
            .await
            .ok_or_else(|| IndividualError::NodeNotFound(node_id.clone()))?;

        let ip = desc.ip;
        let port = desc.port;

        let client = reqwest::Client::new();
        let ping = Ping {
            sent_at: chrono::Utc::now(),
        };
        let response = client
            .post(format!("http://{}:{}/api/ping", ip, port).as_str())
            .json(&ping)
            .send()
            .await?;
        let received_at = chrono::Utc::now();
        let response: PingResponse = response.json().await?;

        self.compute_latency(&ping, &response, &received_at)
    }
}

#[async_trait]
impl LatencyEstimation for LatencyEstimationImpl {
    async fn latency_to_neighbors(&self) -> Result<(), Error> {
        let mut handles = Vec::new();
        let mut tried_nodes = Vec::new(); // same order as handles
        for node in self.node_situation.get_neighbors().await {
            tried_nodes.push(node.clone());
            handles.push(async move {
                let (incoming, outgoing) = self.make_latency_request_to(&node).await?;
                self.incoming_latencies
                    .write()
                    .await
                    .entry(node.clone())
                    .or_default()
                    .update(incoming);
                self.outgoing_latencies
                    .write()
                    .await
                    .entry(node.clone())
                    .or_default()
                    .update(outgoing);
                Ok(())
            });
        }

        let attempts = handles.len();
        let errors: Vec<(NodeId, IndividualError)> = futures::future::join_all(handles)
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
        self.outgoing_latencies
            .read()
            .await
            .get(id)
            .map(|avg| avg.get_avg())
    }

    async fn get_latency_from_avg(&self, id: &NodeId) -> Option<Time> {
        self.incoming_latencies
            .read()
            .await
            .get(id)
            .map(|avg| avg.get_avg())
    }
}
