use std::cmp::Ordering;
use std::net::IpAddr;

use crate::FogNodeFaaSPortExternal;
use serde::{Deserialize, Serialize};
use uom::si::f64::{Ratio, Time};
use uom::si::ratio::ratio;
use uom::si::time::millisecond;

use super::super::domain::sla::Sla;
use super::super::{BidId, NodeId};

#[derive(Debug)]
pub struct Latency {
    pub median:              Time,
    pub average:             Time,
    pub interquantile_range: Time,
    pub packet_loss:         Ratio,
}

#[serde_with::serde_as]
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct AccumulatedLatency {
    #[serde_as(as = "helper::uom_helper::time::Helper")]
    pub median:             Time,
    #[serde_as(as = "helper::uom_helper::time::Helper")]
    pub average:            Time,
    #[serde_as(as = "helper::uom_helper::time::Helper")]
    pub median_uncertainty: Time,
    #[serde_as(as = "helper::uom_helper::ratio::Helper")]
    pub packet_loss:        Ratio,
}

impl Default for AccumulatedLatency {
    fn default() -> Self {
        Self {
            median:             Time::new::<millisecond>(0.0),
            average:            Time::new::<millisecond>(0.0),
            median_uncertainty: Time::new::<millisecond>(0.0),
            packet_loss:        uom::si::f64::Ratio::new::<ratio>(0.0),
        }
    }
}

impl AccumulatedLatency {
    pub fn accumulate(&self, latency: Latency) -> Self {
        let median = self.median + latency.median;
        let average = self.average + latency.average;

        let std_deviation =
            latency.interquantile_range.get::<millisecond>() / 1.349;
        let uncertainty = self.median_uncertainty.get::<millisecond>();
        let median_uncertainty = Time::new::<millisecond>(
            (std_deviation.powi(2) + uncertainty.powi(2)).powf(0.5),
        );

        let packet_loss = self.packet_loss * latency.packet_loss;

        Self { median, average, median_uncertainty, packet_loss }
    }
}
#[serde_with::serde_as]
#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BidRequest<'a> {
    pub node_origin:              NodeId,
    pub sla:                      &'a Sla,
    pub accumulated_latency:      AccumulatedLatency,
    #[cfg(feature = "mincpurandom")]
    pub nb_propositions_required: usize,
}

/// Same as [`BidRequest`](BidRequest) but with an owned SLA
#[serde_with::serde_as]
#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BidRequestOwned {
    pub node_origin:              NodeId,
    pub sla:                      Sla,
    pub accumulated_latency:      AccumulatedLatency,
    #[cfg(feature = "mincpurandom")]
    pub nb_propositions_required: usize,
}

/// A bid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Bid {
    pub bid: f64,
    pub sla: Sla,
    pub id:  BidId,
}

/// The accepted bid
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AcceptedBid {
    pub chosen:    InstanciatedBid,
    pub proposals: BidProposals,
    pub sla:       Sla,
}

/// The bid proposal and the node who issued it
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct BidProposal {
    pub node_id: NodeId,
    pub id:      BidId,
    pub bid:     f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct BidProposals {
    pub bids: Vec<BidProposal>,
}

impl PartialOrd for BidProposal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.bid.partial_cmp(&other.bid)
    }
}

impl PartialEq for BidProposal {
    fn eq(&self, other: &Self) -> bool { self.bid == other.bid }
}

impl Eq for BidProposal {}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct InstanciatedBid {
    pub bid:   BidProposal,
    pub ip:    IpAddr,
    pub port:  FogNodeFaaSPortExternal,
    pub price: f64,
}
