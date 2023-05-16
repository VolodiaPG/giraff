use crate::NodeSituation;
use anyhow::{ensure, Context, Result};
use model::domain::exp_average::ExponentialMovingAverage;
use model::domain::moving_median::{MovingMedian, MovingMedianSize};
use model::NodeId;
use std::fmt;
use std::fmt::Debug;
use std::sync::Arc;
use uom::si::f64::Time;
use uom::si::time::millisecond;

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
}

#[derive(Debug)]
pub struct Latency {
    pub median:              Time,
    pub average:             Time,
    pub interquantile_range: Time,
}

#[derive(Debug)]
pub struct LatencyEstimation {
    node_situation:            Arc<NodeSituation>,
    latency:                   Arc<dashmap::DashMap<NodeId, Latencies>>,
    alpha:                     model::domain::exp_average::Alpha,
    moving_median_window_size: MovingMedianSize,
}

impl LatencyEstimation {
    pub fn new(
        node_situation: Arc<NodeSituation>,
        alpha: model::domain::exp_average::Alpha,
        moving_median_window_size: MovingMedianSize,
    ) -> Self {
        Self {
            node_situation,
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

        if let Some(latency) = latency {
            crate::prom_metrics::LATENCY_NEIGHBORS_AVG_GAUGE
                .with_label_values(&[&format!("{ip}")])
                .set(latency.average.get::<millisecond>());

            crate::prom_metrics::LATENCY_NEIGHBORS_MEDIAN_GAUGE
                .with_label_values(&[&format!("{ip}")])
                .set(latency.median.get::<millisecond>());

            crate::prom_metrics::LATENCY_NEIGHBORS_INTERQUARTILE_RANGE_GAUGE
                .with_label_values(&[&format!("{ip}")])
                .set(latency.interquantile_range.get::<millisecond>());
        }

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

    pub async fn get_latency_to(&self, id: &NodeId) -> Option<Latency> {
        self.latency.get(id).and_then(|x| {
            let latencies = x.value();
            match (
                latencies.moving_median.median(),
                latencies.moving_median.interquantile_range(),
                latencies.moving_average.get(),
            ) {
                (Some(median), Some(interquantile_range), average) => {
                    Some(Latency { median, interquantile_range, average })
                }
                _ => None,
            }
        })
    }

    fn update_latency(
        &self,
        map: &dashmap::DashMap<NodeId, Latencies>,
        key: &NodeId,
        value: Time,
    ) -> Option<Latency> {
        let Some(mut entry) = map.get_mut(key)
            else{
                let moving_average = ExponentialMovingAverage::new(self.alpha.clone(), value);
                let mut moving_median = MovingMedian::new(self.moving_median_window_size.clone());
                moving_median.update(value);

                map.insert(
                    key.clone(),
                    Latencies{
                        moving_average,
                        moving_median,
                    }
                );

                return None;
            };

        entry.value_mut().moving_average.update(value);
        entry.value_mut().moving_median.update(value);

        match (
            entry.value().moving_median.median(),
            entry.value().moving_median.interquantile_range(),
            entry.value().moving_average.get(),
        ) {
            (Some(median), Some(interquantile_range), average) => {
                Some(Latency { average, median, interquantile_range })
            }
            _ => None,
        }
    }
}
