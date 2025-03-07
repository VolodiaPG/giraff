use crate::domain::sla::Sla;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct Proposed {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

#[derive(Debug, Clone)]
pub struct Paid {
    pub bid:     f64,
    pub sla:     Sla,
    pub node:    String,
    pub paid_at: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct Provisioned {
    pub bid:           f64,
    pub sla:           Sla,
    pub node:          String,
    pub function_name: String,
    pub opened_port:   i32,
}

#[derive(Debug, Clone)]
pub struct Live {
    pub bid:           f64,
    pub sla:           Sla,
    pub node:          String,
    pub function_name: String,
    pub opened_port:   i32,
}

#[derive(Debug, Clone)]
pub struct Finished {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

pub trait Finishable {
    fn to_finished(&self) -> Finished;
}

impl Proposed {
    pub fn new(bid: f64, sla: Sla, node: String) -> Self {
        Self { bid, sla, node }
    }

    pub fn to_paid(self, paid_at: DateTime<Utc>) -> Paid {
        Paid { bid: self.bid, sla: self.sla, node: self.node, paid_at }
    }
}

impl Paid {
    pub fn to_provisioned(
        self,
        function_name: String,
        opened_port: i32,
    ) -> Provisioned {
        Provisioned {
            function_name,
            bid: self.bid,
            sla: self.sla,
            node: self.node,
            opened_port,
        }
    }
}
impl Finishable for Paid {
    fn to_finished(&self) -> Finished {
        Finished {
            bid:  self.bid.clone(),
            sla:  self.sla.clone(),
            node: self.node.clone(),
        }
    }
}
impl Provisioned {
    pub fn to_live(self) -> Live {
        Live {
            function_name: self.function_name,
            bid:           self.bid,
            sla:           self.sla,
            node:          self.node,
            opened_port:   self.opened_port,
        }
    }
}

impl Finishable for Provisioned {
    fn to_finished(&self) -> Finished {
        Finished {
            bid:  self.bid.clone(),
            sla:  self.sla.clone(),
            node: self.node.clone(),
        }
    }
}

impl Finishable for Live {
    fn to_finished(&self) -> Finished {
        Finished {
            bid:  self.bid.clone(),
            sla:  self.sla.clone(),
            node: self.node.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChosenBid {
    pub bid:   crate::view::auction::BidProposal,
    pub price: f64,
}
