use super::{Latency, LatencyEstimation};
use crate::monitoring::NeighborLatency;
use crate::NodeSituation;
use anyhow::{bail, ensure, Context, Result};
use async_trait::async_trait;
use chrono::Utc;
use helper::monitoring::MetricsExporter;
use model::domain::exp_average::ExponentialMovingAverage;
use model::domain::moving_median::{MovingMedian, MovingMedianSize};
use model::NodeId;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::Duration;
use surge_ping::{Client, Config, PingIdentifier, PingSequence};
use tokio::time::interval;
use uom::si::f64::{Ratio, Time};
use uom::si::ratio::ratio;
use uom::si::time::millisecond;

const NB_ICMP_SENT: u16 = 10;
const SAMPLING_TIME_MS: u16 = 250;

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
struct Latencies {
    moving_average: ExponentialMovingAverage,
    moving_median:  MovingMedian,
    packet_loss:    PacketLossRing,
}

#[derive(Debug)]
struct PacketLossRing {
    buffer:      Vec<Ratio>,
    cursor:      usize,
    window_size: usize,
}

impl PacketLossRing {
    pub fn new(window_size: usize) -> Self {
        Self {
            buffer: Vec::with_capacity(window_size),
            cursor: 0,
            window_size,
        }
    }

    pub fn update(&mut self, value: Ratio) {
        if self.buffer.len() < self.window_size {
            self.buffer.push(value);
        } else {
            self.buffer[self.cursor] = value;
        }
        self.cursor = (self.cursor + 1) % self.window_size;
    }

    pub fn get(&self) -> Ratio {
        let mut sum = Ratio::new::<ratio>(0.0);
        for ii in 0..(self.buffer.len()) {
            sum += self.buffer[ii];
        }

        sum / self.buffer.len() as f64
    }
}

#[derive(Debug)]
pub struct LatencyEstimationImpl {
    node_situation:            Arc<NodeSituation>,
    metrics:                   Arc<MetricsExporter>,
    latency:                   Arc<dashmap::DashMap<NodeId, Latencies>>,
    alpha:                     model::domain::exp_average::Alpha,
    moving_median_window_size: MovingMedianSize,
}
#[async_trait]
impl LatencyEstimation for LatencyEstimationImpl {
    async fn latency_to_neighbors(&self) -> Result<()> {
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

    async fn get_latency_to(&self, id: &NodeId) -> Option<Latency> {
        self.latency.get(id).and_then(|x| {
            let latencies = x.value();
            match (
                latencies.moving_median.median(),
                latencies.moving_median.interquantile_range(),
                latencies.moving_average.get(),
                latencies.packet_loss.get(),
            ) {
                (
                    Some(median),
                    Some(interquantile_range),
                    average,
                    packet_loss,
                ) => Some(Latency {
                    median,
                    interquantile_range,
                    average,
                    packet_loss,
                }),
                _ => None,
            }
        })
    }
}

impl LatencyEstimationImpl {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        metrics: Arc<MetricsExporter>,
        alpha: model::domain::exp_average::Alpha,
        moving_median_window_size: MovingMedianSize,
    ) -> Self {
        Self {
            node_situation,
            metrics,
            latency: Arc::new(dashmap::DashMap::new()),
            alpha,
            moving_median_window_size,
        }
    }

    async fn ping(&self, node: NodeId) -> Result<()> {
        let ip = self
            .node_situation
            .get_fog_node_neighbor(&node)
            .with_context(|| format!("Failed to find node {node}"))?
            .ip;
        let port = self
            .node_situation
            .get_fog_node_neighbor(&node)
            .with_context(|| format!("Failed to find node {node}"))?
            .port_http;
        let my_ip = self.node_situation.get_my_public_ip();
        let my_port = self.node_situation.get_my_public_port_http();

        let client = Client::new(&Config::default())?;
        let payload = [0; 56];
        let mut pinger =
            client.pinger(ip, PingIdentifier(rand::random())).await;
        pinger.timeout(Duration::from_secs(1));
        let mut interval =
            interval(Duration::from_millis(SAMPLING_TIME_MS.into()));
        let mut durations = Time::new::<millisecond>(0.0);
        let mut nb_failed: u16 = 0;
        for idx in 0..NB_ICMP_SENT {
            interval.tick().await;
            match pinger.ping(PingSequence(idx), &payload).await {
                Ok((_, dur)) => {
                    let raw_latency = Time::new::<uom::si::time::millisecond>(
                        dur.as_millis() as f64 / 2.0,
                    );
                    durations += raw_latency;
                }
                Err(_e) => nb_failed += 1,
            };
        }

        if nb_failed == NB_ICMP_SENT {
            bail!("All ICMP requests failed");
        }

        let nb = (NB_ICMP_SENT - nb_failed) as f64;
        let raw_latency = durations / nb;
        let raw_packet_loss = (nb_failed / NB_ICMP_SENT) as f64;

        self.update_latency(
            &self.latency,
            &node,
            raw_latency,
            Ratio::new::<ratio>(raw_packet_loss),
        );

        let latency = self.get_latency_to(&node).await;
        if let Some(latency) = latency {
            self.metrics
                .observe(NeighborLatency {
                    raw_packet_loss,
                    packet_loss: latency.packet_loss.get::<ratio>(),
                    raw: raw_latency.get::<millisecond>(),
                    average: latency.average.get::<millisecond>(),
                    median: latency.median.get::<millisecond>(),
                    interquartile_range: latency
                        .interquantile_range
                        .get::<millisecond>(),
                    instance_to: format!("{}:{}", ip, port),
                    instance_address: format!("{}:{}", my_ip, my_port),
                    timestamp: Utc::now(),
                })
                .await?;
        }
        Ok(())
    }

    fn update_latency(
        &self,
        map: &dashmap::DashMap<NodeId, Latencies>,
        key: &NodeId,
        value: Time,
        packet_loss: Ratio,
    ) {
        match map.get_mut(key) {
            Some(mut entry) => {
                entry.value_mut().moving_average.update(value);
                entry.value_mut().moving_median.update(value);
                entry.value_mut().packet_loss.update(packet_loss);
            }
            None => {
                let moving_average =
                    ExponentialMovingAverage::new(self.alpha.clone(), value);
                let mut moving_median =
                    MovingMedian::new(self.moving_median_window_size.clone());
                moving_median.update(value);

                let mut packet_loss_ring = PacketLossRing::new(
                    self.moving_median_window_size.clone().into_inner(),
                );
                packet_loss_ring.update(packet_loss);

                map.insert(
                    key.clone(),
                    Latencies {
                        moving_average,
                        moving_median,
                        packet_loss: packet_loss_ring,
                    },
                );
            }
        };
    }
}
