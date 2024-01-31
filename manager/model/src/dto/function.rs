use crate::domain::sla::Sla;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct FunctionRecord<State>(pub State);

#[derive(Debug, Clone)]
pub struct Proposed {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

#[derive(Debug, Clone)]
pub struct Reserved {
    pub bid:  f64,
    pub sla:  Sla,
    pub node: String,
}

#[derive(Debug, Clone)]
pub struct Provisioned {
    pub bid:           f64,
    pub sla:           Sla,
    pub node:          String,
    pub function_name: String,
}
#[derive(Debug, Clone)]
pub struct Live {
    pub bid:           f64,
    pub sla:           Sla,
    pub node:          String,
    pub function_name: String,
}

#[derive(Debug, Clone)]
pub struct Finished {
    pub bid: f64,
    pub sla: Sla,
}

impl FunctionRecord<Proposed> {
    pub fn new(bid: f64, sla: Sla, node: String) -> Self {
        Self(Proposed { bid, sla, node })
    }

    pub fn to_provisioned(
        self,
        function_name: String,
    ) -> FunctionRecord<Provisioned> {
        FunctionRecord(Provisioned {
            function_name,
            bid: self.0.bid,
            sla: self.0.sla,
            node: self.0.node,
        })
    }
}

impl FunctionRecord<Provisioned> {
    pub fn to_live(self) -> FunctionRecord<Live> {
        FunctionRecord(Live {
            function_name: self.0.function_name,
            bid:           self.0.bid,
            sla:           self.0.sla,
            node:          self.0.node,
        })
    }
}

impl FunctionRecord<Live> {
    pub fn to_finished(self) -> FunctionRecord<Finished> {
        FunctionRecord(Finished { bid: self.0.bid, sla: self.0.sla })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ChosenBid {
    pub bid:   crate::view::auction::BidProposal,
    pub price: f64,
}
